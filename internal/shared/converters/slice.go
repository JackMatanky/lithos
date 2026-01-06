package converters

// ToSlice converts a value to a slice of any.
// It handles []any and []string types.
// Returns the slice and true if successful, nil and false otherwise.
func ToSlice(value any) ([]any, bool) {
	switch v := value.(type) {
	case []any:
		return v, true
	case []string:
		result := make([]any, len(v))
		for i, s := range v {
			result[i] = s
		}
		return result, true
	default:
		return nil, false
	}
}

// IsSlice checks if a value is a slice type supported by ToSlice.
func IsSlice(value any) bool {
	switch value.(type) {
	case []any, []string:
		return true
	default:
		return false
	}
}
