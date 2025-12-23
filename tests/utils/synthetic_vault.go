package utils

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"
)

// Constants for synthetic vault generation.
const (
	// Default counts for large vault.
	defaultContactCount      = 2000
	defaultTaskCount         = 3000
	defaultOrganizationCount = 500
	defaultMeetingCount      = 2500
	defaultNoteCount         = 2000

	// Massive vault multipliers.
	massiveMultiplier = 10

	// Small vault divisors.
	smallDivisor = 40

	// Per-note defaults.
	defaultTagsPerNote  = 3
	defaultLinksPerNote = 5

	// File permissions.
	dirPerm  = 0o755
	filePerm = 0o644

	// Date range for generation (days back).
	dateRangeDays = 365
)

// SyntheticVaultConfig configures synthetic vault generation.
type SyntheticVaultConfig struct {
	ContactCount      int
	TaskCount         int
	OrganizationCount int
	MeetingCount      int
	NoteCount         int
	TagsPerNote       int
	LinksPerNote      int
}

// logger interface for generation functions.
type logger interface {
	Helper()
	Logf(format string, args ...any)
}

// DefaultLargeVaultConfig creates a 10,000+ note vault.
func DefaultLargeVaultConfig() SyntheticVaultConfig {
	return SyntheticVaultConfig{
		ContactCount:      defaultContactCount,
		TaskCount:         defaultTaskCount,
		OrganizationCount: defaultOrganizationCount,
		MeetingCount:      defaultMeetingCount,
		NoteCount:         defaultNoteCount,
		TagsPerNote:       defaultTagsPerNote,
		LinksPerNote:      defaultLinksPerNote,
	}
}

// MassiveVaultConfig creates a 100,000+ note vault.
func MassiveVaultConfig() SyntheticVaultConfig {
	return SyntheticVaultConfig{
		ContactCount:      defaultContactCount * massiveMultiplier,
		TaskCount:         defaultTaskCount * massiveMultiplier,
		OrganizationCount: defaultOrganizationCount * massiveMultiplier,
		MeetingCount:      defaultMeetingCount * massiveMultiplier,
		NoteCount:         defaultNoteCount * massiveMultiplier,
		TagsPerNote:       defaultTagsPerNote,
		LinksPerNote:      defaultLinksPerNote,
	}
}

// GenerateSyntheticVault creates a large test vault with realistic data.
func GenerateSyntheticVault(
	t *testing.T,
	ws *Workspace,
	config SyntheticVaultConfig,
) {
	t.Helper()

	// Create directory structure
	ws.MkdirAll("vault/contacts", dirPerm)
	ws.MkdirAll("vault/tasks", dirPerm)
	ws.MkdirAll("vault/organizations", dirPerm)
	ws.MkdirAll("vault/meetings", dirPerm)
	ws.MkdirAll("vault/notes", dirPerm)
	ws.MkdirAll("schemas", dirPerm)

	// Copy schemas from testdata
	schemaFiles := []string{
		"dir.json",
		"dir_contact.json",
		"task.json",
		"property_bank.json",
	}
	for _, f := range schemaFiles {
		CopyFromTestdata(
			t,
			ws,
			filepath.Join("schemas", f),
			"vault",
			"schemas",
			f,
		)
	}

	// Generate all note types using helper functions
	generateContacts(t, ws, config.ContactCount)
	generateOrganizations(t, ws, config.OrganizationCount)
	generateTasks(t, ws, config.TaskCount)
	generateMeetings(t, ws, config.MeetingCount)
	generateNotes(t, ws, config.NoteCount)

	totalNotes := config.ContactCount + config.TaskCount +
		config.OrganizationCount + config.MeetingCount + config.NoteCount
	t.Logf("Generated synthetic vault with %d total notes", totalNotes)
}

// GenerateTestVault creates a small test vault for quick tests.
func GenerateTestVault(t *testing.T, ws *Workspace) {
	config := SyntheticVaultConfig{
		ContactCount:      defaultContactCount / smallDivisor,
		TaskCount:         defaultTaskCount / smallDivisor,
		OrganizationCount: defaultOrganizationCount / smallDivisor,
		MeetingCount:      defaultMeetingCount / smallDivisor,
		NoteCount:         defaultNoteCount / smallDivisor,
		TagsPerNote:       defaultTagsPerNote,
		LinksPerNote:      defaultLinksPerNote,
	}
	GenerateSyntheticVault(t, ws, config)
}

// GenerateLargeVault creates a large 10k+ note vault.
func GenerateLargeVault(t *testing.T, ws *Workspace) {
	GenerateSyntheticVault(t, ws, DefaultLargeVaultConfig())
}

// GenerateMassiveVault creates a massive 100k+ note vault.
func GenerateMassiveVault(t *testing.T, ws *Workspace) {
	GenerateSyntheticVault(t, ws, MassiveVaultConfig())
}

