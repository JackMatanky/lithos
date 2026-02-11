//! Template domain events.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Template created domain event.
///
/// Published when a new template is created, allowing other bounded contexts
/// to react to template availability.
///
/// # Examples
/// ```
/// use lithos_core::template::events::TemplateCreated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = TemplateCreated::new(id, "daily-note", 1234567890);
/// assert_eq!(event.id, id, "Template id should match");
/// assert_eq!(event.name, "daily-note", "Template name should match");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TemplateCreated {
    /// UUID of the template.
    pub id: Uuid,
    /// Name of the template.
    pub name: String,
    /// Unix timestamp when the template was created.
    pub timestamp: i64,
}

/// Domain events that can be emitted by the Template aggregate.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Events {
    /// Template was created.
    TemplateCreated(TemplateCreated),
}

impl TemplateCreated {
    /// Creates a new template created event.
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
        // GIVEN the events enum
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN checking Send + Sync bounds
        is_send_sync::<Events>();

        // THEN it satisfies the bounds
    }
}
