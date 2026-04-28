//! Schema domain events and pipeline events.
//!
//! ## Event-Driven Architecture
//!
//! This module contains two types of events:
//!
//! ### Domain Events
//!
//! Aggregate-level events emitted by the PropertyBank:
//! - `SchemaCreated` - New schema registered
//! - `SchemaResolved` - Schema resolved through inheritance pipeline
//! - `PropertyRegistered` - New property added to bank
//! - `PropertyBankLoaded` - Bank loaded/reloaded
//!
//! ### Pipeline Events (Observability)
//!
//! Fine-grained events emitted by [`crate::schema::loader`] for observability
//! and reactive coordination:
//!
//! - **`SchemaEvent`**: File scan, staleness checks, resolution, persistence
//! - **`PropertyBankEvent`**: Bank lifecycle, staleness, cascade triggers
//!
//! ## Event Handlers
//!
//! Implement [`SchemaEventHandler`] to receive pipeline events.
//!
//! Provided implementations:
//! - [`LoggingHandler`] - Tracing integration
//! - [`MetricsHandler`] - Prometheus/StatsD metrics (stub)
//! - [`EventCollector`] - Test utility for assertions
#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive Archived types despite \
              #[non_exhaustive] on source types"
)]

use std::{path::PathBuf, time::SystemTime};

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};
use tracing::{debug, error, info, warn};

use super::{
    error::SchemaError,
    identifier::{SchemaId, SchemaName},
    property::{PropertyId, PropertyName},
};

/// Schema created domain event.
///
/// Published when a new schema is created for the first time (new ID assigned).
///
/// # Examples
/// ```
/// use std::time::SystemTime;
///
/// use lithos_core::schema::{
///     events::SchemaCreated,
///     identifier::{SchemaId, SchemaName},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let id = SchemaId::new();
/// let name = SchemaName::try_new("schema")?;
/// let event = SchemaCreated::new(id, &name, SystemTime::now());
/// assert_eq!(event.id, id, "Schema id should match");
/// assert_eq!(event.name, name, "Schema name should match");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaCreated {
    /// UUID of the schema.
    pub id: SchemaId,
    /// Name of the schema.
    pub name: SchemaName,
    /// Timestamp when the schema was created.
    #[rkyv(with = AsUnixTime)]
    pub timestamp: SystemTime,
}

impl SchemaCreated {
    /// Creates a new schema created event.
    ///
    /// # Examples
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use lithos_core::schema::{
    ///     events::SchemaCreated,
    ///     identifier::{SchemaId, SchemaName},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = SchemaId::new();
    /// let name = SchemaName::try_new("schema")?;
    /// let _event = SchemaCreated::new(id, &name, SystemTime::now());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(id: SchemaId, name: &SchemaName, timestamp: SystemTime) -> Self {
        Self {
            id,
            name: name.clone(),
            timestamp,
        }
    }
}

/// Schema resolved domain event.
///
/// Published every time a schema is resolved (both new and existing schemas).
///
/// # Examples
/// ```
/// use std::time::SystemTime;
///
/// use lithos_core::schema::{
///     events::SchemaResolved,
///     identifier::{SchemaId, SchemaName},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = SchemaId::new();
/// let name = SchemaName::try_new("schema")?;
/// let event = SchemaResolved::new(id, &name, SystemTime::now());
/// assert_eq!(event.id, id);
/// assert_eq!(event.name, name);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaResolved {
    /// UUID of the schema.
    pub id: SchemaId,
    /// Name of the schema.
    pub name: SchemaName,
    /// Timestamp when the schema was resolved.
    #[rkyv(with = AsUnixTime)]
    pub timestamp: SystemTime,
}

impl SchemaResolved {
    /// Creates a new schema resolved event.
    ///
    /// # Examples
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use lithos_core::schema::{
    ///     events::SchemaResolved,
    ///     identifier::{SchemaId, SchemaName},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = SchemaId::new();
    /// let name = SchemaName::try_new("schema")?;
    /// let _event = SchemaResolved::new(id, &name, SystemTime::now());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(id: SchemaId, name: &SchemaName, timestamp: SystemTime) -> Self {
        Self {
            id,
            name: name.clone(),
            timestamp,
        }
    }
}

