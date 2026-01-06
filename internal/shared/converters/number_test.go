package converters

import (
	"math"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestToFloat64(t *testing.T) {
	tests := []struct {
		name       string
		input      any
		wantValue  float64
		wantStatus bool
	}{
		{"int", 123, 123.0, true},
		{"int8", int8(123), 123.0, true},
		{"int16", int16(123), 123.0, true},
		{"int32", int32(123), 123.0, true},
		{"int64", int64(123), 123.0, true},
		{"uint", uint(123), 123.0, true},
		{"uint8", uint8(123), 123.0, true},
		{"uint16", uint16(123), 123.0, true},
		{"uint32", uint32(123), 123.0, true},
		{"uint64", uint64(123), 123.0, true},
		{"float32", float32(123.45), float64(float32(123.45)), true},
		{"float64", 123.45, 123.45, true},
		{"string", "123", 0, false},
		{"bool", true, 0, false},
		{"nil", nil, 0, false},
		{"infinity", math.Inf(1), math.Inf(1), true},
		{"NaN", math.NaN(), math.NaN(), true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok := ToFloat64(tt.input)
			assert.Equal(t, tt.wantStatus, ok)

			switch {
			case math.IsNaN(tt.wantValue):
				assert.True(t, math.IsNaN(got))
			case math.IsInf(tt.wantValue, 0):
				assert.True(t, math.IsInf(got, 0))
				assert.Equal(t, math.Inf(1) > 0, tt.wantValue > 0) // Check sign
			case tt.wantValue == 0:
				assert.InDelta(t, 0, got, 0)
			default:
				assert.InEpsilon(t, tt.wantValue, got, 1e-9)
			}
		})
	}
}
