package query

import (
	"time"

	"github.com/rs/zerolog"
)

// StalenessObserver provides structured logging for cache staleness events.
// Thread-safe: Safe for concurrent use from multiple query operations.
type StalenessObserver struct {
	log zerolog.Logger
}

// NewStalenessObserver creates an observer for cache staleness events.
func NewStalenessObserver(log zerolog.Logger) *StalenessObserver {
	return &StalenessObserver{log: log}
}

// RecordStaleness logs a cache staleness detection with structured telemetry.
// This enables Ops dashboards to track cache consistency issues.
func (o *StalenessObserver) RecordStaleness(
	path string,
	backend string,
	delta time.Duration,
) {
	o.log.Warn().
		Str("path", path).
		Str("backend", backend).
		Dur("delta", delta).
		Time("detected_at", time.Now()).
		Msg("stale cache entry detected")
}
