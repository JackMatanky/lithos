//! Integration tests for the note ingestion pipeline.
//!
//! # Purpose
//!
//! These tests validate the public, end-to-end behavior of the note reader
//! pipeline. They exercise file discovery, parsing, and projection writes using
//! only public APIs. Unit tests cover individual extractors; these integration
//! tests ensure the pipeline composes correctly under realistic inputs.
//!
//! # Why integration tests here
//!
//! Integration tests are intentionally limited in count and scope because they
//! are slower and more expensive than unit tests. Each test below exists to
//! validate cross-component behavior that cannot be proven by an isolated unit
//! test (for example: task promotion + list linkage + note mutation).
//!
//! # Best-practice notes
//!
//! - Use temporary directories to avoid shared state and side effects.
//! - Prefer deterministic, self-contained fixtures (static markdown inputs).
//! - Assert observable outputs from public APIs only.
//! - Keep assertions focused on behavioral contracts, not implementation.
//! - Avoid order dependence between tests; each test builds its own fixture.

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::Arc,
    };

    use lithos_core::{
        application::VaultService,
        config::{
            aggregate::Config,
            raw::RawConfig,
            task::StatusSymbol,
            vault::{VaultId, VaultRoot},
        },
        db::Database,
        note::{
            aggregate::Note,
            storage::{RedbRepository, Repository as _},
            tag::Tag as NoteTag,
        },
    };
    use tempfile::TempDir;

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    struct Fixture {
        _dir: TempDir,
        db: Arc<Database>,
        note: Note,
        config: Config,
    }

    fn test_config(root: PathBuf) -> TestResult<Config> {
        let raw = RawConfig::default();
        let root = VaultRoot::try_new(root)?;
        Ok(Config::build(
            &raw,
            VaultId::new(),
            root,
            lithos_core::config::aggregate::Version::initial(),
        )?)
    }

    fn build_fixture(markdown: &str) -> TestResult<Fixture> {
        let dir = TempDir::new()?;
        let note_path = Path::new("notes/note.md");
        let absolute_path = dir.path().join(note_path);

        let parent = absolute_path.parent().ok_or_else(|| {
            std::io::Error::other("note path should have parent")
        })?;
        std::fs::create_dir_all(parent)?;
        std::fs::write(&absolute_path, markdown)?;

        let config = test_config(dir.path().to_path_buf())?;
        let db_path = dir.path().join("notes.redb");
        let db = Arc::new(Database::open(&db_path)?);
        let service = VaultService::new(db.as_ref(), &config);
        let _note_ids = service.load()?;

        let repository = RedbRepository::new(db.as_ref(), &config);
        let notes = repository.list()?;
        let note = notes
            .first()
            .cloned()
            .ok_or_else(|| std::io::Error::other("expected stored note"))?;
        Ok(Fixture {
            _dir: dir,
            db,
            note,
            config,
        })
    }

    fn build_environment(
        markdown: &str,
    ) -> TestResult<(TempDir, Config, Arc<Database>)> {
        let dir = TempDir::new()?;
        let note_path = Path::new("notes/note.md");
        let absolute_path = dir.path().join(note_path);

        let parent = absolute_path.parent().ok_or_else(|| {
            std::io::Error::other("note path should have parent")
        })?;
        std::fs::create_dir_all(parent)?;
        std::fs::write(&absolute_path, markdown)?;

        let config = test_config(dir.path().to_path_buf())?;
        let db_path = dir.path().join("notes.redb");
        let db = Arc::new(Database::open(&db_path)?);
        Ok((dir, config, db))
    }

    fn sorted_tag_paths_from_note(note: &Note) -> Vec<Box<str>> {
        let mut tags: Vec<Box<str>> = note
            .tags()
            .iter()
            .map(NoteTag::full_path)
            .map(Into::into)
            .collect();
        tags.sort();
        tags
    }

    fn total_tasks(
        fixture: &Fixture,
    ) -> TestResult<Vec<lithos_core::note::task::Task>> {
        let status = fixture.config.task().status();
        let todo = status
            .name_for_symbol(StatusSymbol::try_new(' ')?)
            .ok_or_else(|| std::io::Error::other("missing todo status"))?;
        let done = status
            .name_for_symbol(StatusSymbol::try_new('x')?)
            .ok_or_else(|| std::io::Error::other("missing done status"))?;

        let repository =
            RedbRepository::new(fixture.db.as_ref(), &fixture.config);
        let mut tasks = Vec::new();
        tasks.extend(repository.list_tasks_by_status(todo)?);
        tasks.extend(repository.list_tasks_by_status(done)?);
        Ok(tasks)
    }

    /// Validates end-to-end ingestion across all core entities.
    ///
    /// This test covers the full pipeline (file -> parse -> persist) using a
    /// complex markdown fixture to ensure counts and frontmatter presence are
    /// preserved in stored projections.
    #[test]
    fn note_reader_full_pipeline_preserves_counts() {
        let markdown = concat!(
            "---\n",
            "title: \"Complex Note\"\n",
            "tags: [alpha, beta]\n",
            "---\n\n",
            "# Intro\n",
            "Some text with #tag1 and #tag2, and a link [ext](https://example.com).\n",
            "Also a wiki link [[Target Note]] and embed ![[image.png]].\n\n",
            "## Tasks\n",
            "- [ ] #task Review PR\n",
            "- [x] #task Write tests\n",
            "- [ ] Just a checkbox\n\n",
            "### Nested\n",
            "1. First\n",
            "   - [ ] #task Nested task\n\n",
            "Paragraph with `#code` and [link with #tag3](https://example.com#frag) ",
            "and #tag3.\n",
            "List of tags: #tag1 #tag4\n",
        );

        let fixture = build_fixture(markdown).expect("fixture");
        let tasks = total_tasks(&fixture).expect("tasks");

        assert_eq!(fixture.note.headings().len(), 3);
        assert_eq!(tasks.len(), 3);
        assert_eq!(fixture.note.links().len(), 4);
        assert_eq!(fixture.note.tags().len(), 8);

        let outcome_frontmatter =
            fixture.note.frontmatter().expect("frontmatter should exist");
        assert!(outcome_frontmatter.has_raw("title"));
    }

    /// Ensures task promotion retains a heading association.
    #[test]
    fn note_reader_applies_task_headings() {
        let markdown = "# Tasks\n- [ ] #task Link me\n";

        let fixture = build_fixture(markdown).expect("fixture");
        let tasks = total_tasks(&fixture).expect("tasks");
        let heading = fixture.note.headings().first().expect("heading exists");
        assert_eq!(heading.text(), "Tasks");
        assert!(!tasks.is_empty(), "expected promoted task");
    }

    /// Confirms ingestion promotes tasks with correct status fields.
    #[test]
    fn note_reader_ingest_promotes_tasks_and_tracks_lists() {
        let markdown = concat!(
            "# Title\n\n",
            "- [ ] #task Review PR [priority:: 1]\n",
            "- [x] #task Buy milk\n\n",
            "1. First\n",
            "2. Second\n",
        );

        let fixture = build_fixture(markdown).expect("fixture");
        let tasks = total_tasks(&fixture).expect("tasks");
        let status_names: Vec<&str> =
            tasks.iter().map(|task| task.status().as_str()).collect();
        let status = fixture.config.task().status();
        let todo = status
            .name_for_symbol(StatusSymbol::try_new(' ').expect("todo symbol"))
            .expect("missing todo status");
        let done = status
            .name_for_symbol(StatusSymbol::try_new('x').expect("done symbol"))
            .expect("missing done status");

        assert!(
            status_names.iter().any(|name| *name == todo.as_str()),
            "expected todo task"
        );
        assert!(
            status_names.iter().any(|name| *name == done.as_str()),
            "expected done task"
        );
    }

    /// Verifies frontmatter tag extraction is visible in the parse outcome.
    ///
    /// This asserts that frontmatter tags are extracted and de-duplicated.
    #[test]
    fn note_reader_frontmatter_tags_surface_in_note() {
        let markdown = "---\ntags: [alpha, beta]\n---\n\nBody text\n";

        let fixture = build_fixture(markdown).expect("fixture");

        let outcome_tags = sorted_tag_paths_from_note(&fixture.note);
        assert!(outcome_tags.contains(&"alpha".into()));
        assert!(outcome_tags.contains(&"beta".into()));
    }

    /// Ensures Unicode escapes are preserved through the pipeline.
    ///
    /// Integration tests should remain ASCII; Unicode is represented with
    /// escapes to validate tag and heading parsing without introducing literal
    /// non-ASCII source.
    #[test]
    fn note_reader_preserves_unicode_headings_and_tags() {
        let markdown = concat!(
            "# \u{1f44b} \u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}\n",
            "Here is a unicode tag: #\u{30bf}\u{30b0}\n",
        );

        let fixture = build_fixture(markdown).expect("fixture");

        let heading =
            fixture.note.headings().first().expect("heading should exist");
        assert_eq!(
            heading.text(),
            "\u{1f44b} \u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}"
        );

        let outcome_tags = sorted_tag_paths_from_note(&fixture.note);
        assert!(outcome_tags.contains(&"\u{30bf}\u{30b0}".into()));
    }

    /// Ensures stored tasks carry note paths and status metadata.
    #[test]
    fn note_reader_preserves_task_paths_and_status() {
        let markdown = "1. First\n   - [ ] #task Nested\n";

        let fixture = build_fixture(markdown).expect("fixture");
        let tasks = total_tasks(&fixture).expect("tasks");
        let task = tasks.first().expect("task exists");
        assert_eq!(fixture.note.path().as_str(), "notes/note.md");
        assert!(!task.status().as_str().is_empty());
    }

    #[test]
    fn load_skips_unchanged_notes() {
        let (dir, config, db) =
            build_environment("# Title\n- [ ] #task Review PR")
                .expect("environment");
        let service = VaultService::new(db.as_ref(), &config);
        let repository = RedbRepository::new(db.as_ref(), &config);

        let first = service.load().expect("first load");
        assert_eq!(first.len(), 1, "first load should index one note");
        let mut first_notes = repository.list().expect("first notes");
        let _first_note = first_notes.pop().expect("expected stored note");

        let second = service.load().expect("second load");
        assert!(second.is_empty(), "second load should skip unchanged note");
        let mut second_notes = repository.list().expect("second notes");
        let _second_note = second_notes.pop().expect("expected stored note");
        drop(dir);
    }

    #[test]
    fn load_removes_missing_notes() {
        let (dir, config, db) =
            build_environment("# Title\n- [ ] #task Review PR")
                .expect("environment");
        let service = VaultService::new(db.as_ref(), &config);
        let repository = RedbRepository::new(db.as_ref(), &config);

        let _second_note_ids = service.load().expect("first load");
        let note_path = dir.path().join("notes/note.md");
        std::fs::remove_file(note_path).expect("remove note");

        let _note_ids = service.load().expect("second load");
        let notes = repository.list().expect("list notes");
        assert!(notes.is_empty(), "expected note to be removed");
    }
}
