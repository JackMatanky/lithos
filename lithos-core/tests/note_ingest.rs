//! Integration tests for markdown ingestion.

use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    fs::FsReader,
    note::{
        list::{List, ListItem, ListType},
        reader::NoteReader,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_markdown_promotes_tasks_and_tracks_lists() {
        let unique = format!("lithos_note_ingest_{}", std::process::id());
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(root.join("notes")).expect("create notes dir");
        let config = Config::build(
            &RawConfig::default(),
            VaultId::new(),
            VaultRoot::try_new(root.clone()).expect("vault root"),
            lithos_core::config::aggregate::Version::initial(),
        )
        .expect("config build");
        let markdown = concat!(
            "# Title\n\n",
            "- [ ] #task Review PR [priority:: 1]\n",
            "- [x] Buy milk\n\n",
            "1. First\n",
            "2. Second\n",
        );

        std::fs::write(root.join("notes/ingest.md"), markdown)
            .expect("write markdown");

        let reader = FsReader::new(root.as_path());
        let parsed = NoteReader::new(&config)
            .parse(&reader, std::path::Path::new("notes/ingest.md"))
            .expect("parse markdown");
        assert!(parsed.modified_at().is_some(), "expected modified_at");

        let lists: Vec<&List> = parsed.lists().iter().collect();
        assert_eq!(lists.len(), 2, "expected unordered + ordered lists");

        let unordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list missing");
        let items: Vec<_> = unordered.items().collect();
        assert_eq!(items.len(), 2, "unordered list item count");

        let first_item = items.first().expect("missing first item");
        assert!(
            matches!(**first_item, ListItem::Checkbox { .. }),
            "expected checkbox item"
        );
        let ListItem::Checkbox {
            task_id,
            status,
            ..
        } = (*first_item).clone()
        else {
            return;
        };

        assert_eq!(status.value(), ' ', "expected unchecked status");
        assert!(task_id.is_some(), "expected promoted task id");
        assert_eq!(parsed.tasks().len(), 1, "expected one promoted task");
    }
}
