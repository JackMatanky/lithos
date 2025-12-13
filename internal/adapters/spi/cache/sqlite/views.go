package sqlite

import (
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
)

const defaultFileClassKey = "fileClass"

// ViewGenerationOptions controls customization of schema view output.
type ViewGenerationOptions struct {
	FileClassKey string
}

func (o ViewGenerationOptions) normalized() ViewGenerationOptions {
	if o.FileClassKey == "" {
		o.FileClassKey = defaultFileClassKey
	}
	return o
}

// GenerateSchemaView returns the SQL statements to create a view and associated
// indexes for a given schema using default options.
func GenerateSchemaView(schema domain.Schema) (string, error) {
	return GenerateSchemaViewWithOptions(schema, ViewGenerationOptions{
		FileClassKey: defaultFileClassKey,
	})
}

// GenerateSchemaViewWithOptions returns the SQL statements to create a view and
// associated indexes for a given schema with custom options.
func GenerateSchemaViewWithOptions(
	schema domain.Schema,
	opts ViewGenerationOptions,
) (string, error) {
	opts = opts.normalized()

	if schema.Name == "" {
		return "", fmt.Errorf("schema name cannot be empty")
	}

	viewName := schemaViewName(schema.Name)
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

		extractExpr := fmt.Sprintf("json_extract(frontmatter, '%s')", jsonPath)
		typedExpr := applySQLType(extractExpr, prop)
		columns = append(columns, fmt.Sprintf("%s AS %s", typedExpr, colName))

		// Generate Index for this property
		// Index name: idx_{schema}_{prop}
		indexName := schemaIndexName(schema.Name, prop.Name)
		indexSQL := fmt.Sprintf(
			"CREATE INDEX IF NOT EXISTS %s ON notes(%s) WHERE json_extract(frontmatter, '$.%s') = '%s';",
			indexName,
			extractExpr,
			opts.FileClassKey,
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
		"SELECT %s FROM notes WHERE json_extract(frontmatter, '$.%s') = '%s'",
		strings.Join(columns, ", "),
		opts.FileClassKey,
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

func applySQLType(expr string, prop domain.Property) string {
	switch prop.Spec.(type) {
	case *domain.NumberSpec:
		return fmt.Sprintf("CAST(%s AS REAL)", expr)
	case *domain.BoolSpec:
		return fmt.Sprintf("CAST(%s AS INTEGER)", expr)
	case *domain.StringSpec, *domain.DateSpec, *domain.FileSpec:
		return fmt.Sprintf("CAST(%s AS TEXT)", expr)
	default:
		if prop.Spec != nil && prop.Spec.Type() == domain.PropertyTypeNumber {
			return fmt.Sprintf("CAST(%s AS REAL)", expr)
		}
		return expr
	}
}
