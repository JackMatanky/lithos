//! Integration tests for markdown ingestion.

use lithos_core::{
    config::task::TaskConfig,
    note::{
        aggregate::{Note, NoteId},
        ingest::ingest_markdown,
        list::{List, ListItem, ListType},
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Integration test uses assertions in Result-returning \
                  function."
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Test matches &ListItem using match ergonomics."
    )]
    fn ingest_markdown_promotes_tasks_and_tracks_lists()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = TaskConfig::default();
        let markdown = concat!(
            "# Title\n\n",
            "- [ ] #task Review PR [priority:: 1]\n",
            "- [x] Buy milk\n\n",
            "1. First\n",
            "2. Second\n",
        );

        let mut note = Note::new(NoteId::new(), "notes/ingest.md".to_owned())?;
        ingest_markdown(&mut note, markdown, &config)?;

        let lists: Vec<&List> = note.lists().collect();
        assert_eq!(lists.len(), 2, "expected unordered + ordered lists");

        let unordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list missing");
        assert_eq!(unordered.items().len(), 2, "unordered list item count");

        let first_item = unordered.items().first().expect("missing first item");
        let ListItem::Checkbox {
            task_id,
            status,
            ..
        } = first_item
        else {
            return Err("expected checkbox item".into());
        };

        assert_eq!(status.value(), ' ', "expected unchecked status");
        assert!(task_id.is_some(), "expected promoted task id");
        assert_eq!(note.tasks().count(), 1, "expected one promoted task");

        Ok(())
    }
}
