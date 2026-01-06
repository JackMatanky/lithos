package converters

import (
	"fmt"
	"strconv"
)

// ToString coerces various types to their string representation.
// It handles string, fmt.Stringer, int, int64, float64, float32, and bool
// types.
// Returns the string and true if successful, "" and false otherwise.
func ToString(value any) (string, bool) {
	switch v := value.(type) {
	case string:
		return v, true
	case fmt.Stringer:
		return v.String(), true
	case int:
		return strconv.Itoa(v), true
	case int64:
		return strconv.FormatInt(v, 10), true
	case float64:
		return strconv.FormatFloat(v, 'f', -1, 64), true
	case float32:
		return strconv.FormatFloat(float64(v), 'f', -1, 32), true
	case bool:
		return strconv.FormatBool(v), true
	default:
		return "", false
	}
}
