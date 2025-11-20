package sqlite

import (
	"strings"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

func TestGenerateSchemaView(t *testing.T) {
	tests := []struct {
		name      string
		schema    domain.Schema
		wantView  string
		wantIndex []string
		wantErr   bool
	}{
		{
			name: "Valid schema with properties",
			schema: domain.Schema{
				Name: "contact",
				Properties: []domain.Property{
					{Name: "name", Spec: &domain.StringSpec{}},
					{Name: "age", Spec: &domain.NumberSpec{}},
				},
			},
			wantView: "CREATE VIEW IF NOT EXISTS v_contact_notes AS " +
				"SELECT path, frontmatter, modified_at, indexed_time, size, " +
				"json_extract(frontmatter, '$.name') AS name, " +
				"json_extract(frontmatter, '$.age') AS age " +
				"FROM notes WHERE json_extract(frontmatter, '$.fileClass') = 'contact';",
			wantIndex: []string{
				"CREATE INDEX IF NOT EXISTS idx_contact_name ON notes(json_extract(frontmatter, '$.name')) " +
					"WHERE json_extract(frontmatter, '$.fileClass') = 'contact';",
				"CREATE INDEX IF NOT EXISTS idx_contact_age ON notes(json_extract(frontmatter, '$.age')) " +
					"WHERE json_extract(frontmatter, '$.fileClass') = 'contact';",
			},
			wantErr: false,
		},
		{
			name:    "Empty schema name",
			schema:  domain.Schema{Name: ""},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := GenerateSchemaView(tt.schema)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"GenerateSchemaView() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
				return
			}
			if !tt.wantErr {
				if !strings.Contains(got, tt.wantView) {
					t.Errorf(
						"GenerateSchemaView() view SQL mismatch.\nGot:\n%s\nWant substring:\n%s",
						got,
						tt.wantView,
					)
				}
				for _, idx := range tt.wantIndex {
					if !strings.Contains(got, idx) {
						t.Errorf(
							"GenerateSchemaView() missing index SQL.\nGot:\n%s\nWant substring:\n%s",
							got,
							idx,
						)
					}
				}
			}
		})
	}
}