/// Schema deleted domain event.
///
/// Published when a schema is removed from the vault.
///
/// # Examples
/// ```
/// use std::time::SystemTime;
///
/// use lithos_core::schema::{
///     events::SchemaDeleted,
///     identifier::{SchemaId, SchemaName},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = SchemaId::new();
/// let name = SchemaName::try_new("schema")?;
/// let event = SchemaDeleted::new(id, &name, SystemTime::now());
/// assert_eq!(event.id, id);
/// assert_eq!(event.name, name);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaDeleted {
    /// UUID of the deleted schema.
    pub id: SchemaId,
    /// Name of the deleted schema.
    pub name: SchemaName,
    /// Timestamp when the schema was deleted.
    #[rkyv(with = AsUnixTime)]
    pub timestamp: SystemTime,
}

impl SchemaDeleted {
    /// Creates a new schema deleted event.
    ///
    /// # Examples
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use lithos_core::schema::{
    ///     events::SchemaDeleted,
    ///     identifier::{SchemaId, SchemaName},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = SchemaId::new();
    /// let name = SchemaName::try_new("schema")?;
    /// let _event = SchemaDeleted::new(id, &name, SystemTime::now());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(id: SchemaId, name: &SchemaName, timestamp: SystemTime) -> Self {
        Self {
            id,
            name: name.clone(),
            timestamp,
        }
    }
}

/// Property registered domain event.
///
/// Published when a single new property is added to the property bank.
///
/// # Examples
/// ```
/// use std::time::SystemTime;
///
/// use lithos_core::schema::{
///     events::PropertyRegistered,
///     property::{PropertyId, PropertyName},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = PropertyId::new();
/// let name = PropertyName::try_new("flag")?;
/// let event = PropertyRegistered::new(id, &name, SystemTime::now());
/// assert_eq!(event.id, id);
/// assert_eq!(event.name, name);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct PropertyRegistered {
    /// UUID of the property.
    pub id: PropertyId,
    /// Name of the property.
    pub name: PropertyName,
    /// Timestamp when the property was registered.
    #[rkyv(with = AsUnixTime)]
    pub timestamp: SystemTime,
}

impl PropertyRegistered {
    /// Creates a new property registered event.
    ///
    /// # Examples
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use lithos_core::schema::{
    ///     events::PropertyRegistered,
    ///     property::{PropertyId, PropertyName},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = PropertyId::new();
    /// let name = PropertyName::try_new("flag")?;
    /// let _event = PropertyRegistered::new(id, &name, SystemTime::now());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        id: PropertyId,
        name: &PropertyName,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            id,
            name: name.clone(),
            timestamp,
        }
    }
}

/// Property bank loaded domain event.
///
/// Published when the property bank is loaded or reloaded from vault data.
///
/// # Examples
/// ```
/// use std::time::SystemTime;
///
/// use lithos_core::schema::events::PropertyBankLoaded;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let event = PropertyBankLoaded::new(3, SystemTime::now());
/// assert_eq!(event.property_count, 3);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct PropertyBankLoaded {
    /// Number of properties in the bank after loading.
    pub property_count: usize,
    /// Timestamp when the bank was loaded.
    #[rkyv(with = AsUnixTime)]
    pub timestamp: SystemTime,
}

impl PropertyBankLoaded {
    /// Creates a new property bank loaded event.
    ///
    /// # Examples
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use lithos_core::schema::events::PropertyBankLoaded;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _event = PropertyBankLoaded::new(1, SystemTime::now());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(property_count: usize, timestamp: SystemTime) -> Self {
        Self {
            property_count,
            timestamp,
        }
    }
}

/// Domain events for the Schema context.
///
/// # Examples
/// ```
/// use std::time::SystemTime;
///
/// use lithos_core::schema::{
///     events::{Events, SchemaCreated},
///     identifier::{SchemaId, SchemaName},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = SchemaId::new();
/// let name = SchemaName::try_new("schema")?;
/// let created = SchemaCreated::new(id, &name, SystemTime::now());
/// let event = Events::SchemaCreated(created);
/// match event {
///     Events::SchemaCreated(_) => {}
///     _ => {}
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum Events {
    /// Schema was created for the first time.
    SchemaCreated(SchemaCreated),
    /// Schema was resolved (happens on every resolution pass).
    SchemaResolved(SchemaResolved),
    /// Schema was deleted from the vault.
    SchemaDeleted(SchemaDeleted),
    /// A single property was registered in the bank.
    PropertyRegistered(PropertyRegistered),
    /// Property bank was loaded or reloaded.
    PropertyBankLoaded(PropertyBankLoaded),
}

