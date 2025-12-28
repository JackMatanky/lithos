package frontmatter

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"strings"
	"time"

	"github.com/JackMatanky/lithos/internal/app/events"
	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// QueryServicePort defines the interface needed for FileSpec validation.
// This allows for dependency injection of query services in tests.
type QueryServicePort interface {
	PathQuery(
		ctx context.Context,
		opts spi.PathQueryOptions,
	) ([]domain.Note, error)
}

// FrontmatterService validates frontmatter against schema rules with semantic
// business logic enforcement. Pure domain service focused on schema compliance
// validation. Delegates markdown parsing to MarkdownParserPort (adapter layer)
// to maintain clean hexagonal architecture separation.
//
// Architecture: Domain service orchestrates semantic validation using
// MarkdownParserPort for syntactic parsing and schema validation for business
// rules.
//
// Dependencies:
//   - SchemaEngine: For loading and resolving schemas before validation
//   - MarkdownParserPort: For syntactic frontmatter parsing (YAML structure)
//   - QueryService: For validating FileSpec properties against vault index
//   - Logger: For structured logging of validation operations
type FrontmatterService struct {
	schemaEngine *schema.SchemaEngine
	logger       zerolog.Logger
	eventBus     events.EventBus
	queryService QueryServicePort

	// Validators are stateless and can be reused
	stringValidator *StringValidator
	numberValidator *NumberValidator
	dateValidator   *DateValidator
	boolValidator   *BoolValidator
}

// NewFrontmatterService creates a new FrontmatterService with required
// dependencies. It initializes the service for frontmatter validation
// operations.
//
// Parameters:
//   - schemaEngine: Required for schema loading and resolution
//   - logger: Required for observability and error tracking
//   - eventBus: Required for publishing validation events
//
// - queryService: Required for validating FileSpec properties against vault
// index
//
// Returns a configured FrontmatterService ready for use.
func NewFrontmatterService(
	schemaEngine *schema.SchemaEngine,
	logger zerolog.Logger,
	eventBus events.EventBus,
	queryService QueryServicePort,
) *FrontmatterService {
	return &FrontmatterService{
		schemaEngine:    schemaEngine,
		logger:          logger,
		eventBus:        eventBus,
		queryService:    queryService,
		stringValidator: &StringValidator{},
		numberValidator: &NumberValidator{},
		dateValidator:   &DateValidator{},
		boolValidator:   &BoolValidator{},
	}
}

// Validate validates frontmatter against schema rules and FileSpec properties.
// Performs comprehensive validation including schema compliance and vault index
// validation for file references.
//
// Validation Process:
//  1. Schema validation (required fields, types, constraints)
//  2. FileSpec validation (file references exist in vault)
//  3. Wikilink resolution and ambiguity checking
//  4. Error aggregation with remediation hints
//
// Parameters:
//   - ctx: Context for cancellation and timeout support
//   - noteID: Identifier for the note being validated
//   - fm: Pre-parsed frontmatter to validate
//
// Returns:
//   - error: Aggregated ValidationError instances or nil if validation passes
//
// Error Handling:
//   - Returns ValidationError with field, reason, value, and remediation
//   - Aggregates multiple validation errors
//   - Supports cancellation via context
func (s *FrontmatterService) Validate(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
) error {
	start := time.Now()
	var validationErrors []error

	// Check for cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// Perform schema validation
	if schemaErr := s.IsSchemaCompliant(ctx, noteID, fm); schemaErr != nil {
		validationErrors = append(validationErrors, schemaErr)
	}

	// Perform FileSpec validation if schema available
	if fm.FileClass() != "" {
		if fileSpecErr := s.validateFileSpecProperties(ctx, fm); fileSpecErr != nil {
			validationErrors = append(validationErrors, fileSpecErr)
		}
	}

	validationErr := s.aggregateValidationErrors(validationErrors)
	duration := time.Since(start)
	s.publishValidationEvent(ctx, noteID, fm, validationErr, duration)
	return validationErr
}

