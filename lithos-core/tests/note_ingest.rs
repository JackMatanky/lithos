//! Integration tests for markdown ingestion.

use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    fs::FsReader,
    note::{
        adapter::reader::NoteReader,
        aggregate::{Note, NoteId},
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
        let unique = format!("lithos_note_ingest_{}", std::process::id());
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(root.join("notes"))?;
        let config = Config::build(
            &RawConfig::default(),
            VaultId::new(),
            VaultRoot::try_new(root.clone())?,
        )?;
        let markdown = concat!(
            "# Title\n\n",
            "- [ ] #task Review PR [priority:: 1]\n",
            "- [x] Buy milk\n\n",
            "1. First\n",
            "2. Second\n",
        );

        std::fs::write(root.join("notes/ingest.md"), markdown)?;

        let mut note = Note::new(NoteId::new(), "notes/ingest.md")?;
        let reader = FsReader::new(root.as_path());
        NoteReader::new(&config).apply(
            &reader,
            &mut note,
            std::path::Path::new("notes/ingest.md"),
        )?;

        let lists: Vec<&List> = note.lists().collect();
        assert_eq!(lists.len(), 2, "expected unordered + ordered lists");

        let unordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list missing");
        assert_eq!(unordered.items().count(), 2, "unordered list item count");

        let first_item = unordered.items().next().expect("missing first item");
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
