package domain

import (
	"context"
	"fmt"
	"regexp"
	"strings"
)

// Property type constants.
const (
	PropertyTypeString PropertySpecType = "string"
	PropertyTypeNumber PropertySpecType = "number"
	PropertyTypeBool   PropertySpecType = "bool"
	PropertyTypeDate   PropertySpecType = "date"
	PropertyTypeFile   PropertySpecType = "file"
)

// PropertySpecType represents the type of property constraint.
type PropertySpecType string

// PropertySpec defines the interface for type-specific validation constraints.
// Each property type implements this interface to provide polymorphic
// validation.
type PropertySpec interface {
	// Type returns the property type identifier.
	Type() PropertySpecType

	// Validate performs structural validation of the constraint definition.
	Validate(ctx context.Context) error
}

// StringSpec defines validation constraints for string properties.
//
// Reference: docs/architecture/data-models.md#stringspec.
type StringSpec struct {
	// Enum contains allowed values as a fixed list (optional).
	Enum []string `json:"enum,omitempty"`

	// Pattern contains a regex pattern for validation (optional).
	Pattern string `json:"pattern,omitempty"`

	// compiledRegex caches the compiled regex pattern to avoid recompilation.
	compiledRegex *regexp.Regexp
}

// NumberSpec defines validation constraints for numeric properties.
//
// Reference: docs/architecture/data-models.md#numberspec.
type NumberSpec struct {
	// Min is the minimum allowed value (inclusive, optional).
	Min *float64 `json:"min,omitempty"`

	// Max is the maximum allowed value (inclusive, optional).
	Max *float64 `json:"max,omitempty"`

	// Step is the increment/decrement amount (optional).
	Step *float64 `json:"step,omitempty"`
}

// BoolSpec defines validation constraints for boolean properties.
// This is a marker type with no additional constraints.
//
// Reference: docs/architecture/data-models.md#boolspec.
type BoolSpec struct{}

// DateSpec defines validation constraints for date properties.
//
// Reference: docs/architecture/data-models.md#datespec.
type DateSpec struct {
	// Format is the Go time layout string (defaults to RFC3339 if empty).
	Format string `json:"format,omitempty"`
}

// FileSpec defines validation constraints for file reference properties.
//
// Reference: docs/architecture/data-models.md#filespec.
type FileSpec struct {
	// FileClass restricts valid file references to notes with specific
	// fileClass (optional).
	FileClass string `json:"file_class,omitempty"`

	// Directory restricts valid file references to notes within specific vault
	// directory (optional).
	Directory string `json:"directory,omitempty"`

	// compiledFileClass caches the compiled regex for FileClass pattern.
	compiledFileClass *regexp.Regexp

	// compiledDirectory caches the compiled regex for Directory pattern.
	compiledDirectory *regexp.Regexp
}

// Type returns PropertyTypeString.
func (s StringSpec) Type() PropertySpecType {
	return PropertyTypeString
}

// Validate checks that Pattern is a valid regex if specified.
// Uses caching to avoid recompiling regex patterns.
//
// Validation Layer: Domain Layer (Semantic)
// - Validates business rules for property specifications
// - Performs regex compilation (infrastructure concern but in domain for
// caching)
// See: docs/architecture/coding-standards.md#validation-layer-separation.
func (s *StringSpec) Validate(ctx context.Context) error {
	if s.Pattern == "" {
		return nil
	}
	if s.compiledRegex == nil {
		regex, err := regexp.Compile(s.Pattern)
		if err != nil {
			return fmt.Errorf("invalid pattern regex: %w", err)
		}
		s.compiledRegex = regex
	}
	return nil
}

// Match checks if the given value matches the Pattern regex.
// Returns true if no pattern is specified or if value matches.
// Automatically compiles the regex if not already cached.
func (s *StringSpec) Match(value string) bool {
	if s.Pattern == "" {
		return true
	}
	if s.compiledRegex == nil {
		// Attempt compilation, but if it fails we return false since
		// structural validation should have caught this earlier.
		regex, err := regexp.Compile(s.Pattern)
		if err != nil {
			return false
		}
		s.compiledRegex = regex
	}
	return s.compiledRegex.MatchString(value)
}

