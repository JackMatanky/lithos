# Converters Package

The converters package provides type-safe utility functions for common type conversions in Lithos, moving conversion logic out of service layer.

## Overview

Go's `interface{}` type is flexible but requires runtime type assertions. This package centralizes conversion logic, making it:
- **Testable**: Conversion logic in one place
- **Reusable**: Same converters used across services
- **Type-Safe**: Clear function signatures with error handling
- **Performant**: Efficient conversions without reflection

## Available Converters

### number.go

Converts `interface{}` to numeric types with error handling.

#### Functions

```go
func ToFloat64(value interface{}) (float64, error)
func ToInt(value interface{}) (int, error)
func ToInt64(value interface{}) (int64, error)
```

**Supported Types:**
- `float32`, `float64` → Float64
- `int`, `int8`, `int16`, `int32`, `int64` → Int/Int64
- `uint`, `uint8`, `uint16`, `uint32`, `uint64` → Int/Int64
- `string` → ParseFloat/ParseInt
- `bool` → 1 (true) / 0 (false)

**Error Handling:**
- Type mismatch errors for unsupported types
- Parse errors for invalid strings
- Overflow errors for out-of-range values

**Example:**
```go
// From frontmatter value (interface{})
age, err := converters.ToInt(note.Frontmatter["age"])
if err != nil {
    return fmt.Errorf("invalid age: %w", err)
}

// Percentage from string
percent, err := converters.ToFloat64("42.5")
if err != nil {
    return fmt.Errorf("invalid percentage: %w", err)
}
```

### string.go

String manipulation utilities.

#### Functions

```go
func ToString(value interface{}) (string, error)
func ToLower(value string) string
func ToUpper(value string) string
func TrimSpace(value string) string
```

**ToString Supported Types:**
- `string` → Identity
- `int`, `int64`, `float64` → Format with fmt.Sprintf
- `bool` → "true" / "false"
- `[]byte` → String conversion

**Example:**
```go
// Normalize frontmatter field value
value, err := converters.ToString(note.Frontmatter["title"])
if err != nil {
    return fmt.Errorf("invalid title: %w", err)
}

// Normalize for comparison
normalized := converters.ToLower(value)
```

### slice.go

Slice manipulation utilities.

#### Functions

```go
func ToSlice(value interface{}) ([]interface{}, error)
func Contains(slice []string, item string) bool
func Unique(slice []string) []string
```

**ToSlice Supported Types:**
- `[]interface{}` → Identity
- `[]string` → Convert to []interface{}
- Other types → Error

**Example:**
```go
// Tags from frontmatter (interface{})
tags, err := converters.ToSlice(note.Frontmatter["tags"])
if err != nil {
    return fmt.Errorf("tags must be array: %w", err)
}

// Check if tag exists
if converters.Contains(note.Tags, "important") {
    // Handle important notes
}

// Remove duplicates
uniqueTags := converters.Unique(note.Tags)
```

### frontmatter.go

Frontmatter-specific conversions for schema validation.

#### Functions

```go
func ToPropertyValue(value interface{}, prop domain.Property) (interface{}, error)
func ConvertToType(value interface{}, targetType string) (interface{}, error)
```

**ToPropertyValue:**
Converts frontmatter value to schema property type.

**Supported Property Types:**
- `"string"` → String conversion
- `"number"` → Float64 conversion
- `"integer"` → Int64 conversion
- `"boolean"` → Bool conversion
- `"array"` → Slice conversion
- `"object"` → Map conversion

**Example:**
```go
// Validate frontmatter value against property definition
value := note.Frontmatter["priority"]
prop := schema.GetProperty("priority")

validated, err := converters.ToPropertyValue(value, prop)
if err != nil {
    return fmt.Errorf("invalid priority: %w", err)
}

// Use validated value in frontmatter
note.Frontmatter["priority"] = validated
```

## Design Principles

### Why Separate Converters?

**Before:**
```go
// In TemplateService
func (t *TemplateEngine) executeFilteredQuery(...) {
    value := args[0]
    var strValue string
    switch v := value.(type) {
    case string:
        strValue = v
    case int:
        strValue = strconv.Itoa(v)
    default:
        strValue = fmt.Sprintf("%v", v)
    }
    // Use strValue...
}
```

**After:**
```go
// In TemplateService
func (t *TemplateEngine) executeFilteredQuery(...) {
    strValue, err := converters.ToString(value)
    if err != nil {
        return nil, fmt.Errorf("invalid value: %w", err)
    }
    // Use strValue...
}
```

**Benefits:**
- DRY: Conversion logic in one place
- Testable: Unit test conversions independently
- Consistent: Same conversion rules everywhere
- Maintainable: Update logic once

### Why No Reflection?

Converters use type assertions, not reflection:
- **Faster**: Type assertions are compile-time checked
- **Safer**: Compile-time errors for wrong types
- **Simpler**: No reflection boilerplate
- **Idiomatic**: Go's preferred approach

### Error Handling

All conversion functions return errors for:
- Type mismatches
- Invalid formats (e.g., "abc" for number)
- Out-of-range values

**Best Practice:** Always check errors from converters

```go
// Bad: Ignore error
value, _ := converters.ToFloat64(input)

// Good: Handle error
value, err := converters.ToFloat64(input)
if err != nil {
    return fmt.Errorf("conversion failed: %w", err)
}
```

## Performance

Type assertions are very fast (O(1)):
```go
BenchmarkToFloat64_String-12        5000000    280 ns/op    0 B/op    0 allocs/op
BenchmarkToFloat64_Int-12          10000000    120 ns/op    0 B/op    0 allocs/op
BenchmarkToFloat64_Float64-12      20000000     60 ns/op    0 B/op    0 allocs/op
```

## Testing

Unit tests cover:
- All supported types
- Error cases (type mismatch, invalid format)
- Edge cases (nil, empty strings, zero values)

Run tests:
```bash
go test ./internal/shared/converters/...
```

## Usage in Services

### TemplateEngine
Converts template function arguments before query execution:
```go
func (t *TemplateEngine) executeFilteredQuery(field, value interface{}) {
    strValue, err := converters.ToString(value)
    if err != nil {
        return nil, err
    }
    return t.queryService.FrontmatterQuery(field, strValue)
}
```

### FrontmatterService
Validates frontmatter values against schema properties:
```go
func (f *FrontmatterService) validateFileReference(...) {
    value := fileSpec.Default
    prop := schema.GetProperty("path")
    validated, err := converters.ToPropertyValue(value, prop)
    if err != nil {
        return fmt.Errorf("invalid default value: %w", err)
    }
    // Use validated value...
}
```

### VaultIndexer
Converts frontmatter values during indexing:
```go
func (i *VaultIndexer) processFrontmatter(...) {
    tags, err := converters.ToSlice(note.Frontmatter["tags"])
    if err != nil {
        log.Warn().Err(err).Msg("skipping invalid tags")
        return
    }
    // Index tags...
}
```

## Future Enhancements

Potential additions:
- **time.go**: Convert to time.Time with formats
- **duration.go**: Parse duration strings
- **file_path.go**: Path normalization utilities
- **color.go**: Hex/RGB color conversions
