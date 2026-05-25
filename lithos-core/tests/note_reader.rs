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
        config::{
            aggregate::Config,
            builder,
            task::StatusSymbol,
            vault::{VaultId, VaultRoot},
        },
        db::Store,
        fs::PathKey,
        note::{
            aggregate::Note, repository::ReadRepository as _,
            storage::RedbRepository as NoteRepository, tag::Tag as NoteTag,
        },
        vault::VaultProcessor,
    };
    use tempfile::TempDir;

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    struct Fixture {
        _dir: TempDir,
        store: Arc<Store>,
        note: Note,
        config: Config,
    }

    fn test_config(root: PathBuf) -> TestResult<Config> {
        let root = VaultRoot::try_new(root)?;
        Ok(builder::build_from_layers(
            None,
            None,
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
        let store = Arc::new(Store::open(&db_path)?);
        let report =
            VaultProcessor::new().process_full(Arc::clone(&store), &config)?;
        if report.markdown_routed() == 0 {
            return Err(std::io::Error::other(
                "expected markdown routing to occur",
            )
            .into());
        }

        let repository = NoteRepository::new(Arc::clone(&store));
        let notes = repository.list()?;
        let note = notes
            .first()
            .cloned()
            .ok_or_else(|| std::io::Error::other("expected stored note"))?;
        Ok(Fixture {
            _dir: dir,
            store,
            note,
            config,
        })
    }

    fn build_environment(
        markdown: &str,
    ) -> TestResult<(TempDir, Config, Arc<Store>)> {
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
        let store = Arc::new(Store::open(&db_path)?);
        Ok((dir, config, store))
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

        let repository = NoteRepository::new(Arc::clone(&fixture.store));
        let mut tasks = Vec::new();
        let notes = repository.list()?;
        for note in notes {
            for task in note.tasks() {
                if task.status() == todo.as_str()
                    || task.status() == done.as_str()
                {
                    tasks.push(task.clone());
                }
            }
        }
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
        assert_eq!(fixture.note.tags().len(), 7);

        let outcome_frontmatter =
            fixture.note.frontmatter().expect("frontmatter should exist");
        assert!(outcome_frontmatter.has("title"));
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
            tasks.iter().map(lithos_core::note::task::Task::status).collect();
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
        assert!(!task.status().is_empty());
    }

    #[test]
    fn load_skips_unchanged_notes() {
        let (dir, config, store) =
            build_environment("# Title\n- [ ] #task Review PR")
                .expect("environment");
        let processor = VaultProcessor::new();
        let repository = NoteRepository::new(Arc::clone(&store));

        let first = processor
            .process_full(Arc::clone(&store), &config)
            .expect("first load");
        assert_eq!(
            first.markdown_routed(),
            1,
            "first load should index one note"
        );
        let mut first_notes = repository.list().expect("first notes");
        let _first_note = first_notes.pop().expect("expected stored note");

        let second = VaultProcessor::new()
            .process_full(Arc::clone(&store), &config)
            .expect("second load");
        assert_eq!(
            second.notes_created_or_updated(),
            0,
            "second load should skip unchanged note"
        );
        let mut second_notes = repository.list().expect("second notes");
        let _second_note = second_notes.pop().expect("expected stored note");
        drop(dir);
    }

    #[test]
    fn load_removes_missing_notes() {
        let (dir, config, store) =
            build_environment("# Title\n- [ ] #task Review PR")
                .expect("environment");
        let processor = VaultProcessor::new();
        let repository = NoteRepository::new(Arc::clone(&store));

        let _report = processor
            .process_full(Arc::clone(&store), &config)
            .expect("first load");
        let note_path = dir.path().join("notes/note.md");
        std::fs::remove_file(note_path).expect("remove note");

        let _second_report = VaultProcessor::new()
            .process_full(Arc::clone(&store), &config)
            .expect("second load");
        let notes = repository.list().expect("list notes");
        assert!(notes.is_empty(), "expected note to be removed");
    }

    #[test]
    fn full_scan_reports_pruned_files_for_removed_notes() {
        let (dir, config, store) =
            build_environment("# Title\n- [ ] #task Review PR")
                .expect("environment");

        let _first = VaultProcessor::new()
            .process_full(Arc::clone(&store), &config)
            .expect("first load");

        let note_path = dir.path().join("notes/note.md");
        std::fs::remove_file(note_path).expect("remove note");

        let second = VaultProcessor::new()
            .process_full(Arc::clone(&store), &config)
            .expect("second load");
        assert_eq!(
            second.files_deleted(),
            1,
            "full scan should prune one removed file"
        );
    }

    #[test]
    fn partial_scan_does_not_prune_unscanned_missing_notes() {
        let dir = TempDir::new().expect("temp dir");
        let notes_dir = dir.path().join("notes");
        std::fs::create_dir_all(&notes_dir).expect("create notes dir");

        let keep_path = notes_dir.join("keep.md");
        let drop_path = notes_dir.join("drop.md");
        std::fs::write(&keep_path, "# Keep\n").expect("write keep note");
        std::fs::write(&drop_path, "# Drop\n").expect("write drop note");

        let config = test_config(dir.path().to_path_buf()).expect("config");
        let db_path = dir.path().join("notes.redb");
        let store = Arc::new(Store::open(&db_path).expect("open db"));
        let repository = NoteRepository::new(Arc::clone(&store));

        let first = VaultProcessor::new()
            .process_full(Arc::clone(&store), &config)
            .expect("first full scan");
        assert_eq!(
            first.markdown_routed(),
            2,
            "both notes should route on full scan"
        );

        std::fs::remove_file(&drop_path).expect("remove drop note");

        let partial_paths = vec![
            PathKey::try_new("notes/keep.md").expect("partial vault path"),
        ];
        let partial = VaultProcessor::new()
            .process_partial(Arc::clone(&store), &config, &partial_paths)
            .expect("partial scan");

        assert_eq!(
            partial.files_deleted(),
            0,
            "partial scan must not prune paths outside scan input"
        );

        let notes = repository.list().expect("list notes");
        assert_eq!(
            notes.len(),
            2,
            "missing unscanned note remains until next full prune"
        );
    }
}
