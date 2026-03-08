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
//! See [`crate::schema::handlers`] for concrete implementations:
//! - `LoggingHandler` - Tracing integration
//! - `MetricsHandler` - Prometheus/StatsD metrics
//! - `EventCollector` - Test utility for assertions
#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive Archived types despite \
              #[non_exhaustive] on source types"
)]

use std::{path::PathBuf, time::SystemTime};

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use super::{
    aggregate::{SchemaId, SchemaName},
    bank::BankVersion,
    error::SchemaError,
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
///     aggregate::{SchemaId, SchemaName},
///     events::SchemaCreated,
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
    ///     aggregate::{SchemaId, SchemaName},
    ///     events::SchemaCreated,
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
///     aggregate::{SchemaId, SchemaName},
///     events::SchemaResolved,
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
    ///     aggregate::{SchemaId, SchemaName},
    ///     events::SchemaResolved,
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
///     aggregate::{SchemaId, SchemaName},
///     events::SchemaDeleted,
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
    ///     aggregate::{SchemaId, SchemaName},
    ///     events::SchemaDeleted,
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
/// use lithos_core::schema::{bank::BankVersion, events::PropertyBankLoaded};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let event =
///     PropertyBankLoaded::new(3, BankVersion::initial(), SystemTime::now());
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
    /// Version of the property bank.
    pub bank_version: BankVersion,
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
    /// use lithos_core::schema::{bank::BankVersion, events::PropertyBankLoaded};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _event =
    ///     PropertyBankLoaded::new(1, BankVersion::initial(), SystemTime::now());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        property_count: usize,
        bank_version: BankVersion,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            property_count,
            bank_version,
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
///     aggregate::{SchemaId, SchemaName},
///     events::{Events, SchemaCreated},
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
    /// Property bank version changed (cascade).
    BankVersionChanged,
}

/// Pipeline event for property bank loading observability.
///
/// These events track the property bank through the loading pipeline.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertyBankEvent {
    /// Property bank is fresh (no changes).
    Fresh {
        /// Bank version.
        version: BankVersion,
    },

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
        /// Bank version.
        version: BankVersion,
    },

    /// Property bank persisted to database.
    Persisted {
        /// Bank version.
        version: BankVersion,
    },

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
        let version = BankVersion::initial();
        let timestamp = SystemTime::now();
        let event = PropertyBankLoaded::new(42, version, timestamp);

        assert_eq!(event.property_count, 42);
        assert_eq!(event.bank_version, version);
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
        let event = PropertyBankEvent::Fresh {
            version: BankVersion::initial(),
        };
        assert!(matches!(event, PropertyBankEvent::Fresh { .. }));
    }

    #[test]
    fn pipeline_events_are_send_sync() {
        fn is_send_sync<T: Send + Sync>() {}

        is_send_sync::<SchemaEvent>();
        is_send_sync::<PropertyBankEvent>();
        is_send_sync::<StalenessReason>();
    }
}