// ============================================================================
// Pipeline Events (Phase 3)
// ============================================================================

/// Pipeline event for schema loading observability.
///
/// These events track the lifecycle of schema files through the loading
/// pipeline: scanning → staleness check → resolution → persistence.
///
/// Unlike domain events (`SchemaCreated`, etc.), pipeline events are emitted
/// during the orchestration process for observability and reactive
/// coordination.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaEvent {
    // --- Scan Phase ---
    /// Schema directory scan started.
    ScanStarted {
        /// Path to the schemas directory being scanned.
        directory: PathBuf,
    },

    /// Schema file discovered during scan.
    FileDiscovered {
        /// Path to the discovered schema file.
        path: PathBuf,
        /// Schema name derived from filename.
        name: Box<str>,
    },

    /// Schema directory scan completed.
    ScanCompleted {
        /// Number of schema files found.
        file_count: usize,
    },

    // --- Staleness Phase ---
    /// Schema file is fresh (no changes since last load).
    SchemaFresh {
        /// Schema name.
        name: Box<str>,
    },

    /// Schema file is stale (changed since last load or new).
    SchemaStale {
        /// Schema name.
        name: Box<str>,
        /// Reason for staleness.
        reason: StalenessReason,
    },

    // --- Resolution Phase ---
    /// Schema resolution started.
    SchemaResolutionStarted {
        /// Schema name being resolved.
        name: Box<str>,
    },

    /// Schema resolved successfully.
    SchemaResolved {
        /// Schema name.
        name: Box<str>,
        /// Schema ID after resolution.
        id: SchemaId,
    },

    /// All schemas resolved.
    SchemaResolutionCompleted {
        /// Number of schemas resolved.
        schema_count: usize,
    },

    // --- Persistence Phase ---
    /// Raw schema file cached to database.
    RawFileCached {
        /// Schema name.
        name: Box<str>,
    },

    /// Resolved schema persisted to database.
    SchemaPersisted {
        /// Schema name.
        name: Box<str>,
        /// Schema ID.
        id: SchemaId,
    },

    // --- Error Events ---
    /// Parse error during schema ingestion.
    ParseError {
        /// Path to the file that failed to parse.
        path: PathBuf,
        /// Error details.
        error: Box<str>,
    },

    /// Validation error during schema ingestion.
    ValidationError {
        /// Schema name that failed validation.
        name: Box<str>,
        /// Validation error details.
        error: SchemaError,
    },

    /// Resolution error during schema processing.
    ResolutionError {
        /// Schema name that failed resolution.
        name: Box<str>,
        /// Resolution error details.
        error: SchemaError,
    },
}

/// Reason why a schema is marked as stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StalenessReason {
    /// Schema is new (never loaded before).
    New,
    /// Schema file modified timestamp changed.
    Modified,
    /// Schema file content hash changed.
    ContentChanged,
    /// Property bank changed (cascade).
    PropertyBankChanged,
}

/// Pipeline event for property bank loading observability.
///
/// These events track the property bank through the loading pipeline.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertyBankEvent {
    /// Property bank is fresh (no changes).
    Fresh,

    /// Property bank is stale (changed).
    Stale {
        /// Reason for staleness.
        reason: StalenessReason,
    },

    /// Property bank resolution started.
    ResolutionStarted,

    /// Property bank resolved successfully.
    Resolved {
        /// Number of properties in the bank.
        property_count: usize,
    },

    /// Property bank persisted to database.
    Persisted,

    /// Property bank change triggered schema cascade.
    TriggeredCascade {
        /// Number of schemas marked stale.
        affected_schema_count: usize,
    },
}

// ============================================================================
// Event Handlers (Phase 3)
// ============================================================================

