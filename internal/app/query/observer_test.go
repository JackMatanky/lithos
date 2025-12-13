package query

import (
	"testing"
	"time"

	"github.com/rs/zerolog"
)

func TestStalenessObserver_RecordStaleness(t *testing.T) {
	log := zerolog.Nop()
	observer := NewStalenessObserver(log)

	// This should not panic - we're just testing the interface
	observer.RecordStaleness("test.md", "boltdb", 5*time.Second)
}

func TestBackendFailureTracker_RecordFailure(t *testing.T) {
	log := zerolog.Nop()
	tracker := NewBackendFailureTracker("test", log)

	// Initial state
	if tracker.GetFailureCount() != 0 {
		t.Errorf(
			"Expected initial failure count 0, got %d",
			tracker.GetFailureCount(),
		)
	}

	// Record failures
	tracker.RecordFailure()
	tracker.RecordFailure()

	if tracker.GetFailureCount() != 2 {
		t.Errorf("Expected failure count 2, got %d", tracker.GetFailureCount())
	}

	// Record success should reset
	tracker.RecordSuccess()

	if tracker.GetFailureCount() != 0 {
		t.Errorf(
			"Expected failure count 0 after success, got %d",
			tracker.GetFailureCount(),
		)
	}
}

func TestBackendFailureTracker_LastFailureTime(t *testing.T) {
	log := zerolog.Nop()
	tracker := NewBackendFailureTracker("test", log)

	before := time.Now()
	time.Sleep(1 * time.Millisecond) // Ensure time difference

	tracker.RecordFailure()

	after := time.Now()
	lastFailure := tracker.GetLastFailureTime()

	if lastFailure.Before(before) || lastFailure.After(after) {
		t.Errorf("Last failure time %v not within expected range [%v, %v]",
			lastFailure, before, after)
	}
}
