//! Schema domain events.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Schema created domain event.
///
/// Published when a new schema is created, allowing other bounded contexts
/// to react to schema definition changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaCreated {
    /// UUID of the schema.
    pub id: Uuid,
    /// Name of the schema.
    pub name: String,
    /// Unix timestamp when the schema was created.
    pub timestamp: i64,
}

impl SchemaCreated {
    /// Creates a new schema created event.
    #[inline]
    #[must_use]
    pub fn new(id: Uuid, name: String, timestamp: i64) -> Self {
        Self {
            id,
            name,
            timestamp,
        }
    }
}

/// Property bank updated domain event.
///
/// Published when the property bank is updated, allowing other systems
/// to react to property definition changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyBankUpdated {
    /// Number of properties in the bank after update.
    pub property_count: usize,
    /// Unix timestamp when the update occurred.
    pub timestamp: i64,
}

impl PropertyBankUpdated {
    /// Creates a new property bank updated event.
    #[inline]
    #[must_use]
    pub fn new(property_count: usize, timestamp: i64) -> Self {
        Self {
            property_count,
            timestamp,
        }
    }
}