// GenerateSyntheticVaultBench is a benchmark-compatible version.
func GenerateSyntheticVaultBench(
	b *testing.B,
	ws *Workspace,
	config SyntheticVaultConfig,
) {
	b.Helper()

	// Create directory structure
	ws.MkdirAll("vault/contacts", dirPerm)
	ws.MkdirAll("vault/tasks", dirPerm)
	ws.MkdirAll("vault/organizations", dirPerm)
	ws.MkdirAll("vault/meetings", dirPerm)
	ws.MkdirAll("vault/notes", dirPerm)
	ws.MkdirAll("schemas", dirPerm)

	// Copy schemas
	schemaFiles := []string{
		"dir.json",
		"dir_contact.json",
		"task.json",
		"property_bank.json",
	}
	for _, f := range schemaFiles {
		// For benchmarks, copy from testdata manually
		src := filepath.Join("..", "..", "testdata", "vault", "schemas", f)
		data, err := os.ReadFile(src)
		if err != nil {
			b.Logf("Warning: couldn't copy schema %s: %v", f, err)
			continue
		}
		ws.WriteFile(filepath.Join("schemas", f), data, filePerm)
	}

	// Generate all note types using the same logic
	generateContacts(b, ws, config.ContactCount)
	generateOrganizations(b, ws, config.OrganizationCount)
	generateTasks(b, ws, config.TaskCount)
	generateMeetings(b, ws, config.MeetingCount)
	generateNotes(b, ws, config.NoteCount)

	totalNotes := config.ContactCount + config.TaskCount +
		config.OrganizationCount + config.MeetingCount + config.NoteCount
	b.Logf("Generated synthetic vault with %d total notes", totalNotes)
}

// Helper functions for generation (can be used by both T and B)

func generateContacts(l logger, ws *Workspace, count int) {
	l.Helper()
	l.Logf("Generating %d contacts...", count)

	for i := range count {
		filename := fmt.Sprintf("contact_%05d.md", i)
		uuid := fmt.Sprintf("550e8400-e29b-41d4-a716-%012d", i)

		content := fmt.Sprintf(`---
file_class: dir_contact
uuid: %s
title: Contact %d
name_first: FirstName%d
name_last: LastName%d
email_personal: contact%d@example.com
tags: [contact, professional]
---
# Contact %d
`, uuid, i, i, i, i, i)

		ws.WriteFile(
			filepath.Join("vault", "contacts", filename),
			[]byte(content),
			filePerm,
		)
	}
}

func generateOrganizations(l logger, ws *Workspace, count int) {
	l.Helper()
	l.Logf("Generating %d organizations...", count)

	for i := range count {
		filename := fmt.Sprintf("org_%05d.md", i)
		content := fmt.Sprintf(`---
file_class: organization
title: Organization %d
tags: [organization, tech]
---
# Organization %d
`, i, i)

		ws.WriteFile(
			filepath.Join("vault", "organizations", filename),
			[]byte(content),
			filePerm,
		)
	}
}

func generateTasks(l logger, ws *Workspace, count int) {
	l.Helper()
	l.Logf("Generating %d tasks...", count)

	statuses := []string{"to_do", "in_progress", "done"}
	for i := range count {
		filename := fmt.Sprintf("task_%05d.md", i)
		uuid := fmt.Sprintf("661f8511-e30c-42d5-a817-%012d", i)
		status := statuses[i%len(statuses)]
		startDate := time.Now().
			AddDate(0, 0, -(i % dateRangeDays)).
			Format("2006-01-02")

		content := fmt.Sprintf(`---
file_class: task
uuid: %s
title: Task %d
status: %s
task_start: %s
tags: [task]
---
# Task %d
`, uuid, i, status, startDate, i)

		ws.WriteFile(
			filepath.Join("vault", "tasks", filename),
			[]byte(content),
			filePerm,
		)
	}
}

func generateMeetings(l logger, ws *Workspace, count int) {
	l.Helper()
	l.Logf("Generating %d meetings...", count)

	for i := range count {
		filename := fmt.Sprintf("meeting_%05d.md", i)
		meetingDate := time.Now().
			AddDate(0, 0, -(i % dateRangeDays)).
			Format("2006-01-02")

		content := fmt.Sprintf(`---
file_class: meeting
title: Meeting %d
date: %s
tags: [meeting]
---
# Meeting %d
`, i, meetingDate, i)

		ws.WriteFile(
			filepath.Join("vault", "meetings", filename),
			[]byte(content),
			filePerm,
		)
	}
}

func generateNotes(l logger, ws *Workspace, count int) {
	l.Helper()
	l.Logf("Generating %d general notes...", count)

	categories := []string{"ideas", "research", "project"}
	for i := range count {
		filename := fmt.Sprintf("note_%05d.md", i)
		category := categories[i%len(categories)]
		created := time.Now().
			AddDate(0, 0, -(i % dateRangeDays)).
			Format("2006-01-02")

		content := fmt.Sprintf(`---
file_class: note
title: Note %d
category: %s
created: %s
tags: [note, %s]
---
# Note %d
`, i, category, created, category, i)

		ws.WriteFile(
			filepath.Join("vault", "notes", filename),
			[]byte(content),
			filePerm,
		)
	}
}
