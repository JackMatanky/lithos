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
	"sync"

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
	mu        sync.RWMutex
	signature string
	cache     []domain.Schema
	cacheBank domain.PropertyBank
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
		mu:        sync.RWMutex{},
		signature: "",
		cache:     nil,
		cacheBank: domain.PropertyBank{
			Properties: make(map[string]domain.Property),
		},
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

	// Try to use cache first
	schemas, bank, useCache := a.tryUseCache()
	if useCache {
		return schemas, bank, nil
	}

	// Load fresh data
	return a.loadFreshData(ctx)
}

// tryUseCache attempts to return cached results if available and valid.
func (a *SchemaLoaderAdapter) tryUseCache() ([]domain.Schema, domain.PropertyBank, bool) {
	signature, sigErr := a.computeSignature()
	if sigErr != nil {
		return nil, domain.PropertyBank{}, false
	}

	a.mu.RLock()
	defer a.mu.RUnlock()

	if signature == a.signature && len(a.cache) > 0 {
		if a.log != nil {
			a.log.Debug().Msg("using cached schemas and property bank")
		}
		return cloneSchemas(a.cache), clonePropertyBank(a.cacheBank), true
	}

	return nil, domain.PropertyBank{}, false
}

// loadFreshData loads and processes schemas from scratch.
func (a *SchemaLoaderAdapter) loadFreshData(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	bank, err := a.loadAndLogPropertyBank(ctx)
	if err != nil {
		return nil, domain.PropertyBank{}, err
	}

	schemas, err := a.loadAndLogSchemas(ctx, bank)
	if err != nil {
		return nil, domain.PropertyBank{}, err
	}

	extendedSchemas, err := a.validateAndExtendSchemas(ctx, schemas)
	if err != nil {
		return nil, domain.PropertyBank{}, err
	}

	a.updateCache(extendedSchemas, bank)
	return extendedSchemas, bank, nil
}

// loadAndLogPropertyBank loads the property bank with logging.
func (a *SchemaLoaderAdapter) loadAndLogPropertyBank(
	ctx context.Context,
) (domain.PropertyBank, error) {
	bank, err := a.loadPropertyBank(ctx)
	if err != nil {
		return domain.PropertyBank{}, err
	}

	if a.log != nil {
		a.log.Debug().
			Str("path", a.config.PropertyBankPath()).
			Int("properties", len(bank.Properties)).
			Msg("property bank loaded")
	}

	return bank, nil
}

// loadAndLogSchemas loads schemas with logging.
func (a *SchemaLoaderAdapter) loadAndLogSchemas(
	ctx context.Context, bank domain.PropertyBank,
) ([]domain.Schema, error) {
	schemas, err := a.loadSchemas(ctx, bank)
	if err != nil {
		return nil, err
	}

	if a.log != nil {
		a.log.Debug().
			Int("count", len(schemas)).
			Str("directory", a.config.SchemasDir).
			Msg("raw schemas loaded")
	}

	return schemas, nil
}

// validateAndExtendSchemas validates and resolves inheritance.
func (a *SchemaLoaderAdapter) validateAndExtendSchemas(
	ctx context.Context, schemas []domain.Schema,
) ([]domain.Schema, error) {
	if err := a.validateSchemas(ctx, schemas); err != nil {
		return nil, err
	}

	extendedSchemas, err := a.resolveInheritance(ctx, schemas)
	if err != nil {
		return nil, err
	}

	if a.log != nil {
		a.log.Debug().
			Int("count", len(extendedSchemas)).
			Msg("schemas validated and inheritance resolved")
	}

	return extendedSchemas, nil
}

// updateCache stores the results in cache.
func (a *SchemaLoaderAdapter) updateCache(
	schemas []domain.Schema, bank domain.PropertyBank,
) {
	signature, sigErr := a.computeSignature()
	if sigErr == nil {
		a.mu.Lock()
		a.signature = signature
		a.cache = cloneSchemas(schemas)
		a.cacheBank = clonePropertyBank(bank)
		a.mu.Unlock()
	}
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

func (a *SchemaLoaderAdapter) computeSignature() (string, error) {
	var parts []string

	bankInfo, err := os.Stat(a.config.PropertyBankPath())
	switch {
	case err == nil:
		parts = append(parts, fmt.Sprintf(
			"bank:%d:%d",
			bankInfo.ModTime().UnixNano(),
			bankInfo.Size(),
		))
	case errors.Is(err, os.ErrNotExist):
		parts = append(parts, "bank:none")
	default:
		return "", err
	}

	err = a.walkDir(
		a.config.SchemasDir,
		func(path string, d fs.DirEntry, walkErr error) error {
			if walkErr != nil {
				return walkErr
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

			fileInfo, statErr := d.Info()
			if statErr != nil {
				return statErr
			}

			parts = append(parts, fmt.Sprintf(
				"schema:%s:%d:%d",
				filepath.ToSlash(path),
				fileInfo.ModTime().UnixNano(),
				fileInfo.Size(),
			))
			return nil
		},
	)
	if err != nil {
		return "", err
	}

	sort.Strings(parts)
	return strings.Join(parts, "|"), nil
}

func cloneSchemas(schemas []domain.Schema) []domain.Schema {
	if len(schemas) == 0 {
		return nil
	}
	cloned := make([]domain.Schema, len(schemas))
	copy(cloned, schemas)
	return cloned
}

func clonePropertyBank(bank domain.PropertyBank) domain.PropertyBank {
	if len(bank.Properties) == 0 {
		return domain.PropertyBank{Properties: map[string]domain.Property{}}
	}
	props := make(map[string]domain.Property, len(bank.Properties))
	for k, v := range bank.Properties {
		props[k] = v
	}
	return domain.PropertyBank{Properties: props}
}
