//! Event handler implementations for schema pipeline observability.

use tracing::{debug, error, info, warn};

use super::events::{
    PropertyBankEvent, SchemaEvent, SchemaEventHandler, StalenessReason,
};

// ============================================================================
// Helper Functions
// ============================================================================

#[inline]
fn staleness_reason_str(reason: StalenessReason) -> &'static str {
    match reason {
        StalenessReason::New => "new",
        StalenessReason::Modified => "modified",
        StalenessReason::ContentChanged => "content_changed",
        StalenessReason::BankVersionChanged => "bank_version_changed",
    }
}

#[inline]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Helper function matches on borrowed SchemaEvent variants"
)]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Helper function only handles scan-related events, ignores others"
)]
fn log_scan_events(event: &SchemaEvent) {
    match event {
        SchemaEvent::ScanStarted {
            directory,
        } => {
            info!(directory = %directory.display(), "Schema scan started");
        }
        SchemaEvent::FileDiscovered {
            path,
            name,
        } => {
            debug!(path = %path.display(), schema_name = %name, "Schema file discovered");
        }
        SchemaEvent::ScanCompleted {
            file_count,
        } => {
            info!(file_count = %file_count, "Schema scan completed");
        }
        _ => {}
    }
}

#[inline]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Helper function matches on borrowed SchemaEvent variants"
)]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Helper function only handles staleness events, ignores others"
)]
fn log_staleness_events(event: &SchemaEvent) {
    match event {
        SchemaEvent::SchemaFresh {
            name,
        } => {
            debug!(schema_name = %name, "Schema is fresh");
        }
        SchemaEvent::SchemaStale {
            name,
            reason,
        } => {
            warn!(
                schema_name = %name,
                reason = %staleness_reason_str(*reason),
                "Schema is stale"
            );
        }
        _ => {}
    }
}

#[inline]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Helper function matches on borrowed SchemaEvent variants"
)]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Helper function only handles resolution events, ignores others"
)]
fn log_resolution_events(event: &SchemaEvent) {
    match event {
        SchemaEvent::SchemaResolutionStarted {
            name,
        } => {
            debug!(schema_name = %name, "Schema resolution started");
        }
        SchemaEvent::SchemaResolved {
            name,
            id,
        } => {
            debug!(schema_name = %name, schema_id = %id, "Schema resolved");
        }
        SchemaEvent::SchemaResolutionCompleted {
            schema_count,
        } => {
            info!(schema_count = %schema_count, "Schema resolution completed");
        }
        _ => {}
    }
}

#[inline]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Helper function matches on borrowed SchemaEvent variants"
)]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Helper function only handles persistence events, ignores others"
)]
fn log_persistence_events(event: &SchemaEvent) {
    match event {
        SchemaEvent::RawFileCached {
            name,
        } => {
            debug!(schema_name = %name, "Raw file cached");
        }
        SchemaEvent::SchemaPersisted {
            name,
            id,
        } => {
            debug!(schema_name = %name, schema_id = %id, "Schema persisted");
        }
        _ => {}
    }
}

#[inline]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Helper function matches on borrowed SchemaEvent variants"
)]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Helper function only handles error events, ignores others"
)]
fn log_error_events(event: &SchemaEvent) {
    match event {
        SchemaEvent::ParseError {
            path,
            error,
        } => {
            error!(path = %path.display(), error = %error, "Schema parse error");
        }
        SchemaEvent::ValidationError {
            name,
            error,
        } => {
            error!(schema_name = %name, error = %error, "Schema validation error");
        }
        SchemaEvent::ResolutionError {
            name,
            error,
        } => {
            error!(schema_name = %name, error = %error, "Schema resolution error");
        }
        _ => {}
    }
}

// ============================================================================
// Logging Handler
// ============================================================================

/// Event handler that logs pipeline events using `tracing`.
///
/// Emits structured logs at appropriate levels:
/// - `info`: Major pipeline milestones (scan started/completed, resolution
///   completed)
/// - `debug`: Individual file operations (file discovered, schema resolved)
/// - `warn`: Staleness events
/// - `error`: Error events
///
/// # Examples
/// ```
/// use std::path::PathBuf;
///
/// use lithos_core::schema::{
///     events::{SchemaEvent, SchemaEventHandler},
///     handlers::LoggingHandler,
/// };
///
/// let handler = LoggingHandler::new();
/// let event = SchemaEvent::ScanStarted {
///     directory: PathBuf::from("/vault/schemas"),
/// };
/// handler.handle_schema(&event);
/// ```
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct LoggingHandler;

