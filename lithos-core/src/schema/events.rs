//! Schema domain events.
#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive Archived types despite \
              #[non_exhaustive] on source types"
)]

use serde::{Deserialize, Serialize};

use super::aggregate::{SchemaId, SchemaName, Timestamp};

/// Property bank updated domain event.
///
/// Published when the property bank is updated, allowing other systems
/// to react to property definition changes.
///
/// # Examples
/// ```
/// use lithos_core::schema::{
///     aggregate::Timestamp, events::PropertyBankUpdated,
/// };
///
/// let event = PropertyBankUpdated::new(12, Timestamp::from_secs(1234567890));
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
    pub timestamp: Timestamp,
}

impl PropertyBankUpdated {
    /// Creates a new property bank updated event.
    #[inline]
    #[must_use]
    pub fn new(property_count: usize, timestamp: Timestamp) -> Self {
        Self {
            property_count,
            timestamp,
        }
    }
}

/// Schema created domain event.
///
/// Published when a new schema is created, allowing other bounded contexts
/// to react to schema definition changes.
///
/// # Examples
/// ```
/// use lithos_core::schema::{
///     aggregate::{SchemaId, SchemaName, Timestamp},
///     events::SchemaCreated,
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let id = SchemaId::new();
/// let name = SchemaName::new("schema")?;
/// let event = SchemaCreated::new(id, &name, Timestamp::from_secs(1234567890));
/// assert_eq!(event.id, id, "Schema id should match");
/// assert_eq!(event.name, name, "Schema name should match");
/// # Ok(())
/// # }
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
    pub id: SchemaId,
    /// Name of the schema.
    pub name: SchemaName,
    /// Unix timestamp when the schema was created.
    pub timestamp: Timestamp,
}

impl SchemaCreated {
    /// Creates a new schema created event.
    #[inline]
    #[must_use]
    pub fn new(id: SchemaId, name: &SchemaName, timestamp: Timestamp) -> Self {
        Self {
            id,
            name: name.clone(),
            timestamp,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_bank_updated_captures_payload() {
        let timestamp = Timestamp::from_secs(123);
        let event = PropertyBankUpdated::new(42, timestamp);

        assert_eq!(event.property_count, 42);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test uses expect for deterministic event setup."
    )]
    fn schema_created_captures_payload() {
        let id = SchemaId::new();
        let name = SchemaName::new("schema").expect("Valid schema name");
        let timestamp = Timestamp::from_secs(456);
        let event = SchemaCreated::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn events_are_send_sync() {
        // GIVEN the schema events enum
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN checking Send + Sync bounds
        is_send_sync::<Events>();

        // THEN it satisfies the bounds
    }
}
