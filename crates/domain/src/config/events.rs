//! Configuration domain events.

use serde::{Deserialize, Serialize};

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
    fn config_updated_event_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<ConfigUpdated>();
    }

    #[test]
    fn config_updated_event_is_serializable() {
        let event = ConfigUpdated {
            source: "vault".to_owned(),
            timestamp: 1_234_567_890,
        };

        let result = serde_json::to_string(&event);
        assert!(result.is_ok(), "should serialize successfully");
        if let Ok(json) = result {
            assert!(json.contains("vault"));
            assert!(json.contains("1234567890"));
        }
    }

    #[test]
    fn config_updated_event_is_deserializable() {
        let json = r#"{"source":"vault","timestamp":1234567890}"#;
        let result: Result<ConfigUpdated, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "should deserialize successfully");

        if let Ok(event) = result {
            assert_eq!(event.timestamp, 1_234_567_890);
            assert_eq!(event.source, "vault");
        }
    }
}
