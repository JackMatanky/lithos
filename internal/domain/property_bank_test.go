package domain

import (
	"fmt"
	"sync"
	"testing"
)

func TestNewPropertyBank(t *testing.T) {
	tests := []struct {
		name         string
		location     string
		wantLocation string
	}{
		{
			name:         "empty location uses default",
			location:     "",
			wantLocation: "schemas/properties/",
		},
		{
			name:         "custom location",
			location:     "custom/path/",
			wantLocation: "custom/path/",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := NewPropertyBank(tt.location)
			if got.Location != tt.wantLocation {
				t.Errorf(
					"NewPropertyBank() location = %v, want %v",
					got.Location,
					tt.wantLocation,
				)
			}
			if len(got.Properties) != 0 {
				t.Errorf(
					"NewPropertyBank() properties should be empty, got %v",
					got.Properties,
				)
			}
		})
	}
}

func TestPropertyBank_RegisterProperty(t *testing.T) {
	pb := NewPropertyBank("")

	stringProp := NewProperty("title", true, false, StringPropertySpec{
		Enum: []string{"Mr", "Mrs", "Dr"},
	})

	tests := []struct {
		name      string
		regName   string
		property  Property
		wantError bool
	}{
		{
			name:      "valid registration",
			regName:   "another_title",
			property:  stringProp,
			wantError: false,
		},
		{
			name:      "empty name",
			regName:   "",
			property:  stringProp,
			wantError: true,
		},
		{
			name:      "whitespace name",
			regName:   "  test  ",
			property:  stringProp,
			wantError: true,
		},
		{
			name:      "duplicate name",
			regName:   "standard_title",
			property:  stringProp,
			wantError: true,
		},
	}

	// Register first property
	err := pb.RegisterProperty("standard_title", stringProp)
	if err != nil {
		t.Fatalf("Failed to register initial property: %v", err)
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			regErr := pb.RegisterProperty(tt.regName, tt.property)
			if (regErr != nil) != tt.wantError {
				t.Errorf(
					"RegisterProperty() error = %v, wantError %v",
					regErr,
					tt.wantError,
				)
			}
		})
	}
}

func TestPropertyBank_GetProperty(t *testing.T) {
	pb := NewPropertyBank("")

	stringProp := NewProperty("title", true, false, StringPropertySpec{
		Enum: []string{"Mr", "Mrs", "Dr"},
	})

	if err := pb.RegisterProperty("standard_title", stringProp); err != nil {
		t.Fatalf("Failed to register property: %v", err)
	}

	tests := []struct {
		name   string
		key    string
		exists bool
	}{
		{
			name:   "existing property",
			key:    "standard_title",
			exists: true,
		},
		{
			name:   "non-existing property",
			key:    "nonexistent",
			exists: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, exists := pb.GetProperty(tt.key)
			if exists != tt.exists {
				t.Errorf(
					"GetProperty() exists = %v, want %v",
					exists,
					tt.exists,
				)
			}
			if tt.exists && got.Name != "title" {
				t.Errorf("GetProperty() name = %v, want title", got.Name)
			}
		})
	}
}

func TestPropertyBank_HasProperty(t *testing.T) {
	pb := NewPropertyBank("")

	stringProp := NewProperty("title", true, false, StringPropertySpec{})

	if err := pb.RegisterProperty("standard_title", stringProp); err != nil {
		t.Fatalf("Failed to register property: %v", err)
	}

	tests := []struct {
		name   string
		key    string
		exists bool
	}{
		{
			name:   "existing property",
			key:    "standard_title",
			exists: true,
		},
		{
			name:   "non-existing property",
			key:    "nonexistent",
			exists: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := pb.HasProperty(tt.key)
			if got != tt.exists {
				t.Errorf("HasProperty() = %v, want %v", got, tt.exists)
			}
		})
	}
}

func TestPropertyBank_Validate(t *testing.T) {
	tests := []struct {
		name      string
		setup     func() *PropertyBank
		wantError bool
	}{
		{
			name: "valid property bank",
			setup: func() *PropertyBank {
				pb := NewPropertyBank("schemas/properties/")
				stringProp := NewProperty(
					"title",
					true,
					false,
					StringPropertySpec{},
				)
				if err := pb.RegisterProperty("standard_title", stringProp); err != nil {
					t.Fatalf("Failed to register property: %v", err)
				}
				return &pb
			},
			wantError: false,
		},
		{
			name: "empty location",
			setup: func() *PropertyBank {
				pb := NewPropertyBank("")
				pb.Location = "" // Force empty
				return &pb
			},
			wantError: true,
		},
		{
			name: "invalid property",
			setup: func() *PropertyBank {
				pb := NewPropertyBank("schemas/properties/")
				invalidProp := Property{
					Name:     "",
					Required: true,
				} // Invalid: empty name
				pb.Properties["invalid"] = invalidProp // Bypass registration validation
				return &pb
			},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			pb := tt.setup()
			err := pb.Validate()
			if (err != nil) != tt.wantError {
				t.Errorf(
					"Validate() error = %v, wantError %v",
					err,
					tt.wantError,
				)
			}
		})
	}
}

func TestPropertyBank_RegisterPropertyConcurrent(t *testing.T) {
	pb := NewPropertyBank("")
	baseProp := NewProperty("title", false, false, StringPropertySpec{})

	const total = 20
	var wg sync.WaitGroup

	for i := range total {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			name := fmt.Sprintf("prop_%d", i)
			if err := pb.RegisterProperty(name, baseProp); err != nil {
				t.Errorf("RegisterProperty(%s) error = %v", name, err)
			}
		}(i)
	}

	wg.Wait()

	if got := len(pb.Properties); got != total {
		t.Errorf("len(Properties) = %d, want %d", got, total)
	}
}
