//! Orchestration helpers for ingesting raw content into the Note domain.
//!
//! Provides high-level functions to populate a [`crate::note::aggregate::Note`]
//! aggregate from external sources or raw markdown.

//! Markdown ingestion helpers for the note context.
//!
//! Keeps adapter-facing parsing orchestration out of the core aggregate.

use super::{aggregate::Note, error::NoteError, parser::NoteParser};
use crate::config::task::TaskConfig;

/// Apply markdown parsing to a note using the task configuration.
///
/// # Errors
///
/// Returns `NoteError` when parsing fails.
#[inline]
pub fn ingest_markdown(
    note: &mut Note,
    markdown: &str,
    task_config: &TaskConfig,
) -> Result<(), NoteError> {
    NoteParser::new(task_config).apply_to_note(note, markdown)
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;
    use crate::note::aggregate::NoteId;

    #[test]
    fn ingest_markdown_populates_lists_and_tasks() -> Result<(), NoteError> {
        let config = TaskConfig::default();
        let mut note = Note::new(NoteId::new(), "notes/ingest.md".to_owned())?;

        ingest_markdown(&mut note, "- [ ] #task Review PR\n", &config)?;

        assert_eq!(note.lists().count(), 1, "expected one list");
        assert_eq!(note.tasks().count(), 1, "expected one task");
        Ok(())
    }
}
