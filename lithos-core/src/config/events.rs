//! Configuration domain events.

#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived structs and enums"
)]

use serde::{Deserialize, Serialize};

/// Configuration updated domain event.
///
/// This event is published when configuration changes occur, allowing
/// other bounded contexts to react to configuration updates.
///
/// # Examples
/// ```
/// use lithos_core::config::events::ConfigUpdated;
///
/// let event = ConfigUpdated::new("vault".to_string(), 1234567890);
/// assert_eq!(event.timestamp, 1234567890, "Timestamp should match input");
/// assert_eq!(event.source, "vault", "Source should match input");
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
pub struct ConfigUpdated {
    /// Source of the configuration change (e.g., "global", "vault").
    pub source: String,
    /// Unix timestamp when the configuration was updated.
    pub timestamp: i64,
}

/// Domain events that can be emitted by the Config aggregate.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum Events {
    /// Configuration was updated.
    ConfigUpdated(ConfigUpdated),
}

impl ConfigUpdated {
    /// Creates a new configuration updated event.
    #[inline]
    #[must_use]
    pub fn new(source: String, timestamp: i64) -> Self {
        Self {
            source,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture uses expect for deterministic setup. Failure \
                  indicates invalid test data. Expect is idiomatic in setup."
    )]
    fn deserialized_event() -> ConfigUpdated {
        let json = r#"{"source":"vault","timestamp":1234567890}"#;
        serde_json::from_str(json)
            .expect("ConfigUpdated should deserialize successfully")
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture uses expect for deterministic setup. Failure \
                  indicates invalid test data. Expect is idiomatic in setup."
    )]
    fn serialized_event() -> String {
        let event = ConfigUpdated {
            source: "vault".to_owned(),
            timestamp: 1_234_567_890,
        };
        serde_json::to_string(&event)
            .expect("ConfigUpdated should serialize successfully")
    }

    #[test]
    fn config_updated_event_deserializes_timestamp() {
        let event = deserialized_event();

        assert_eq!(
            event.timestamp, 1_234_567_890,
            "Expected deserialized timestamp to match"
        );
    }

    #[test]
    fn config_updated_event_deserializes_source() {
        let event = deserialized_event();

        assert_eq!(event.source, "vault", "Expected source to be 'vault'");
    }

    #[test]
    fn config_updated_event_is_send_and_sync() {
        // GIVEN: the ConfigUpdated event type
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN: checking Send + Sync bounds
        is_send_sync::<ConfigUpdated>();

        // THEN: the event type satisfies Send + Sync
    }

    #[test]
    fn config_updated_event_serializes_source() {
        let json = serialized_event();

        assert!(json.contains("vault"), "JSON should contain vault_path field");
    }

    #[test]
    fn config_updated_event_serializes_timestamp() {
        let json = serialized_event();

        assert!(
            json.contains("1234567890"),
            "JSON should contain timestamp value"
        );
    }
}
