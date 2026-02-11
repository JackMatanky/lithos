//! Schema domain events.
#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive Archived types despite \
              #[non_exhaustive] on source types"
)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Property bank updated domain event.
///
/// Published when the property bank is updated, allowing other systems
/// to react to property definition changes.
///
/// # Examples
/// ```
/// use lithos_core::schema::events::PropertyBankUpdated;
///
/// let event = PropertyBankUpdated::new(12, 1234567890);
/// assert_eq!(event.property_count, 12, "Property count should match");
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct PropertyBankUpdated {
    /// Number of properties in the bank after update.
    pub property_count: usize,
    /// Unix timestamp when the update occurred.
    pub timestamp: i64,
}

/// Schema created domain event.
///
/// Published when a new schema is created, allowing other bounded contexts
/// to react to schema definition changes.
///
/// # Examples
/// ```
/// use lithos_core::schema::events::SchemaCreated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = SchemaCreated::new(id, "schema", 1234567890);
/// assert_eq!(event.id, id, "Schema id should match");
/// assert_eq!(event.name, "schema", "Schema name should match");
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaCreated {
    /// UUID of the schema.
    pub id: Uuid,
    /// Name of the schema.
    pub name: String,
    /// Unix timestamp when the schema was created.
    pub timestamp: i64,
}

/// Domain events for the Schema context.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum Events {
    /// Property bank was updated.
    PropertyBankUpdated(PropertyBankUpdated),
    /// Schema was created.
    SchemaCreated(SchemaCreated),
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

impl SchemaCreated {
    /// Creates a new schema created event.
    #[inline]
    #[must_use]
    pub fn new(id: Uuid, name: &str, timestamp: i64) -> Self {
        Self {
            id,
            name: name.into(),
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_send_sync() {
        // GIVEN the schema events enum
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN checking Send + Sync bounds
        is_send_sync::<Events>();

        // THEN it satisfies the bounds
    }
}
