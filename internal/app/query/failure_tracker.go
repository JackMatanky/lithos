package query

import (
	"sync"
	"time"

	"github.com/rs/zerolog"
)

// BackendFailureTracker tracks consecutive failures for storage backends.
// This provides basic resilience information without full circuit breaker
// complexity.
type BackendFailureTracker struct {
	mu              sync.RWMutex
	name            string
	failureCount    int
	lastFailureTime time.Time
	log             zerolog.Logger
}

// NewBackendFailureTracker creates a failure tracker for a storage backend.
func NewBackendFailureTracker(
	name string,
	log zerolog.Logger,
) *BackendFailureTracker {
	return &BackendFailureTracker{
		name:            name,
		log:             log,
		failureCount:    0,
		lastFailureTime: time.Time{},
		mu:              sync.RWMutex{},
	}
}

// RecordFailure increments the failure count and logs the event.
func (t *BackendFailureTracker) RecordFailure() {
	t.mu.Lock()
	defer t.mu.Unlock()

	t.failureCount++
	t.lastFailureTime = time.Now()

	t.log.Warn().
		Str("backend", t.name).
		Int("failure_count", t.failureCount).
		Time("last_failure", t.lastFailureTime).
		Msg("backend failure recorded")
}

// RecordSuccess resets the failure count.
func (t *BackendFailureTracker) RecordSuccess() {
	t.mu.Lock()
	defer t.mu.Unlock()

	if t.failureCount > 0 {
		t.failureCount = 0
	}
}

// GetFailureCount returns the current consecutive failure count.
func (t *BackendFailureTracker) GetFailureCount() int {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.failureCount
}

// GetLastFailureTime returns when the last failure occurred.
func (t *BackendFailureTracker) GetLastFailureTime() time.Time {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.lastFailureTime
}
