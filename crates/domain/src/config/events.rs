//! Configuration domain events.

use serde::{Deserialize, Serialize};

/// Domain events that can be emitted by the Config aggregate.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConfigEvents {
    /// Configuration was updated.
    ConfigUpdated(ConfigUpdated),
}

/// Configuration updated domain event.
///
/// This event is published when configuration changes occur, allowing
/// other bounded contexts to react to configuration updates.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigUpdated;
///
/// let event = ConfigUpdated::new("vault".to_string(), 1234567890);
/// assert_eq!(event.timestamp, 1234567890);
/// assert_eq!(event.source, "vault");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfigUpdated {
    /// Source of the configuration change (e.g., "global", "vault").
    pub source: String,
    /// Unix timestamp when the configuration was updated.
    pub timestamp: i64,
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

    #[test]
    fn config_updated_event_is_deserializable() {
        // GIVEN: JSON for a configuration updated event
        let json = r#"{"source":"vault","timestamp":1234567890}"#;

        // WHEN: deserializing into ConfigUpdated
        let result: Result<ConfigUpdated, _> = serde_json::from_str(json);

        // THEN: deserialization succeeds and preserves fields
        assert!(result.is_ok(), "should deserialize successfully");

        if let Ok(event) = result {
            assert_eq!(event.timestamp, 1_234_567_890);
            assert_eq!(event.source, "vault");
        }
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
    fn config_updated_event_is_serializable() {
        // GIVEN: a configuration updated event
        let event = ConfigUpdated {
            source: "vault".to_owned(),
            timestamp: 1_234_567_890,
        };

        // WHEN: serializing to JSON
        let result = serde_json::to_string(&event);

        // THEN: serialization succeeds and includes expected fields
        assert!(result.is_ok(), "should serialize successfully");
        if let Ok(json) = result {
            assert!(json.contains("vault"));
            assert!(json.contains("1234567890"));
        }
    }
}
