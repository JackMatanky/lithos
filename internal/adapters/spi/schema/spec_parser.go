package schema

import (
	"encoding/json"

	"github.com/JackMatanky/lithos/internal/domain"
)

// propertySpecParsers map property type identifiers to domain spec decoders.
var propertySpecParsers = map[string]func(json.RawMessage) (domain.PropertySpec, error){
	"string": parseStringSpec,
	"number": parseNumberSpec,
	"bool":   parseBoolSpec,
	"date":   parseDateSpec,
	"file":   parseFileSpec,
}

func parseStringSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return &domain.StringSpec{}, nil
	}
	var spec domain.StringSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return &spec, nil
}

func parseNumberSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return domain.NumberSpec{}, nil
	}
	var spec domain.NumberSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func parseBoolSpec(json.RawMessage) (domain.PropertySpec, error) {
	return domain.BoolSpec{}, nil
}

func parseDateSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return domain.DateSpec{}, nil
	}
	var spec domain.DateSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func parseFileSpec(raw json.RawMessage) (domain.PropertySpec, error) {
	if len(raw) == 0 {
		return &domain.FileSpec{}, nil
	}
	var spec domain.FileSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, err
	}
	return &spec, nil
}
