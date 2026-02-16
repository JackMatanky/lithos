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

/// Schema ingestion service.
pub mod schema_ingestion;

/// Schema ingestion service type alias.
pub type SchemaIngestionService<'svc, Q, C> =
    schema_ingestion::SchemaIngestionService<'svc, Q, C>;