// Type returns PropertyTypeNumber.
func (n NumberSpec) Type() PropertySpecType {
	return PropertyTypeNumber
}

// Validate checks Min <= Max and Step > 0 if specified.
//
// Validation Layer: Domain Layer (Semantic)
// - Validates business rules for number constraints
// See: docs/architecture/coding-standards.md#validation-layer-separation.
func (n NumberSpec) Validate(ctx context.Context) error {
	if err := validateNumberRange(n.Min, n.Max); err != nil {
		return err
	}
	return validateNumberStep(n.Step)
}

// Type returns PropertyTypeBool.
func (b BoolSpec) Type() PropertySpecType {
	return PropertyTypeBool
}

// Validate returns nil (no constraints to validate).
//
// Validation Layer: Domain Layer (Semantic)
// - Bool specs have no additional constraints to validate
// See: docs/architecture/coding-standards.md#validation-layer-separation.
func (b BoolSpec) Validate(ctx context.Context) error {
	return nil
}

// Type returns PropertyTypeDate.
func (d DateSpec) Type() PropertySpecType {
	return PropertyTypeDate
}

// Validate performs basic validation of the DateSpec.
//
// Validation Layer: Domain Layer (Semantic)
// - Validates date format constraints and layout tokens
// See: docs/architecture/coding-standards.md#validation-layer-separation.
func (d DateSpec) Validate(ctx context.Context) error {
	if d.Format == "" {
		// Empty format defaults to RFC3339, which is valid
		return nil
	}
	// Validate that Format contains at least one standard Go time layout token
	validTokens := []string{
		"2006",
		"01",
		"02",
		"15",
		"04",
		"05",
		"MST",
		"Z07:00",
		"PM",
		"Jan",
		"Mon",
	}
	for _, token := range validTokens {
		if strings.Contains(d.Format, token) {
			return nil
		}
	}
	return fmt.Errorf(
		"invalid Go time layout: format must contain at least one standard time token",
	)
}

// Type returns PropertyTypeFile.
func (f FileSpec) Type() PropertySpecType {
	return PropertyTypeFile
}

// Validate checks that FileClass and Directory patterns are valid regexes,
// handling negation prefix. Uses caching to avoid recompilation.
//
// Validation Layer: Domain Layer (Semantic)
// - Validates file reference constraints and patterns
// See: docs/architecture/coding-standards.md#validation-layer-separation.
func (f *FileSpec) Validate(ctx context.Context) error {
	if err := validateFilePatternCached("fileClass", f.FileClass, &f.compiledFileClass); err != nil {
		return err
	}
	return validateFilePatternCached(
		"directory",
		f.Directory,
		&f.compiledDirectory,
	)
}

// validateNumberRange checks that Min <= Max if both are specified.
func validateNumberRange(minVal, maxVal *float64) error {
	if minVal != nil && maxVal != nil && *minVal > *maxVal {
		return fmt.Errorf(
			"min (%f) cannot be greater than max (%f)",
			*minVal,
			*maxVal,
		)
	}
	return nil
}

// validateNumberStep checks that Step > 0 if specified.
func validateNumberStep(step *float64) error {
	if step != nil && *step <= 0 {
		return fmt.Errorf("step must be positive, got %f", *step)
	}
	return nil
}

// validateFilePatternCached checks if the pattern is a valid regex, allowing
// negation prefix (^). Uses caching to avoid recompilation.
func validateFilePatternCached(
	field, value string,
	compiled **regexp.Regexp,
) error {
	if value == "" {
		return nil
	}
	if *compiled == nil {
		pattern := strings.TrimPrefix(value, "^") // Allow negation prefix
		regex, err := regexp.Compile(pattern)
		if err != nil {
			return fmt.Errorf("invalid %s pattern: %w", field, err)
		}
		*compiled = regex
	}
	return nil
}
