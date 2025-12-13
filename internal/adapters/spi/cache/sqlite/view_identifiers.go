package sqlite

import (
	"fmt"
	"strings"
	"unicode"
)

func schemaViewName(schemaName string) string {
	return fmt.Sprintf("v_%s_notes", sanitizeIdentifier(schemaName))
}

func schemaIndexPrefix(schemaName string) string {
	return fmt.Sprintf("idx_%s_", sanitizeIdentifier(schemaName))
}

func schemaIndexName(schemaName, propertyName string) string {
	return fmt.Sprintf(
		"%s%s",
		schemaIndexPrefix(schemaName),
		sanitizeIdentifier(propertyName),
	)
}

func sanitizeIdentifier(name string) string {
	if name == "" {
		return ""
	}

	var b strings.Builder
	for _, r := range name {
		if unicode.IsLetter(r) || unicode.IsDigit(r) || r == '_' {
			b.WriteRune(r)
			continue
		}
		b.WriteRune('_')
	}
	return b.String()
}