impl LoggingHandler {
    /// Creates a new logging handler.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::handlers::LoggingHandler;
    ///
    /// let handler = LoggingHandler::new();
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl SchemaEventHandler for LoggingHandler {
    #[inline]
    fn handle_property_bank(&self, event: &PropertyBankEvent) {
        match *event {
            PropertyBankEvent::Fresh {
                version,
            } => {
                debug!(bank_version = %version, "Property bank is fresh");
            }
            PropertyBankEvent::Stale {
                reason,
            } => {
                warn!(reason = %staleness_reason_str(reason), "Property bank is stale");
            }
            PropertyBankEvent::ResolutionStarted => {
                debug!("Property bank resolution started");
            }
            PropertyBankEvent::Resolved {
                property_count,
                version,
            } => {
                info!(
                    property_count = %property_count,
                    bank_version = %version,
                    "Property bank resolved"
                );
            }
            PropertyBankEvent::Persisted {
                version,
            } => {
                debug!(bank_version = %version, "Property bank persisted");
            }
            PropertyBankEvent::TriggeredCascade {
                affected_schema_count,
            } => {
                info!(
                    affected_schema_count = %affected_schema_count,
                    "Property bank change triggered schema cascade"
                );
            }
        }
    }

    #[inline]
    fn handle_schema(&self, event: &SchemaEvent) {
        log_scan_events(event);
        log_staleness_events(event);
        log_resolution_events(event);
        log_persistence_events(event);
        log_error_events(event);
    }
}

// ============================================================================
// Metrics Handler (Stub)
// ============================================================================

/// Event handler that collects metrics for monitoring.
///
/// This is a stub implementation. In production, this would integrate with
/// Prometheus, `StatsD`, or another metrics system.
///
/// # Examples
/// ```
/// use lithos_core::schema::handlers::MetricsHandler;
///
/// let handler = MetricsHandler::new();
/// ```
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct MetricsHandler;

impl MetricsHandler {
    /// Creates a new metrics handler.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::handlers::MetricsHandler;
    ///
    /// let handler = MetricsHandler::new();
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl SchemaEventHandler for MetricsHandler {
    #[inline]
    fn handle_property_bank(&self, _event: &PropertyBankEvent) {
        // Stub: In production, emit metrics to Prometheus/StatsD
    }

    #[inline]
    fn handle_schema(&self, _event: &SchemaEvent) {
        // Stub: In production, emit metrics to Prometheus/StatsD
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::schema::{aggregate::SchemaId, bank::BankVersion};

    #[test]
    fn logging_handler_new() {
        let handler = LoggingHandler::new();
        handler.handle_schema(&SchemaEvent::ScanStarted {
            directory: PathBuf::from("/test"),
        });
    }

    #[test]
    fn logging_handler_handles_all_schema_events() {
        let handler = LoggingHandler::new();

        handler.handle_schema(&SchemaEvent::ScanStarted {
            directory: PathBuf::from("/test"),
        });
        handler.handle_schema(&SchemaEvent::FileDiscovered {
            path: PathBuf::from("/test/task.json"),
            name: "task".into(),
        });
        handler.handle_schema(&SchemaEvent::ScanCompleted {
            file_count: 5,
        });
        handler.handle_schema(&SchemaEvent::SchemaFresh {
            name: "task".into(),
        });
        handler.handle_schema(&SchemaEvent::SchemaStale {
            name: "task".into(),
            reason: StalenessReason::Modified,
        });
        handler.handle_schema(&SchemaEvent::SchemaResolutionStarted {
            name: "task".into(),
        });
        handler.handle_schema(&SchemaEvent::SchemaResolved {
            name: "task".into(),
            id: SchemaId::new(),
        });
        handler.handle_schema(&SchemaEvent::SchemaResolutionCompleted {
            schema_count: 3,
        });
        handler.handle_schema(&SchemaEvent::RawFileCached {
            name: "task".into(),
        });
        handler.handle_schema(&SchemaEvent::SchemaPersisted {
            name: "task".into(),
            id: SchemaId::new(),
        });
        handler.handle_schema(&SchemaEvent::ParseError {
            path: PathBuf::from("/test/bad.json"),
            error: "syntax error".into(),
        });
        handler.handle_schema(&SchemaEvent::ValidationError {
            name: "bad".into(),
            error: crate::schema::error::SchemaError::EmptySchemaName,
        });
        handler.handle_schema(&SchemaEvent::ResolutionError {
            name: "bad".into(),
            error: crate::schema::error::SchemaError::EmptySchemaName,
        });
    }

    #[test]
    fn logging_handler_handles_all_property_bank_events() {
        let handler = LoggingHandler::new();

        handler.handle_property_bank(&PropertyBankEvent::Fresh {
            version: BankVersion::initial(),
        });
        handler.handle_property_bank(&PropertyBankEvent::Stale {
            reason: StalenessReason::Modified,
        });
        handler.handle_property_bank(&PropertyBankEvent::ResolutionStarted);
        handler.handle_property_bank(&PropertyBankEvent::Resolved {
            property_count: 10,
            version: BankVersion::initial(),
        });
        handler.handle_property_bank(&PropertyBankEvent::Persisted {
            version: BankVersion::initial(),
        });
        handler.handle_property_bank(&PropertyBankEvent::TriggeredCascade {
            affected_schema_count: 5,
        });
    }

    #[test]
    fn metrics_handler_new() {
        let handler = MetricsHandler::new();
        handler.handle_schema(&SchemaEvent::ScanStarted {
            directory: PathBuf::from("/test"),
        });
    }

    #[test]
    fn handlers_are_send_sync() {
        fn is_send_sync<T: Send + Sync>() {}

        is_send_sync::<LoggingHandler>();
        is_send_sync::<MetricsHandler>();
    }
}
