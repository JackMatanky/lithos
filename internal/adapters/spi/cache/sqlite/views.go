package sqlite

import (
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
)

// GenerateSchemaView returns the SQL statements to create a view and associated
// indexes
// for a given schema.
//
// The view name follows the pattern: v_{schema_name}_notes
// Indexes are created on the base table 'notes' using expression indexes to
// support
// the view's columns efficiently.
func GenerateSchemaView(schema domain.Schema) (string, error) {
	if schema.Name == "" {
		return "", fmt.Errorf("schema name cannot be empty")
	}

	viewName := fmt.Sprintf("v_%s_notes", schema.Name)
	var columns []string
	var indexes []string

	// Standard columns from base table
	columns = append(
		columns,
		"path",
		"frontmatter",
		"modified_at",
		"indexed_time",
		"size",
	)

	// Property columns extracted from JSON
	for _, prop := range schema.Properties {
		colName := prop.Name
		jsonPath := fmt.Sprintf("$.%s", prop.Name)

		// Determine SQL type hint (optional in SQLite but good for
		// clarity/casting if needed)
		// For views, we just extract.
		extractExpr := fmt.Sprintf("json_extract(frontmatter, '%s')", jsonPath)
		columns = append(columns, fmt.Sprintf("%s AS %s", extractExpr, colName))

		// Generate Index for this property
		// We index the expression on the base table with a partial index for
		// this fileClass
		// Index name: idx_{schema}_{prop}
		indexName := fmt.Sprintf("idx_%s_%s", schema.Name, prop.Name)
		indexSQL := fmt.Sprintf(
			"CREATE INDEX IF NOT EXISTS %s ON notes(%s) WHERE json_extract(frontmatter, '$.%s') = '%s';",
			indexName,
			extractExpr,
			"fileClass", // Assuming fileClass key is standard, or we inject Config.FileClassKey?
			// The view filters by fileClass, so the index should too for
			// efficiency. Ideally we use the actual FileClassKey from config,
			// but Schema doesn't know it.
			// We'll assume 'fileClass' for now or we need to pass the key.
			// The story examples use 'fileClass'.
			schema.Name,
		)
		indexes = append(indexes, indexSQL)
	}

	// View definition
	// We also need Config.FileClassKey to filter correctly.
	// Since GenerateSchemaView signature is fixed by AC 4 to (schema
	// domain.Schema),
	// we assume 'fileClass' literal or we need to change signature.
	// AC 6 Example uses: WHERE json_extract(frontmatter, '$.fileClass') =
	// 'contact'; I will stick to 'fileClass' as per example, but in production
	// code we should probably
	// inject the key. For strict AC compliance, I use 'fileClass'.

	selectStmt := fmt.Sprintf(
		"SELECT %s FROM notes WHERE json_extract(frontmatter, '$.fileClass') = '%s'",
		strings.Join(columns, ", "),
		schema.Name,
	)

	viewSQL := fmt.Sprintf(
		"CREATE VIEW IF NOT EXISTS %s AS %s;",
		viewName,
		selectStmt,
	)

	// Combine View + Indexes
	var sb strings.Builder
	sb.WriteString(viewSQL)
	sb.WriteString("\n")
	for _, idx := range indexes {
		sb.WriteString(idx)
		sb.WriteString("\n")
	}

	return sb.String(), nil
}

// Note: To support custom FileClassKey, we might need to update the signature
// or use a global/config context.
// Given the AC doesn't specify config, I'll assume standard "fileClass".
