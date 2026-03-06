//! Schema domain events.
#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive Archived types despite \
              #[non_exhaustive] on source types"
)]

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use super::{
    aggregate::{SchemaId, SchemaName},
    bank::BankVersion,
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
}
