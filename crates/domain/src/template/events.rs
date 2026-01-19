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
/// use lithos_domain::TemplateCreated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = TemplateCreated::new(id, "daily-note".to_string(), 1234567890);
/// assert_eq!(event.id, id);
/// assert_eq!(event.name, "daily-note");
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

impl TemplateCreated {
    /// Creates a new template created event.
    #[inline]
    #[must_use]
    pub fn new(id: Uuid, name: String, timestamp: i64) -> Self {
        Self {
            id,
            name,
            timestamp,
        }
    }
}

/// Domain events that can be emitted by the Template aggregate.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TemplateEvents {
    /// Template was created.
    TemplateCreated(TemplateCreated),
}