// IsSchemaCompliant validates frontmatter against schema rules with semantic
// business logic enforcement. Pure domain service focused on schema compliance
// validation. Frontmatter must be pre-parsed using MarkdownParserPort.
//
// Validation Layer: Domain Layer (Semantic)
// - Performs business rule validation (schema compliance, required fields)
// - Uses pre-parsed frontmatter (syntactic validation done in adapter layer)
// - Does NOT perform parsing or structural validation
//
// Validation Process:
//  1. Validates all required fields are present
//  2. Validates field types using appropriate field validators
//  3. Preserves unknown fields (FR6 compliance)
//  4. Enforces array vs scalar expectations without auto-coercion
//  5. Aggregates all validation errors
//
// Parameters:
//   - ctx: Context for cancellation support
//   - fm: Pre-parsed frontmatter to validate (from MarkdownParserPort)
//
// Returns:
//   - error: Aggregated validation errors or nil if validation passes
//
// Error Handling:
//   - Returns structured FrontmatterError with field context
//   - Aggregates multiple validation errors using errors.Join
//   - Supports cancellation via context
func (s *FrontmatterService) IsSchemaCompliant(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
) error {
	start := time.Now()
	var validationErrors []error

	// Check for cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	// Validate against schema if fileClass is present
	if fm.FileClass() != "" {
		if schemaErr := s.validateAgainstSchema(ctx, fm); schemaErr != nil {
			validationErrors = append(validationErrors, schemaErr)
		}
	}

	validationErr := s.aggregateValidationErrors(validationErrors)
	duration := time.Since(start)
	s.publishValidationEvent(ctx, noteID, fm, validationErr, duration)
	return validationErr
}

// getSchemaForValidation retrieves a schema for frontmatter validation.
// Helper method that wraps schema engine access with error handling.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - fileClass: The schema name to retrieve
//
// Returns:
//   - domain.Schema: The resolved schema for validation
//   - error: Schema retrieval error if schema not found or engine unavailable
func (s *FrontmatterService) getSchemaForValidation(
	ctx context.Context,
	fileClass string,
) (domain.Schema, error) {
	if s.schemaEngine == nil {
		return domain.Schema{}, errors.New("schema engine not available")
	}
	// Use generic Get function from schema package
	return schema.Get[domain.Schema](s.schemaEngine, ctx, fileClass)
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
			if !fm.Has(property.Name) {
				validationErrors = append(
					validationErrors,
					lithosErr.NewFrontmatterError(
						"required field missing",
						property.Name,
						nil,
					),
				)
			}
		}
	}

	if len(validationErrors) > 0 {
		return errors.Join(validationErrors...)
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

	for _, property := range sch.Properties {
		value, exists := fm.Get(property.Name)
		if !exists {
			continue
		}

		validator := s.selectValidator(
			property.Spec.Type(),
			s.stringValidator,
			s.numberValidator,
			s.dateValidator,
			s.boolValidator,
		)
		if validator == nil {
			continue
		}

		if err := s.validatePropertyValue(property, value, validator); err != nil {
			validationErrors = append(validationErrors, err)
		}
	}

	if len(validationErrors) > 0 {
		return errors.Join(validationErrors...)
	}
	return nil
}

// selectValidator returns the appropriate validator for a property type.
func (s *FrontmatterService) selectValidator(
	propType domain.PropertySpecType,
	stringVal, numberVal, dateVal, boolVal FieldValidator,
) FieldValidator {
	switch propType {
	case domain.PropertyTypeString:
		return stringVal
	case domain.PropertyTypeNumber:
		return numberVal
	case domain.PropertyTypeDate:
		return dateVal
	case domain.PropertyTypeBool:
		return boolVal
	default:
		return nil
	}
}

// validatePropertyValue validates a single property value (scalar or array).
func (s *FrontmatterService) validatePropertyValue(
	property domain.Property,
	value any,
	validator FieldValidator,
) error {
	if property.Array {
		return s.validateArrayProperty(
			property.Name,
			value,
			property.Spec,
			validator,
		)
	}
	return s.validateScalarProperty(
		property.Name,
		value,
		property.Spec,
		validator,
	)
}

