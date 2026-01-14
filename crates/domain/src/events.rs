//! Domain events for Lithos.
//!
//! This module defines all domain-level events following event-driven architecture.
//! Events represent state changes in the domain and are published after successful operations.

use serde::{Deserialize, Serialize};

/// Configuration updated domain event.
///
/// This event is published when configuration changes occur, allowing
/// other bounded contexts to react to configuration updates.
///
/// # Examples
/// ```ignore
/// // Note: ConfigUpdated is #[non_exhaustive] so can only be constructed within the crate.
/// // Adapters will construct this event when publishing configuration changes.
/// use lithos_domain::ConfigUpdated;
///
/// let event = ConfigUpdated {
///     source: "vault".to_string(),
///     timestamp: 1234567890,
/// };
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

    #[test]
    fn template_created_event_is_serializable() {
        use uuid::Uuid;
        let event = TemplateCreated {
            id: Uuid::now_v7(),
            name: "daily-note".to_owned(),
            timestamp: 1_234_567_890,
        };

        let result = serde_json::to_string(&event);
        assert!(result.is_ok(), "should serialize successfully");
    }
}

/// Template created domain event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TemplateCreated {
    /// UUID of the template.
    pub id: uuid::Uuid,
    /// Name of the template.
    pub name: String,
    /// Unix timestamp when the template was created.
    pub timestamp: i64,
}
