//! Schema domain events.
#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive Archived types despite \
              #[non_exhaustive] on source types"
)]

use serde::{Deserialize, Serialize};

use super::{
    aggregate::{SchemaId, SchemaName, Timestamp},
    bank::BankVersion,
    property::{PropertyId, PropertyName},
};

/// Schema created domain event.
///
/// Published when a new schema is created for the first time (new ID assigned).
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
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::{SchemaId, SchemaName, Timestamp},
    ///     events::SchemaCreated,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = SchemaId::new();
    /// let name = SchemaName::new("schema")?;
    /// let _event = SchemaCreated::new(id, &name, Timestamp::from_secs(456));
    /// # Ok(())
    /// # }
    /// ```
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

/// Schema resolved domain event.
///
/// Published every time a schema is resolved (both new and existing schemas).
///
/// # Examples
/// ```
/// use lithos_core::schema::{
///     aggregate::{SchemaId, SchemaName, Timestamp},
///     events::SchemaResolved,
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = SchemaId::new();
/// let name = SchemaName::new("schema")?;
/// let event = SchemaResolved::new(id, &name, Timestamp::from_secs(123));
/// assert_eq!(event.id, id);
/// assert_eq!(event.name, name);
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
pub struct SchemaResolved {
    /// UUID of the schema.
    pub id: SchemaId,
    /// Name of the schema.
    pub name: SchemaName,
    /// Unix timestamp when the schema was resolved.
    pub timestamp: Timestamp,
}

impl SchemaResolved {
    /// Creates a new schema resolved event.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::{SchemaId, SchemaName, Timestamp},
    ///     events::SchemaResolved,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = SchemaId::new();
    /// let name = SchemaName::new("schema")?;
    /// let _event = SchemaResolved::new(id, &name, Timestamp::from_secs(456));
    /// # Ok(())
    /// # }
    /// ```
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

/// Schema deleted domain event.
///
/// Published when a schema is removed from the vault.
///
/// # Examples
/// ```
/// use lithos_core::schema::{
///     aggregate::{SchemaId, SchemaName, Timestamp},
///     events::SchemaDeleted,
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = SchemaId::new();
/// let name = SchemaName::new("schema")?;
/// let event = SchemaDeleted::new(id, &name, Timestamp::from_secs(789));
/// assert_eq!(event.id, id);
/// assert_eq!(event.name, name);
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
pub struct SchemaDeleted {
    /// UUID of the deleted schema.
    pub id: SchemaId,
    /// Name of the deleted schema.
    pub name: SchemaName,
    /// Unix timestamp when the schema was deleted.
    pub timestamp: Timestamp,
}

impl SchemaDeleted {
    /// Creates a new schema deleted event.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::{SchemaId, SchemaName, Timestamp},
    ///     events::SchemaDeleted,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = SchemaId::new();
    /// let name = SchemaName::new("schema")?;
    /// let _event = SchemaDeleted::new(id, &name, Timestamp::from_secs(321));
    /// # Ok(())
    /// # }
    /// ```
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

/// Property registered domain event.
///
/// Published when a single new property is added to the property bank.
///
/// # Examples
/// ```
/// use lithos_core::schema::{
///     aggregate::Timestamp,
///     events::PropertyRegistered,
///     property::{PropertyId, PropertyName},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = PropertyId::new();
/// let name = PropertyName::new("flag")?;
/// let event = PropertyRegistered::new(id, &name, Timestamp::from_secs(42));
/// assert_eq!(event.id, id);
/// assert_eq!(event.name, name);
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
pub struct PropertyRegistered {
    /// UUID of the property.
    pub id: PropertyId,
    /// Name of the property.
    pub name: PropertyName,
    /// Unix timestamp when the property was registered.
    pub timestamp: Timestamp,
}

impl PropertyRegistered {
    /// Creates a new property registered event.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::Timestamp,
    ///     events::PropertyRegistered,
    ///     property::{PropertyId, PropertyName},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let id = PropertyId::new();
    /// let name = PropertyName::new("flag")?;
    /// let _event = PropertyRegistered::new(id, &name, Timestamp::from_secs(42));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        id: PropertyId,
        name: &PropertyName,
        timestamp: Timestamp,
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
/// use lithos_core::schema::{
///     aggregate::Timestamp, bank::BankVersion, events::PropertyBankLoaded,
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let event = PropertyBankLoaded::new(
///     3,
///     BankVersion::initial(),
///     Timestamp::from_secs(900),
/// );
/// assert_eq!(event.property_count, 3);
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
pub struct PropertyBankLoaded {
    /// Number of properties in the bank after loading.
    pub property_count: usize,
    /// Version of the property bank.
    pub bank_version: BankVersion,
    /// Unix timestamp when the bank was loaded.
    pub timestamp: Timestamp,
}

impl PropertyBankLoaded {
    /// Creates a new property bank loaded event.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::Timestamp, bank::BankVersion, events::PropertyBankLoaded,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _event = PropertyBankLoaded::new(
    ///     1,
    ///     BankVersion::initial(),
    ///     Timestamp::from_secs(900),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        property_count: usize,
        bank_version: BankVersion,
        timestamp: Timestamp,
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
/// use lithos_core::schema::{
///     aggregate::{SchemaId, SchemaName, Timestamp},
///     events::{Events, SchemaCreated},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = SchemaId::new();
/// let name = SchemaName::new("schema")?;
/// let created = SchemaCreated::new(id, &name, Timestamp::from_secs(1));
/// let event = Events::SchemaCreated(created);
/// match event {
///     Events::SchemaCreated(_) => {}
///     _ => {}
/// }
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
    fn schema_resolved_captures_payload() {
        let id = SchemaId::new();
        let name = SchemaName::new("schema").expect("Valid schema name");
        let timestamp = Timestamp::from_secs(456);
        let event = SchemaResolved::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn schema_deleted_captures_payload() {
        let id = SchemaId::new();
        let name = SchemaName::new("schema").expect("Valid schema name");
        let timestamp = Timestamp::from_secs(456);
        let event = SchemaDeleted::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn property_registered_captures_payload() {
        let id = PropertyId::new();
        let name = PropertyName::new("status").expect("Valid property name");
        let timestamp = Timestamp::from_secs(789);
        let event = PropertyRegistered::new(id, &name, timestamp);

        assert_eq!(event.id, id);
        assert_eq!(event.name, name);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn property_bank_loaded_captures_payload() {
        let version = BankVersion::initial();
        let timestamp = Timestamp::from_secs(123);
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
}
