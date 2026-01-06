package converters

// Canonicalize normalizes frontmatter values for type-agnostic comparison.
// Handles numeric type conversions (int 2 == float 2.0) and safe comparison
// for scalar types (string, bool).
// Returns the normalized value and true if successful, nil and false otherwise.
func Canonicalize(value any) (any, bool) {
	if v, ok := ToFloat64(value); ok {
		return v, true
	}
	switch v := value.(type) {
	case string, bool:
		return v, true
	default:
		return nil, false
	}
}