// validateArrayProperty validates array property values.
func (s *FrontmatterService) validateArrayProperty(
	fieldName string,
	value any,
	spec domain.PropertySpec,
	validator FieldValidator,
) error {
	sliceValues, ok := coerceToInterfaceSlice(value)
	if !ok {
		return lithosErr.NewFrontmatterError(
			"field value is not an array",
			fieldName,
			nil,
		)
	}

	var validationErrors []error
	for _, element := range sliceValues {
		if err := validator.Validate(fieldName, element, spec); err != nil {
			validationErrors = append(validationErrors, err)
		}
	}

	if len(validationErrors) > 0 {
		return errors.Join(validationErrors...)
	}
	return nil
}

// validateScalarProperty validates scalar property values.
func (s *FrontmatterService) validateScalarProperty(
	fieldName string,
	value any,
	spec domain.PropertySpec,
	validator FieldValidator,
) error {
	if isSliceValue(value) {
		return lithosErr.NewFrontmatterError(
			"field value must not be an array",
			fieldName,
			nil,
		)
	}
	return validator.Validate(fieldName, value, spec)
}

// validateAgainstSchema performs comprehensive schema validation for
// frontmatter.
// Validates required fields and field types against the schema specification.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - fm: Frontmatter to validate against schema
//
// Returns:
//   - error: Validation errors or nil if validation passes
func (s *FrontmatterService) validateAgainstSchema(
	ctx context.Context,
	fm domain.Frontmatter,
) error {
	var validationErrors []error

	// Get schema for validation
	sch, schemaErr := s.getSchemaForValidation(ctx, fm.FileClass())
	if schemaErr != nil {
		return schemaErr
	}

	// Validate required fields
	if reqErr := s.validateRequiredFields(fm, sch); reqErr != nil {
		validationErrors = append(validationErrors, reqErr)
	}

	// Validate field types
	if typeErr := s.validateFieldTypes(fm, sch); typeErr != nil {
		validationErrors = append(validationErrors, typeErr)
	}

	// Aggregate validation errors
	return s.aggregateValidationErrors(validationErrors)
}

// validateFileSpecField validates a single FileSpec field, handling both
// single values and arrays.
func (s *FrontmatterService) validateFileSpecField(
	ctx context.Context,
	fieldName string,
	fieldValue interface{},
	isArray bool,
) []error {
	var errs []error

	if !isArray { //nolint:nestif // Early return pattern used to reduce nesting complexity
		if strVal, ok := fieldValue.(string); ok {
			if err := s.validateFileReference(ctx, fieldName, strVal); err != nil {
				errs = append(errs, err)
			}
		}
		return errs
	}

	// Array case
	values, ok := coerceToInterfaceSlice(fieldValue)
	if !ok {
		return errs
	}

	for _, value := range values {
		if strVal, isString := value.(string); isString {
			if err := s.validateFileReference(ctx, fieldName, strVal); err != nil {
				errs = append(errs, err)
			}
		}
	}

	return errs
}

// validateFileSpecProperties validates FileSpec properties against the vault
// index. Checks that file references in frontmatter actually exist in the
// indexed vault.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//   - fm: Frontmatter containing FileSpec properties to validate
//
// Returns:
// - error: ValidationError instances for invalid file references or nil if all
// valid.
func (s *FrontmatterService) validateFileSpecProperties(
	ctx context.Context,
	fm domain.Frontmatter,
) error {
	var validationErrors []error

	// Get schema to identify FileSpec properties
	sch, schemaErr := s.getSchemaForValidation(ctx, fm.FileClass())
	if schemaErr != nil {
		return schemaErr
	}

	// Check each property for FileSpec type
	for _, property := range sch.Properties {
		if property.Spec.Type() != domain.PropertyTypeFile {
			continue
		}

		fieldName := property.Name
		fieldValue, exists := fm.Get(fieldName)
		if !exists {
			continue
		}

		// Validate file references
		fieldErrors := s.validateFileSpecField(
			ctx,
			fieldName,
			fieldValue,
			property.Array,
		)
		validationErrors = append(validationErrors, fieldErrors...)
	}

	return s.aggregateValidationErrors(validationErrors)
}

