package converters

// ToFloat64 converts various numeric types to float64 for consistent
// comparison.
// It handles int, int8, int16, int32, int64, uint, uint8, uint16, uint32,
// uint64, float32, and float64.
// Returns the float64 value and true if successful, 0 and false otherwise.
func ToFloat64(value any) (float64, bool) {
	if v, ok := toIntFloat(value); ok {
		return v, true
	}
	return toUintFloat(value)
}

func toIntFloat(value any) (float64, bool) {
	switch v := value.(type) {
	case int:
		return float64(v), true
	case int8:
		return float64(v), true
	case int16:
		return float64(v), true
	case int32:
		return float64(v), true
	case int64:
		return float64(v), true
	case float32:
		return float64(v), true
	case float64:
		return v, true
	default:
		return 0, false
	}
}

func toUintFloat(value any) (float64, bool) {
	switch v := value.(type) {
	case uint:
		return float64(v), true
	case uint8:
		return float64(v), true
	case uint16:
		return float64(v), true
	case uint32:
		return float64(v), true
	case uint64:
		return float64(v), true
	default:
		return 0, false
	}
}
