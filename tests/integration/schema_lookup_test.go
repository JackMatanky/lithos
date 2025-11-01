package integration

// Temporarily commented out until packages are implemented
// All imports and test code commented out until app packages are available

// TestSchemaLookup exercises schema-driven template lookups end-to-end
// Temporarily commented out until required packages are implemented
/*
func TestSchemaLookup(t *testing.T) {
	// Set up test environment with fixtures
	ctx := context.Background()
	env := setupTestEnvironment(t)
	defer env.cleanup()

	t.Run("lookup helper finds note by basename", func(t *testing.T) {
		// Load template that uses {{lookup "john-doe"}}
		tmpl, err := env.templateEngine.Load(ctx, "contact-lookup")
		require.NoError(t, err)

		// Render template
		rendered, err := env.templateEngine.Render(ctx, tmpl)
		require.NoError(t, err)

		// Verify rendered output matches golden file
		golden := loadGoldenFile(t, "contact-lookup.md")
		assert.Equal(t, golden, rendered)
	})

	t.Run("query helper finds all notes by fileClass", func(t *testing.T) {
		tmpl, err := env.templateEngine.Load(ctx, "contact-list")
		require.NoError(t, err)

		rendered, err := env.templateEngine.Render(ctx, tmpl)
		require.NoError(t, err)

		// Verify all contacts appear in output
		assert.Contains(t, rendered, "John Doe")
		assert.Contains(t, rendered, "Jane Smith")

		// Verify against golden file
		golden := loadGoldenFile(t, "contact-list.md")
		assert.Equal(t, golden, rendered)
	})

	t.Run("fileClass helper returns note schema", func(t *testing.T) {
		tmpl, err := env.templateEngine.Load(ctx, "conditional-template")
		require.NoError(t, err)

		rendered, err := env.templateEngine.Render(ctx, tmpl)
		require.NoError(t, err)

		// Verify conditional logic based on fileClass worked
		golden := loadGoldenFile(t, "conditional-output.md")
		assert.Equal(t, golden, rendered)
	})
}
*/
