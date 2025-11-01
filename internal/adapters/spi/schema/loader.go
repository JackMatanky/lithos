package schema

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// SchemaLoaderAdapter implements SchemaPort by loading schema JSON files and
// the property bank from the filesystem configured in domain.Config.
// It validates and resolves inheritance for loaded schemas.
type SchemaLoaderAdapter struct {
	config    *domain.Config
	log       *zerolog.Logger
	readFile  func(string) ([]byte, error)
	walkDir   func(string, fs.WalkDirFunc) error
	validator *SchemaValidator
	extender  *SchemaExtender
}

// NewSchemaLoaderAdapter creates a new filesystem-backed schema loader.
func NewSchemaLoaderAdapter(
	config *domain.Config,
	log *zerolog.Logger,
) *SchemaLoaderAdapter {
	return &SchemaLoaderAdapter{
		config:    config,
		log:       log,
		readFile:  os.ReadFile,
		walkDir:   filepath.WalkDir,
		validator: NewSchemaValidator(),
		extender:  NewSchemaExtender(),
	}
}

// Load loads the property bank first, then all schema documents, validates
// them, resolves inheritance, and returns fully processed schemas alongside
// the shared property bank.
func (a *SchemaLoaderAdapter) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	if err := ctx.Err(); err != nil {
		return nil, domain.PropertyBank{}, err
	}

	var err error
	bank, err := a.loadPropertyBank(ctx)
	if err != nil {
		return nil, domain.PropertyBank{}, err
	}

	if a.log != nil {
		a.log.Debug().
			Str("path", a.config.PropertyBankPath()).
			Int("properties", len(bank.Properties)).
			Msg("property bank loaded")
	}

	schemas, err := a.loadSchemas(ctx, bank)
	if err != nil {
		return nil, domain.PropertyBank{}, err
	}

	if a.log != nil {
		a.log.Debug().
			Int("count", len(schemas)).
			Str("directory", a.config.SchemasDir).
			Msg("raw schemas loaded")
	}

	// Validate schemas
	if validateErr := a.validateSchemas(ctx, schemas); validateErr != nil {
		return nil, domain.PropertyBank{}, validateErr
	}

	// Resolve inheritance
	extendedSchemas, err := a.resolveInheritance(ctx, schemas)
	if err != nil {
		return nil, domain.PropertyBank{}, err
	}

	if a.log != nil {
		a.log.Debug().
			Int("count", len(extendedSchemas)).
			Msg("schemas validated and inheritance resolved")
	}

	return extendedSchemas, bank, nil
}

func (a *SchemaLoaderAdapter) loadPropertyBank(
	ctx context.Context,
) (domain.PropertyBank, error) {
	path := a.config.PropertyBankPath()
	data, err := a.readFile(path)
	if err != nil {
		return domain.PropertyBank{}, wrapPropertyBankReadError(path, err)
	}

	var document propertyBankDTO
	if parseErr := json.Unmarshal(data, &document); parseErr != nil {
		return domain.PropertyBank{}, domainerrors.NewSchemaErrorWithRemediation(
			fmt.Sprintf("failed to parse property bank json: %s", path),
			"property_bank",
			syntaxRemediation(path),
			parseErr,
		)
	}

	return document.toDomain(ctx, path)
}

func (a *SchemaLoaderAdapter) loadSchemas(
	ctx context.Context,
	bank domain.PropertyBank,
) ([]domain.Schema, error) {
	var schemas []domain.Schema

	walkErr := a.walkDir(
		a.config.SchemasDir,
		func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if d.IsDir() {
				return nil
			}

			if filepath.Clean(
				path,
			) == filepath.Clean(
				a.config.PropertyBankPath(),
			) {
				return nil
			}

			if !strings.EqualFold(filepath.Ext(path), ".json") {
				return nil
			}

			schema, err := a.loadSchema(ctx, path, bank)
			if err != nil {
				return err
			}

			schemas = append(schemas, schema)
			return nil
		},
	)

	if walkErr != nil {
		return nil, domainerrors.NewResourceError(
			"schema",
			"scan",
			a.config.SchemasDir,
			fmt.Errorf("failed to scan schemas directory: %w", walkErr),
		)
	}

	sort.Slice(schemas, func(i, j int) bool {
		return schemas[i].Name < schemas[j].Name
	})

	return schemas, nil
}

func (a *SchemaLoaderAdapter) loadSchema(
	ctx context.Context,
	path string,
	bank domain.PropertyBank,
) (domain.Schema, error) {
	data, err := a.readFile(path)
	if err != nil {
		return domain.Schema{}, domainerrors.NewResourceError(
			"schema",
			"load",
			path,
			fmt.Errorf("failed to read schema file: %w", err),
		)
	}

	var document schemaDTO
	if parseErr := json.Unmarshal(data, &document); parseErr != nil {
		return domain.Schema{}, domainerrors.NewSchemaErrorWithRemediation(
			fmt.Sprintf("failed to parse schema json: %s", path),
			path,
			syntaxRemediation(path),
			parseErr,
		)
	}

	schema, err := document.toDomain(ctx, path, bank)
	if err != nil {
		return domain.Schema{}, err
	}
	return schema, nil
}

// validateSchemas performs validation on loaded schemas.
func (a *SchemaLoaderAdapter) validateSchemas(
	ctx context.Context,
	schemas []domain.Schema,
) error {
	if a.log != nil {
		a.log.Debug().Msg("validating schemas")
	}

	if err := a.validator.ValidateAll(ctx, schemas); err != nil {
		if a.log != nil {
			a.log.Error().Err(err).Msg("schema validation failed")
		}
		return fmt.Errorf("schema validation failed: %w", err)
	}

	if a.log != nil {
		a.log.Debug().Msg("schema validation complete")
	}

	return nil
}

// resolveInheritance resolves inheritance chains in schemas.
func (a *SchemaLoaderAdapter) resolveInheritance(
	ctx context.Context,
	schemas []domain.Schema,
) ([]domain.Schema, error) {
	if a.log != nil {
		a.log.Debug().Msg("resolving inheritance")
	}

	extendedSchemas, err := a.extender.ExtendSchemas(ctx, schemas)
	if err != nil {
		if a.log != nil {
			a.log.Error().Err(err).Msg("inheritance resolution failed")
		}
		return nil, fmt.Errorf("inheritance resolution failed: %w", err)
	}

	if a.log != nil {
		a.log.Debug().Msg("inheritance resolution complete")
	}

	return extendedSchemas, nil
}

// wrapPropertyBankReadError converts filesystem failures into ResourceErrors
// with targeted remediation hints.
func wrapPropertyBankReadError(path string, err error) error {
	if errors.Is(err, os.ErrNotExist) {
		return domainerrors.NewResourceError(
			"schema",
			"load",
			path,
			fmt.Errorf(
				"property bank missing: Create schemas/property_bank.json or configure PropertyBankFile: %w",
				err,
			),
		)
	}

	return domainerrors.NewResourceError(
		"schema",
		"load",
		path,
		fmt.Errorf("failed to read property bank file: %w", err),
	)
}

// syntaxRemediation constructs a consistent remediation hint for malformed
// JSON payloads.
func syntaxRemediation(path string) string {
	return fmt.Sprintf("Check JSON syntax in %s", path)
}
