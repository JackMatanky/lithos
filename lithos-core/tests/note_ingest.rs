//! Integration tests for markdown ingestion.

use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    db::Database,
    note::{
        loader::Loader as NoteLoader,
        storage::{RedbRepository, Repository},
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

        let db_path = root.join("notes.redb");
        let db = Database::open(&db_path).expect("open db");
        let repository = RedbRepository::new(&db, &config);
        let loader = NoteLoader::new(&repository, &config);

        let note_path =
            lithos_core::note::paths::NotePath::try_new("notes/ingest.md")
                .expect("note path");
        let note_id = loader
            .load_content(&note_path, markdown.into(), None, None)
            .expect("load markdown");
        let note = repository
            .find_by_id(note_id)
            .expect("query note")
            .expect("note exists");

        assert_eq!(note.list_items().len(), 4, "expected four list items");
        let checkbox_count = note
            .list_items()
            .iter()
            .filter(|item| item.status().is_some())
            .count();
        assert_eq!(checkbox_count, 2, "expected two checkbox items");
        assert_eq!(note.tasks().len(), 1, "expected one promoted task");
    }
}
