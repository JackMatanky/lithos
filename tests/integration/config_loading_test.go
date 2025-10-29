package integration

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/config"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestConfigLoading_Integration tests end-to-end config loading from testdata.
func TestConfigLoading_Integration(t *testing.T) {
	// Change to testdata/vault directory where lithos.json config file is
	// located
	projectRoot := findProjectRoot(t)
	configDir := filepath.Join(projectRoot, "testdata", "vault")
	originalWd, _ := os.Getwd()
	require.NoError(t, os.Chdir(configDir))
	defer func() {
		_ = os.Chdir(originalWd)
	}()

	// Create logger
	log := logger.New(os.Stdout, "debug")

	// Create adapter
	adapter := config.NewViperAdapter(log)

	// Load configuration
	ctx := context.Background()
	cfg, err := adapter.Load(ctx)

	// Assert successful loading
	require.NoError(t, err)
	require.NotNil(t, cfg)

	// Verify config values from testdata/vault/lithos.json
	assert.Equal(t, "/tmp/test-vault", cfg.VaultPath)
	assert.Equal(t, "testdata/templates", cfg.TemplatesDir)
	assert.Equal(t, "testdata/schema", cfg.SchemasDir)
	assert.Equal(
		t,
		"testdata/schema/properties/bank.json",
		cfg.PropertyBankFile,
	)
	assert.Equal(t, ".cache", cfg.CacheDir)
	assert.Equal(t, "info", cfg.LogLevel)
}

// findProjectRoot finds the project root by looking for go.mod.
func findProjectRoot(t *testing.T) string {
	dir, err := os.Getwd()
	require.NoError(t, err)

	for {
		if _, statErr := os.Stat(filepath.Join(dir, "go.mod")); statErr == nil {
			return dir
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			t.Fatal("Could not find project root (go.mod)")
		}
		dir = parent
	}
}
