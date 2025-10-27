// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains Data Transfer Objects (DTOs) used for JSON unmarshaling
// and internal data structures for schema and property bank processing.
package schema

// schemaDTO represents the JSON structure for schema files.
type schemaDTO struct {
	Name       string                 `json:"name"`
	Extends    string                 `json:"extends,omitempty"`
	Excludes   []string               `json:"excludes,omitempty"`
	Properties map[string]interface{} `json:"properties"`
}

// propertyDTO represents the JSON structure for full property definitions
// in schema and property bank files. For MVP, $ref properties appear alone
// and are handled separately.
type propertyDTO struct {
	Name     string                 `json:"name,omitempty"`
	Required bool                   `json:"required,omitempty"`
	Array    bool                   `json:"array,omitempty"`
	Type     string                 `json:"type,omitempty"`
	Spec     map[string]interface{} `json:",inline"`
}

// propertyRefDTO represents the JSON structure for $ref-only property
// references.
// In MVP, $ref appears by itself without other property attributes.
type propertyRefDTO struct {
	Ref string `json:"$ref"`
}

// propertyBankDTO represents the JSON structure for property bank files.
type propertyBankDTO struct {
	Properties map[string]interface{} `json:"properties"`
}
