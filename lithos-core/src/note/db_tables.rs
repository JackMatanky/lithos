//! Database table definitions for note storage.

use redb::{MultimapTableDefinition, TableDefinition};

pub(crate) const STORED_NOTES: TableDefinition<&str, &[u8]> =
    TableDefinition::new("stored_notes");
pub(crate) const NOTE_EVENTS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("note_events");

pub(crate) const PATH_TO_ID: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("path_to_id");
pub(crate) const TAGS_TO_NOTES: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tags_to_notes");
pub(crate) const ALIAS_TO_ID: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("alias_to_id");
pub(crate) const FILE_CLASS_TO_ID: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("file_class_to_id");
pub(crate) const FOLDER_TO_ID: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("folder_to_id");
pub(crate) const TASKS_BY_COMPLETED_DATE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tasks_by_completed_date");
pub(crate) const TASKS_BY_CREATED_DATE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tasks_by_created_date");
pub(crate) const TASKS_BY_DUE_DATE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tasks_by_due_date");
pub(crate) const TASKS_BY_REMINDER_DATE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tasks_by_reminder_date");
pub(crate) const TASKS_BY_STATUS: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tasks_by_status");
pub(crate) const TASKS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("tasks");
pub(crate) const TASKS_BY_METADATA: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tasks_by_metadata");
pub(crate) const TASKS_BY_DEPENDS_ON: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("tasks_by_depends_on");
pub(crate) const FRONTMATTER_KV: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("frontmatter_kv");
