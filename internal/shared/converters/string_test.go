package converters

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestToString(t *testing.T) {
	tests := []struct {
		name       string
		input      any
		wantValue  string
		wantStatus bool
	}{
		{"string", "hello", "hello", true},
		{"int", 123, "123", true},
		{"float64", 123.45, "123.45", true},
		{"bool true", true, "true", true},
		{"bool false", false, "false", true},
		{"nil", nil, "", false},
		{"slice", []string{"a"}, "", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok := ToString(tt.input)
			assert.Equal(t, tt.wantStatus, ok)
			assert.Equal(t, tt.wantValue, got)
		})
	}
}
