//! Integration tests for markdown ingestion.

use std::sync::Arc;

use traces_fs::metadata::{FileMetadata, FsTimes};
use traces_note::{
    paths::NotePath,
    processor::{NoteFileInfo, NoteProcessAction, NoteProcessor},
    repository::ReadRepository,
    storage::RedbRepository,
};
use traces_settings::{
    aggregate::Version,
    builder,
    vault::{VaultId, VaultRoot},
};

#[cfg(test)]
mod tests {
    use traces_db::testing::TestStore;

    use super::*;

    #[test]
    fn ingest_markdown_promotes_tasks_and_tracks_lists() {
        let db = TestStore::new().expect("test db");
        let root = db.dir_path().to_path_buf();
        std::fs::create_dir_all(root.join("notes")).expect("create notes dir");
        let config = builder::build_from_layers(
            None,
            None,
            VaultId::new(),
            VaultRoot::try_new(root.clone()).expect("vault root"),
            Version::initial(),
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

        let store = Arc::clone(db.store());
        let repository = RedbRepository::new(Arc::clone(&store));
        let source = traces_fs::FileReader::new(root.as_path());

        let note_path =
            NotePath::try_new("notes/ingest.md").expect("note path");
        let info = NoteFileInfo::new(
            note_path.clone(),
            FileMetadata::new(FsTimes::new(None, None), 0, false),
        );
        let report = NoteProcessor::new()
            .process_file(&repository, &config, &source, info)
            .expect("load markdown");
        assert_eq!(
            report.action(),
            NoteProcessAction::Created,
            "expected note creation"
        );
        let note_id = report.note_id().expect("note id");
        let note = repository
            .find_by_id(note_id)
            .expect("query note")
            .expect("note exists");

        assert_eq!(note.list_items().len(), 4, "expected four list items");
        let checkbox_count = note
            .list_items()
            .iter()
            .filter(|item| item.base.is_checkbox.is_some())
            .count();
        assert_eq!(checkbox_count, 2, "expected two checkbox items");
        assert_eq!(note.tasks().len(), 1, "expected one promoted task");
    }
}
