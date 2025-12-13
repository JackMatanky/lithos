package vault

import (
	"testing"

	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
)

// TestNoteValidationService_Creation tests that the service can be created.
func TestNoteValidationService_Creation(t *testing.T) {
	eventBus := utils.NewMockEventBus()
	logger := zerolog.Nop()

	// Test that service can be created (frontmatterService would be injected in
	// real usage)
	service := &NoteValidationService{
		frontmatterService: nil, // Would be injected
		eventBus:           eventBus,
		log:                logger,
	}

	assert.NotNil(t, service)
	assert.Equal(t, eventBus, service.eventBus)
	assert.Equal(t, logger, service.log)
}
