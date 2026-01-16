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
    pub id: uuid::Uuid,
    /// Name of the template.
    pub name: String,
    /// Unix timestamp when the template was created.
    pub timestamp: i64,
}

impl TemplateCreated {
    /// Creates a new template created event.
    #[inline]
    #[must_use]
    pub fn new(id: uuid::Uuid, name: String, timestamp: i64) -> Self {
        Self {
            id,
            name,
            timestamp,
        }
    }
}

/// Note created domain event.
///
/// Published when a new Note aggregate is created, allowing other
/// bounded contexts to react to note lifecycle events.
///
/// # Examples
/// ```
/// use lithos_domain::NoteCreated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = NoteCreated::new(id, "projects/lithos.md".to_string(), 1234567890);
/// assert_eq!(event.id, id);
/// assert_eq!(event.path, "projects/lithos.md");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NoteCreated {
    /// UUID v7 of the note.
    pub id: uuid::Uuid,
    /// Vault-relative path of the note.
    pub path: String,
    /// Unix timestamp when the note was created.
    pub timestamp: i64,
}

impl NoteCreated {
    /// Creates a new note created event.
    #[inline]
    #[must_use]
    pub fn new(id: uuid::Uuid, path: String, timestamp: i64) -> Self {
        Self {
            id,
            path,
            timestamp,
        }
    }
}

/// Frontmatter validated domain event.
///
/// Published when frontmatter has been validated against schema in the application layer,
/// allowing other systems to react to validated metadata.
///
/// # Emission Point
/// This event is emitted by the application layer after schema compliance validation,
/// NOT by the domain layer. The domain layer only validates structural consistency.
///
/// # Examples
/// ```
/// use lithos_domain::FrontmatterValidated;
/// use uuid::Uuid;
///
/// let id = Uuid::now_v7();
/// let event = FrontmatterValidated::new(id, 5, 1234567890);
/// assert_eq!(event.note_id, id);
/// assert_eq!(event.field_count, 5);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FrontmatterValidated {
    /// Number of frontmatter fields validated.
    pub field_count: usize,
    /// UUID v7 of the note containing this frontmatter.
    pub note_id: uuid::Uuid,
    /// Unix timestamp when validation occurred.
    pub timestamp: i64,
}

impl FrontmatterValidated {
    /// Creates a new frontmatter validated event.
    #[inline]
    #[must_use]
    pub fn new(
        note_id: uuid::Uuid,
        field_count: usize,
        timestamp: i64,
    ) -> Self {
        Self {
            field_count,
            note_id,
            timestamp,
        }
    }
}

/// Schema created domain event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaCreated {
    /// UUID of the schema.
    pub id: uuid::Uuid,
    /// Name of the schema.
    pub name: String,
    /// Unix timestamp when the schema was created.
    pub timestamp: i64,
}

impl SchemaCreated {
    /// Creates a new schema created event.
    #[inline]
    #[must_use]
    pub fn new(id: uuid::Uuid, name: String, timestamp: i64) -> Self {
        Self {
            id,
            name,
            timestamp,
        }
    }
}

/// Property bank updated domain event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyBankUpdated {
    /// Number of properties in the bank after update.
    pub property_count: usize,
    /// Unix timestamp when the update occurred.
    pub timestamp: i64,
}

impl PropertyBankUpdated {
    /// Creates a new property bank updated event.
    #[inline]
    #[must_use]
    pub fn new(property_count: usize, timestamp: i64) -> Self {
        Self {
            property_count,
            timestamp,
        }
    }
}