/// Handler for schema pipeline events.
///
/// Implementations can perform logging, metrics collection, or reactive
/// coordination based on pipeline events.
///
/// # Examples
/// ```
/// use lithos_core::schema::events::{
///     PropertyBankEvent, SchemaEvent, SchemaEventHandler,
/// };
///
/// struct MyHandler;
///
/// impl SchemaEventHandler for MyHandler {
///     fn handle_property_bank(&self, event: &PropertyBankEvent) {
///         // Handle property bank events
///     }
///
///     fn handle_schema(&self, event: &SchemaEvent) {
///         // Handle schema events
///     }
/// }
/// ```
pub trait SchemaEventHandler: Send + Sync {
    /// Handle a property bank pipeline event.
    fn handle_property_bank(&self, event: &PropertyBankEvent);

    /// Handle a schema pipeline event.
    fn handle_schema(&self, event: &SchemaEvent);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_created_captures_payload() {
        let id = SchemaId::new();
        let name = SchemaName::try_new("schema").expect("Valid schema name");
        let timestamp = SystemTime::now();
        let event = SchemaCreated::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn schema_resolved_captures_payload() {
        let id = SchemaId::new();
        let name = SchemaName::try_new("schema").expect("Valid schema name");
        let timestamp = SystemTime::now();
        let event = SchemaResolved::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn schema_deleted_captures_payload() {
        let id = SchemaId::new();
        let name = SchemaName::try_new("schema").expect("Valid schema name");
        let timestamp = SystemTime::now();
        let event = SchemaDeleted::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn property_registered_captures_payload() {
        let id = PropertyId::new();
        let name =
            PropertyName::try_new("status").expect("Valid property name");
        let timestamp = SystemTime::now();
        let event = PropertyRegistered::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn property_bank_loaded_captures_payload() {
        let timestamp = SystemTime::now();
        let event = PropertyBankLoaded::new(42, timestamp);

        assert_eq!(event.property_count, 42);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn events_are_send_sync() {
        fn is_send_sync<T: Send + Sync>() {}

        is_send_sync::<Events>();
    }

    // Pipeline event tests
    #[test]
    fn schema_event_scan_started() {
        let event = SchemaEvent::ScanStarted {
            directory: PathBuf::from("/vault/schemas"),
        };
        assert!(matches!(event, SchemaEvent::ScanStarted { .. }));
    }

    #[test]
    fn schema_event_file_discovered() {
        let event = SchemaEvent::FileDiscovered {
            path: PathBuf::from("/vault/schemas/task.json"),
            name: "task".into(),
        };
        assert!(matches!(event, SchemaEvent::FileDiscovered { .. }));
    }

    #[test]
    fn schema_event_stale_with_reason() {
        let event = SchemaEvent::SchemaStale {
            name: "task".into(),
            reason: StalenessReason::Modified,
        };
        assert!(
            matches!(event, SchemaEvent::SchemaStale {
                reason: StalenessReason::Modified,
                ..
            }),
            "Expected SchemaStale event with Modified reason"
        );
    }

    #[test]
    fn property_bank_event_fresh() {
        let event = PropertyBankEvent::Fresh;
        assert!(matches!(event, PropertyBankEvent::Fresh));
    }

    #[test]
    fn pipeline_events_are_send_sync() {
        fn is_send_sync<T: Send + Sync>() {}

        is_send_sync::<SchemaEvent>();
        is_send_sync::<PropertyBankEvent>();
        is_send_sync::<StalenessReason>();
    }
}

// ============================================================================
// Event Handler Implementations
// ============================================================================

// ============================================================================
// Helper Functions
// ============================================================================

#[inline]
fn staleness_reason_str(reason: StalenessReason) -> &'static str {
    match reason {
        StalenessReason::New => "new",
        StalenessReason::Modified => "modified",
        StalenessReason::ContentChanged => "content_changed",
        StalenessReason::PropertyBankChanged => "property_bank_changed",
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
/// use lithos_core::schema::events::{
///     LoggingHandler, SchemaEvent, SchemaEventHandler,
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
    /// use lithos_core::schema::events::LoggingHandler;
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
            PropertyBankEvent::Fresh => {
                debug!("Property bank is fresh");
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
            } => {
                info!(
                    property_count = %property_count,
                    "Property bank resolved"
                );
            }
            PropertyBankEvent::Persisted => {
                debug!("Property bank persisted");
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
/// use lithos_core::schema::events::MetricsHandler;
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
    /// use lithos_core::schema::events::MetricsHandler;
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

// ============================================================================
// Test Utilities
// ============================================================================

/// Test utility for recording events emitted during pipeline execution.
///
/// This handler collects all events in memory for testing and assertions.
///
/// # Examples
/// ```
/// # use lithos_core::schema::events::{EventCollector, SchemaEvent, SchemaEventHandler};
/// # use std::path::PathBuf;
/// let collector = EventCollector::new();
/// collector.handle_schema(&SchemaEvent::ScanStarted {
///     directory: PathBuf::from("/vault/schemas"),
/// });
///
/// let events = collector.schema_events();
/// assert_eq!(events.len(), 1);
/// ```
#[derive(Debug, Default)]
pub struct EventCollector {
    schema_events: std::sync::Mutex<Vec<SchemaEvent>>,
    property_bank_events: std::sync::Mutex<Vec<PropertyBankEvent>>,
}

impl EventCollector {
    /// Create a new event collector.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all collected schema events.
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned (rare, only if a panic
    /// occurred while holding the lock).
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Mutex poisoning is exceptional - intentional panic on \
                  poisoned lock"
    )]
    pub fn schema_events(&self) -> Vec<SchemaEvent> {
        self.schema_events.lock().expect("Lock poisoned").clone()
    }

    /// Get all collected property bank events.
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned (rare, only if a panic
    /// occurred while holding the lock).
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Mutex poisoning is exceptional - intentional panic on \
                  poisoned lock"
    )]
    pub fn property_bank_events(&self) -> Vec<PropertyBankEvent> {
        self.property_bank_events.lock().expect("Lock poisoned").clone()
    }

    /// Clear all collected events.
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned (rare, only if a panic
    /// occurred while holding the lock).
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Mutex poisoning is exceptional - intentional panic on \
                  poisoned lock"
    )]
    pub fn clear(&self) {
        self.schema_events.lock().expect("Lock poisoned").clear();
        self.property_bank_events.lock().expect("Lock poisoned").clear();
    }

    /// Get the count of schema events.
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned (rare, only if a panic
    /// occurred while holding the lock).
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Mutex poisoning is exceptional - intentional panic on \
                  poisoned lock"
    )]
    pub fn schema_event_count(&self) -> usize {
        self.schema_events.lock().expect("Lock poisoned").len()
    }

    /// Get the count of property bank events.
    ///
    /// # Panics
    /// Panics if the internal mutex is poisoned (rare, only if a panic
    /// occurred while holding the lock).
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Mutex poisoning is exceptional - intentional panic on \
                  poisoned lock"
    )]
    pub fn property_bank_event_count(&self) -> usize {
        self.property_bank_events.lock().expect("Lock poisoned").len()
    }
}

impl SchemaEventHandler for EventCollector {
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Mutex poisoning is exceptional - intentional panic on \
                  poisoned lock"
    )]
    fn handle_property_bank(&self, event: &PropertyBankEvent) {
        self.property_bank_events
            .lock()
            .expect("Lock poisoned")
            .push(event.clone());
    }

    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Mutex poisoning is exceptional - intentional panic on \
                  poisoned lock"
    )]
    fn handle_schema(&self, event: &SchemaEvent) {
        self.schema_events.lock().expect("Lock poisoned").push(event.clone());
    }
}

#[cfg(test)]
mod handler_tests {
    use super::*;
    use crate::schema::identifier::SchemaId;

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
            error: crate::schema::error::SchemaError::Syntax(
                crate::schema::error::SchemaSyntaxError::SchemaName(
                    crate::schema::error::SchemaNameError::Empty,
                ),
            ),
        });
        handler.handle_schema(&SchemaEvent::ResolutionError {
            name: "bad".into(),
            error: crate::schema::error::SchemaError::Syntax(
                crate::schema::error::SchemaSyntaxError::SchemaName(
                    crate::schema::error::SchemaNameError::Empty,
                ),
            ),
        });
    }

    #[test]
    fn logging_handler_handles_all_property_bank_events() {
        let handler = LoggingHandler::new();

        handler.handle_property_bank(&PropertyBankEvent::Fresh);
        handler.handle_property_bank(&PropertyBankEvent::Stale {
            reason: StalenessReason::Modified,
        });
        handler.handle_property_bank(&PropertyBankEvent::ResolutionStarted);
        handler.handle_property_bank(&PropertyBankEvent::Resolved {
            property_count: 10,
        });
        handler.handle_property_bank(&PropertyBankEvent::Persisted);
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
