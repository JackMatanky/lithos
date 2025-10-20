package property

import (
	"strings"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

func TestPropertySerializer_MarshalJSON(t *testing.T) {
	serializer := NewPropertySerializer()

	tests := []struct {
		name     string
		property domain.Property
		check    func(t *testing.T, data []byte)
	}{
		{
			name: "string property with enum",
			property: domain.NewProperty(
				"status",
				true,
				false,
				domain.StringPropertySpec{
					Enum: []string{"active", "inactive"},
				},
			),
			check: func(t *testing.T, data []byte) {
				str := string(data)
				if !strings.Contains(str, `"name":"status"`) ||
					!strings.Contains(str, `"required":true`) ||
					!strings.Contains(str, `"array":false`) ||
					!strings.Contains(str, `"type":"string"`) ||
					!strings.Contains(str, `"enum":["active","inactive"]`) {
					t.Errorf("MarshalJSON() missing expected fields: %s", str)
				}
			},
		},
		{
			name: "number property with constraints",
			property: domain.NewProperty(
				"age",
				false,
				false,
				domain.NumberPropertySpec{
					Min:  func() *float64 { v := 0.0; return &v }(),
					Max:  func() *float64 { v := 120.0; return &v }(),
					Step: func() *float64 { v := 1.0; return &v }(),
				},
			),
			check: func(t *testing.T, data []byte) {
				str := string(data)
				if !strings.Contains(str, `"name":"age"`) || !strings.Contains(str, `"required":false`) ||
					!strings.Contains(str, `"array":false`) || !strings.Contains(str, `"type":"number"`) ||
					!strings.Contains(str, `"min":0`) ||
					!strings.Contains(str, `"max":120`) ||
					!strings.Contains(str, `"step":1`) {
					t.Errorf("MarshalJSON() missing expected fields: %s", str)
				}
			},
		},
		{
			name: "bool property",
			property: domain.NewProperty(
				"enabled",
				true,
				false,
				domain.BoolPropertySpec{},
			),
			check: func(t *testing.T, data []byte) {
				str := string(data)
				if !strings.Contains(str, `"name":"enabled"`) ||
					!strings.Contains(str, `"required":true`) ||
					!strings.Contains(str, `"array":false`) ||
					!strings.Contains(str, `"type":"bool"`) {
					t.Errorf("MarshalJSON() missing expected fields: %s", str)
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			data, err := serializer.MarshalJSON(tt.property)
			if err != nil {
				t.Errorf("MarshalJSON() error = %v", err)
				return
			}
			tt.check(t, data)
		})
	}
}

func TestPropertySerializer_UnmarshalJSON(t *testing.T) {
	serializer := NewPropertySerializer()

	tests := []struct {
		name    string
		json    string
		want    domain.Property
		wantErr bool
	}{
		{
			name: "string property with enum",
			json: `{"name":"status","required":true,"array":false,"type":"string","enum":["active","inactive"]}`,
			want: domain.NewProperty(
				"status",
				true,
				false,
				domain.StringPropertySpec{
					Enum: []string{"active", "inactive"},
				},
			),
			wantErr: false,
		},
		{
			name: "number property with constraints",
			json: `{"name":"age","required":false,"array":false,"type":"number","min":0,"max":120,"step":1}`,
			want: func() domain.Property {
				min := 0.0
				max := 120.0
				step := 1.0
				return domain.NewProperty(
					"age",
					false,
					false,
					domain.NumberPropertySpec{
						Min:  &min,
						Max:  &max,
						Step: &step,
					},
				)
			}(),
			wantErr: false,
		},
		{
			name: "bool property",
			json: `{"name":"enabled","required":true,"array":false,"type":"bool"}`,
			want: domain.NewProperty(
				"enabled",
				true,
				false,
				domain.BoolPropertySpec{},
			),
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := serializer.UnmarshalJSON([]byte(tt.json))
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"UnmarshalJSON() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
				return
			}
			if !tt.wantErr {
				// Compare fields since we can't directly compare Property
				// structs
				if got.Name != tt.want.Name ||
					got.Required != tt.want.Required ||
					got.Array != tt.want.Array {
					t.Errorf(
						"UnmarshalJSON() got Name=%s Required=%v Array=%v, want Name=%s Required=%v Array=%v",
						got.Name,
						got.Required,
						got.Array,
						tt.want.Name,
						tt.want.Required,
						tt.want.Array,
					)
				}
				// Compare type names
				gotType, _ := got.TypeName()
				wantType, _ := tt.want.TypeName()
				if gotType != wantType {
					t.Errorf(
						"UnmarshalJSON() type = %s, want %s",
						gotType,
						wantType,
					)
				}
			}
		})
	}
}
