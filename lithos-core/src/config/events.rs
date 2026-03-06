//! Configuration domain events.
//!
//! This module defines the [`Events`] emitted when configuration state
//! changes, allowing other contexts to react to updates.

#![expect(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived structs and enums"
)]

use rkyv::{Archive, Deserialize, Serialize};

/// Domain events that can be emitted by the Config aggregate.
///
/// These events represent significant changes in the configuration state
/// and are used to synchronize other bounded contexts.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum Events {
    /// Configuration was updated.
    ///
    /// This variant contains the details of which source triggered the
    /// update and at what time.
    ConfigUpdated(ConfigUpdated),
}

/// Configuration updated domain event.
///
/// This event is published when configuration changes occur, allowing
/// other bounded contexts to react to configuration updates.
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::events::ConfigUpdated;
///
/// let event = ConfigUpdated::new("vault", 1234567890);
/// assert_eq!(event.timestamp, 1234567890);
/// assert_eq!(event.source, "vault");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
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
    pub fn new(source: &str, timestamp: i64) -> Self {
        Self {
            source: source.into(),
            timestamp,
        }
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_updated_event_is_send_and_sync() {
        // GIVEN: the ConfigUpdated event type
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN: checking Send + Sync bounds
        is_send_sync::<ConfigUpdated>();

        // THEN: the event type satisfies Send + Sync
    }

    #[test]
    fn config_updated_event_captures_payload() {
        let event = ConfigUpdated::new("vault", 1_234_567_890);

        assert_eq!(event.source, "vault");
        assert_eq!(event.timestamp, 1_234_567_890);
    }
}
