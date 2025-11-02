package frontmatter

import (
	"bytes"
	"context"
	stderrors "errors"
	"sync"

	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"github.com/yuin/goldmark"
	"github.com/yuin/goldmark/parser"
	"go.abhg.dev/goldmark/frontmatter"
)

// FrontmatterService extracts and validates frontmatter from markdown content.
// It provides clean separation between frontmatter parsing and schema
// validation, enabling VaultIndexer to create valid Note objects with validated
// metadata.
//
// Architecture: Service layer orchestrates frontmatter operations using
// goldmark for parsing and schema validation for type checking.
//
// Dependencies:
//   - SchemaEngine: For loading and resolving schemas before validation
//   - Logger: For structured logging of extraction and validation operations
type FrontmatterService struct {
	schemaEngine *schema.SchemaEngine
	logger       zerolog.Logger
	markdown     goldmark.Markdown
	parserMu     sync.Mutex
}

// NewFrontmatterService creates a new FrontmatterService with required
// dependencies. It initializes the service for frontmatter extraction and
// validation operations.
//
// Parameters:
//   - schemaEngine: Required for schema loading and resolution
//   - logger: Required for observability and error tracking
//
// Returns a configured FrontmatterService ready for use.
func NewFrontmatterService(
	schemaEngine *schema.SchemaEngine,
	logger zerolog.Logger,
) *FrontmatterService {
	return &FrontmatterService{
		schemaEngine: schemaEngine,
		logger:       logger,
		markdown: goldmark.New(
			goldmark.WithExtensions(
				&frontmatter.Extender{
					Formats: frontmatter.DefaultFormats,
					Mode:    frontmatter.SetMetadata,
				},
			),
		),
		parserMu: sync.Mutex{},
	}
}

// Extract extracts frontmatter from markdown content using goldmark parser.
//
// This method supports both YAML (---) and TOML (+++) frontmatter delimiters
// and correctly handles edge cases like code blocks containing similar
// delimiters by using goldmark's robust parsing engine.
//
// Extraction Process:
//   - Uses goldmark with frontmatter extension for industry-standard parsing
//   - Distinguishes between actual frontmatter and code block delimiters
//   - Returns empty Frontmatter when no frontmatter is present
//   - Preserves all frontmatter fields without filtering
//
// Parameters:
//   - content: Raw markdown content as byte slice
//
// Returns:
//   - domain.Frontmatter: Parsed frontmatter with FileClass and Fields
//   - error: FrontmatterError for parsing failures, nil on success
//
// Error Handling:
//   - Returns structured FrontmatterError for goldmark parsing failures
//   - Gracefully handles malformed YAML/TOML (may return empty frontmatter)
//   - Never panics on invalid input
func (s *FrontmatterService) Extract(
	content []byte,
) (domain.Frontmatter, error) {
	// Parse markdown with frontmatter using goldmark
	frontmatterData, err := s.parseMarkdownWithFrontmatter(content)
	if err != nil {
		return domain.Frontmatter{}, errors.NewFrontmatterError(
			"failed to parse frontmatter from markdown",
			"",
			err,
		)
	}

	// Convert parsed data to domain Frontmatter
	return s.convertToFrontmatter(frontmatterData), nil
}

// Validate validates frontmatter against a schema using polymorphic field
// validators. Enforces FR6/FR7 requirements: preserves unknown fields,
// validates required fields, enforces type constraints, and validates array vs
// scalar expectations without auto-coercion.
//
// Validation Process:
//  1. Validates all required fields are present
//  2. Validates field types using appropriate field validators
//  3. Preserves unknown fields (FR6 compliance)
//  4. Enforces array vs scalar expectations
//  5. Aggregates all validation errors
//
// Parameters:
//   - ctx: Context for cancellation support
//   - fm: Frontmatter containing parsed fields to validate
//   - sch: Schema containing validation rules and property specifications
//
// Returns:
//   - error: Aggregated validation errors or nil if validation passes
//
// Error Handling:
//   - Returns structured FrontmatterError with field context
//   - Aggregates multiple validation errors using errors.Join
//   - Supports cancellation via context
func (s *FrontmatterService) Validate(
	ctx context.Context,
	fm domain.Frontmatter,
	sch domain.Schema,
) error {
	var validationErrors []error

	// Check for cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// Validate required fields
	if err := s.validateRequiredFields(fm, sch); err != nil {
		validationErrors = append(validationErrors, err)
	}

	// Validate field types using polymorphic validators
	if err := s.validateFieldTypes(fm, sch); err != nil {
		validationErrors = append(validationErrors, err)
	}

	// Unknown fields are preserved per FR6 - no validation needed

	// Aggregate validation errors
	return s.aggregateValidationErrors(validationErrors)
}

