//! Ingestion services for bounded contexts.
//!
//! This module contains specialized ingestion services for each bounded
//! context:
//!
//! - **SchemaIngestionService**: Handles schema definition file ingestion
//! - **TemplateIngestionService**: Handles template file ingestion (TODO)
//! - **NoteIngestionService**: Handles note file ingestion (TODO)
//!
//! Each service follows the same pattern:
//! 1. Read file via `FileSource`
//! 2. Parse into raw/input type
//! 3. Validate into domain aggregate
//! 4. Persist via command port

/// Note ingestion service.
pub mod note_ingestion;
/// Schema ingestion service.
pub mod schema_ingestion;
/// Template ingestion service.
pub mod template_ingestion;

/// Note ingestion service type alias.
pub type NoteIngestionService<'svc, Q, C> =
    note_ingestion::NoteIngestionService<'svc, Q, C>;

/// Schema ingestion service type alias.
pub type SchemaIngestionService<'svc, Q, C> =
    schema_ingestion::SchemaIngestionService<'svc, Q, C>;

/// Template ingestion service type alias.
pub type TemplateIngestionService<'svc, Q, C> =
    template_ingestion::TemplateIngestionService<'svc, Q, C>;