// validateFileReference validates a single file reference string.
// Supports both direct paths and wikilink format [[basename]].
//
// Parameters:
//   - ctx: Context for cancellation
//   - fieldName: The frontmatter field name for error reporting
//   - value: The file reference string to validate
//
// Returns:
//   - error: ValidationError if file not found or ambiguous, nil if valid
func (s *FrontmatterService) validateFileReference(
	ctx context.Context,
	fieldName string,
	value string,
) error {
	if s.queryService == nil {
		return lithosErr.NewValidationErrorWithRemediation(
			fieldName,
			"query service unavailable",
			value,
			"Ensure QueryService is properly injected into FrontmatterService",
			nil,
		)
	}

	path, isWikilink := s.resolveWikilink(value)
	path = s.normalizePath(path)

	opts := spi.PathQueryOptions{
		Value: path,
		Scope: spi.PathQueryScopeFull,
	}
	if isWikilink {
		opts.Scope = spi.PathQueryScopeBasename
	}

	notes, err := s.queryService.PathQuery(ctx, opts)
	if err != nil {
		return lithosErr.NewValidationErrorWithRemediation(
			fieldName,
			"query failed",
			value,
			"Check QueryService configuration and vault indexing status",
			err,
		)
	}

	if len(notes) == 0 {
		hint := s.generateQueryHint(path, isWikilink)
		return lithosErr.NewValidationErrorWithRemediation(
			fieldName,
			"file not found",
			value,
			hint,
			nil,
		)
	}

	if len(notes) > 1 && isWikilink {
		return s.createAmbiguousError(fieldName, value, path, notes)
	}

	return nil
}

// resolveWikilink detects and resolves wikilink format [[basename]].
// Returns the extracted basename and whether it was a wikilink.
//
// Parameters:
//   - value: The string to check for wikilink format
//
// Returns:
// - string: The resolved path (basename if wikilink, original value otherwise)
//   - bool: Whether the input was a wikilink
func (s *FrontmatterService) resolveWikilink(value string) (string, bool) {
	if strings.HasPrefix(value, "[[") && strings.HasSuffix(value, "]]") &&
		len(value) > 4 {
		basename := value[2 : len(value)-2] // Remove [[ and ]]
		return basename, true
	}
	return value, false
}

// normalizePath normalizes a file path for querying.
// Removes trailing slashes, resolves relative components like ./ and ../.
//
// Parameters:
//   - path: The path to normalize
//
// Returns:
//   - string: The normalized path
func (s *FrontmatterService) normalizePath(path string) string {
	// Use filepath.Clean to resolve ./ and ../ components
	path = filepath.Clean(path)
	// Remove trailing slashes after cleaning
	path = strings.TrimSuffix(path, "/")
	return path
}

// createAmbiguousError creates an error for ambiguous wikilink references.
func (s *FrontmatterService) createAmbiguousError(
	fieldName, value, path string,
	notes []domain.Note,
) error {
	basenames := make([]string, len(notes))
	for i := range notes {
		basenames[i] = notes[i].Path
	}
	hint := fmt.Sprintf(
		"ambiguous wikilink [[%s]]: matches %v",
		path,
		basenames,
	)
	return lithosErr.NewValidationErrorWithRemediation(
		fieldName,
		"ambiguous reference",
		value,
		hint,
		nil,
	)
}

// generateQueryHint generates helpful hints for file not found errors.
// Suggests checking vault or case corrections.
//
// Parameters:
//   - path: The path that was not found
//   - isWikilink: Whether the original reference was a wikilink
//
// Returns:
//   - string: Remediation hint for the user
func (s *FrontmatterService) generateQueryHint(
	path string,
	isWikilink bool,
) string {
	if isWikilink {
		return fmt.Sprintf(
			"Check if [[%s]] exists in vault. If using case-sensitive search, verify exact basename match.",
			path,
		)
	}
	return fmt.Sprintf(
		"Check if '%s' exists in vault. Verify path is relative to vault root and case-sensitive.",
		path,
	)
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
	return errors.Join(validationErrors...)
}