// parseMarkdownWithFrontmatter parses markdown content and extracts frontmatter
// using goldmark with frontmatter extension.
//
// This helper method configures goldmark with the frontmatter extension,
// parses the markdown content, and extracts any frontmatter data found.
// Uses goldmark's parser context to safely extract frontmatter without
// interference from code blocks or other markdown constructs.
//
// Returns empty map when no frontmatter is present (not an error condition).
func (s *FrontmatterService) parseMarkdownWithFrontmatter(
	content []byte,
) (map[string]any, error) {
	// Create parser context for frontmatter extraction
	// Context isolates frontmatter parsing from markdown rendering
	parserCtx := parser.NewContext()

	// Parse the markdown content
	// The frontmatter extension populates the context with extracted data
	var buf bytes.Buffer
	s.parserMu.Lock()
	convertErr := s.markdown.Convert(
		content,
		&buf,
		parser.WithContext(parserCtx),
	)
	s.parserMu.Unlock()
	if convertErr != nil {
		return nil, convertErr
	}

	// Extract frontmatter data from parser context
	frontmatterData := frontmatter.Get(parserCtx)
	if frontmatterData == nil {
		// No frontmatter found - return empty map (not an error)
		return make(map[string]any), nil
	}

	// Decode frontmatter.Data into standard map[string]any
	result := make(map[string]any)
	if err := frontmatterData.Decode(&result); err != nil {
		return nil, err
	}

	return result, nil
}

// convertToFrontmatter converts parsed frontmatter data to domain Frontmatter.
//
// This helper method creates a domain.Frontmatter instance from the raw
// frontmatter data map. The domain constructor handles FileClass extraction
// and field preservation automatically.
//
// Design Note: Separated as a helper method to enable easy extension
// for future frontmatter processing requirements (e.g., field transformation,
// validation preprocessing).
func (s *FrontmatterService) convertToFrontmatter(
	data map[string]any,
) domain.Frontmatter {
	return domain.NewFrontmatter(data)
}

// validateRequiredFields validates that all required fields are present in
// frontmatter. Helper method for FrontmatterService.Validate to check required
// field constraints.
func (s *FrontmatterService) validateRequiredFields(
	fm domain.Frontmatter,
	sch domain.Schema,
) error {
	var validationErrors []error

	for _, property := range sch.Properties {
		if property.Required {
			if _, exists := fm.Fields[property.Name]; !exists {
				validationErrors = append(
					validationErrors,
					errors.NewFrontmatterError(
						"required field missing",
						property.Name,
						nil,
					),
				)
			}
		}
	}

	if len(validationErrors) > 0 {
		return stderrors.Join(validationErrors...)
	}
	return nil
}

// validateFieldTypes validates frontmatter field types using polymorphic
// validators. Helper method for FrontmatterService.Validate to perform
// type-specific validation.
func (s *FrontmatterService) validateFieldTypes(
	fm domain.Frontmatter,
	sch domain.Schema,
) error {
	var validationErrors []error

	// Create validator instances
	stringValidator := &StringValidator{}
	numberValidator := &NumberValidator{}
	dateValidator := &DateValidator{}
	boolValidator := &BoolValidator{}

	for _, property := range sch.Properties {
		value, exists := fm.Fields[property.Name]
		if !exists {
			// Field not present - not an error for type validation
			continue
		}

		// Select appropriate validator based on property spec type
		var validator FieldValidator
		switch property.Spec.Type() {
		case domain.PropertyTypeString:
			validator = stringValidator
		case domain.PropertyTypeNumber:
			validator = numberValidator
		case domain.PropertyTypeDate:
			validator = dateValidator
		case domain.PropertyTypeBool:
			validator = boolValidator
		default:
			// Unknown property type - skip validation
			continue
		}

		// Validate field value using appropriate validator
		if err := validator.Validate(property.Name, value, property.Spec); err != nil {
			validationErrors = append(validationErrors, err)
		}
	}

	if len(validationErrors) > 0 {
		return stderrors.Join(validationErrors...)
	}
	return nil
}

// aggregateValidationErrors aggregates multiple validation errors into a single
// error.
// Helper method for FrontmatterService.Validate to combine validation results.
func (s *FrontmatterService) aggregateValidationErrors(
	validationErrors []error,
) error {
	if len(validationErrors) == 0 {
		return nil
	}
	return stderrors.Join(validationErrors...)
}
