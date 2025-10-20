package domain

import "testing"

func TestStringPropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		spec    StringPropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid string",
			spec:    StringPropertySpec{},
			value:   "test",
			wantErr: false,
		},
		{
			name:    "wrong type",
			spec:    StringPropertySpec{},
			value:   123,
			wantErr: true,
		},
		{
			name: "enum match",
			spec: StringPropertySpec{
				Enum: []string{"red", "green", "blue"},
			},
			value:   "red",
			wantErr: false,
		},
		{
			name: "enum no match",
			spec: StringPropertySpec{
				Enum: []string{"red", "green", "blue"},
			},
			value:   "yellow",
			wantErr: true,
		},
		{
			name: "pattern match",
			spec: StringPropertySpec{
				Pattern: "^[A-Z]+$",
			},
			value:   "ABC",
			wantErr: false,
		},
		{
			name: "pattern no match",
			spec: StringPropertySpec{
				Pattern: "^[A-Z]+$",
			},
			value:   "abc",
			wantErr: true,
		},
		{
			name: "invalid regex",
			spec: StringPropertySpec{
				Pattern: "[invalid",
			},
			value:   "test",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"StringPropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestNumberPropertySpecValidate(t *testing.T) {
	minValue := 0.0
	maxValue := 100.0
	stepValue := 1.0

	tests := []struct {
		name    string
		spec    NumberPropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid number",
			spec:    NumberPropertySpec{},
			value:   42.5,
			wantErr: false,
		},
		{
			name:    "wrong type",
			spec:    NumberPropertySpec{},
			value:   "not a number",
			wantErr: true,
		},
		{
			name: "within bounds",
			spec: NumberPropertySpec{
				Min: &minValue,
				Max: &maxValue,
			},
			value:   50.0,
			wantErr: false,
		},
		{
			name: "below min",
			spec: NumberPropertySpec{
				Min: &minValue,
			},
			value:   -5.0,
			wantErr: true,
		},
		{
			name: "above max",
			spec: NumberPropertySpec{
				Max: &maxValue,
			},
			value:   150.0,
			wantErr: true,
		},
		{
			name: "integer constraint satisfied",
			spec: NumberPropertySpec{
				Step: &stepValue,
			},
			value:   42.0,
			wantErr: false,
		},
		{
			name: "integer constraint violated",
			spec: NumberPropertySpec{
				Step: &stepValue,
			},
			value:   42.5,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"NumberPropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestDatePropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		spec    DatePropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid RFC3339",
			spec:    DatePropertySpec{},
			value:   "2023-10-20T10:30:00Z",
			wantErr: false,
		},
		{
			name: "valid custom format",
			spec: DatePropertySpec{
				Format: "2006-01-02",
			},
			value:   "2023-10-20",
			wantErr: false,
		},
		{
			name: "invalid format",
			spec: DatePropertySpec{
				Format: "2006-01-02",
			},
			value:   "10/20/2023",
			wantErr: true,
		},
		{
			name:    "wrong type",
			spec:    DatePropertySpec{},
			value:   123,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"DatePropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestFilePropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		spec    FilePropertySpec
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid string",
			spec:    FilePropertySpec{},
			value:   "some/path.md",
			wantErr: false,
		},
		{
			name:    "empty string",
			spec:    FilePropertySpec{},
			value:   "",
			wantErr: true,
		},
		{
			name:    "wrong type",
			spec:    FilePropertySpec{},
			value:   123,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"FilePropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestBoolPropertySpecValidate(t *testing.T) {
	tests := []struct {
		name    string
		value   interface{}
		wantErr bool
	}{
		{
			name:    "valid true",
			value:   true,
			wantErr: false,
		},
		{
			name:    "valid false",
			value:   false,
			wantErr: false,
		},
		{
			name:    "wrong type",
			value:   "true",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			spec := BoolPropertySpec{}
			err := spec.Validate(tt.value)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"BoolPropertySpec.Validate() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}