func (s *FrontmatterService) publishValidationEvent(
	ctx context.Context,
	noteID string,
	fm domain.Frontmatter,
	validationErr error,
	duration time.Duration,
) {
	if s.eventBus == nil {
		return
	}
	schemaName := fm.FileClass()
	if strings.TrimSpace(noteID) == "" {
		noteID = "frontmatter/" + schemaName
	}
	if schemaName == "" {
		schemaName = "unknown"
	}
	messages := flattenErrors(validationErr)

	// Publish ValidationPerformedEvent (AC 4.6.2)
	event, err := domain.NewValidationPerformedEvent(
		noteID,
		schemaName,
		validationErr == nil,
		duration,
		messages,
		time.Now(),
	)
	if err != nil {
		s.logger.Error().
			Err(err).
			Msg("failed to create validation performed event")
		return
	}
	if publishErr := s.eventBus.Publish(ctx, event); publishErr != nil {
		s.logger.Warn().
			Err(publishErr).
			Msg("failed to publish validation performed event")
	}

	// Publish ValidationFailedEvent with remediation hints (AC 4.6.10-11)
	if validationErr != nil {
		remediationHints := s.generateRemediationHints(validationErr)
		failedEvent, failErr := domain.NewValidationFailedEvent(
			noteID,
			schemaName,
			messages,
			remediationHints,
			duration,
			time.Now(),
		)
		if failErr != nil {
			s.logger.Error().
				Err(failErr).
				Msg("failed to create validation failed event")
			return
		}
		if publishErr := s.eventBus.Publish(ctx, failedEvent); publishErr != nil {
			s.logger.Warn().
				Err(publishErr).
				Msg("failed to publish validation failed event")
		}
	}
}

func coerceToInterfaceSlice(value any) ([]any, bool) {
	switch v := value.(type) {
	case []any:
		return v, true
	case []string:
		result := make([]any, len(v))
		for i, item := range v {
			result[i] = item
		}
		return result, true
	default:
		return nil, false
	}
}

func isSliceValue(value any) bool {
	switch value.(type) {
	case []any, []string:
		return true
	default:
		return false
	}
}

func flattenErrors(err error) []string {
	if err == nil {
		return nil
	}
	type unwrapper interface{ Unwrap() []error }
	if u, ok := err.(unwrapper); ok {
		var result []string
		for _, inner := range u.Unwrap() {
			result = append(result, flattenErrors(inner)...)
		}
		return result
	}
	return []string{err.Error()}
}

// generateRemediationHints generates helpful remediation hints for validation
// errors.
// This supports AC 4.6.10: ValidationFailedEvent includes remediation hints.
func (s *FrontmatterService) generateRemediationHints(err error) []string {
	if err == nil {
		return nil
	}

	messages := flattenErrors(err)
	hints := make([]string, 0, len(messages))

	for _, msg := range messages {
		switch {
		case strings.Contains(msg, "required field missing"):
			hints = append(
				hints,
				"Add the missing required field to frontmatter",
			)
		case strings.Contains(msg, "file not found"):
			hints = append(
				hints,
				"Verify the file exists in vault or run 'lithos index' to rebuild cache",
			)
		case strings.Contains(msg, "ambiguous reference"):
			hints = append(
				hints,
				"Use full path instead of wikilink to resolve ambiguity",
			)
		case strings.Contains(msg, "field value is not an array"):
			hints = append(
				hints,
				"Change field to array format using YAML list syntax",
			)
		case strings.Contains(msg, "field value must not be an array"):
			hints = append(
				hints,
				"Remove array brackets and use single scalar value",
			)
		case strings.Contains(msg, "query service unavailable"):
			hints = append(
				hints,
				"Ensure QueryService is initialized before validation",
			)
		default:
			hints = append(
				hints,
				"Review schema definition for field constraints",
			)
		}
	}

	return hints
}
