//! Integration tests for the note reader pipeline.
//!
//! # Purpose
//!
//! These tests validate the public, end-to-end behavior of the note reader
//! pipeline. They exercise file ingestion, parsing, and application to the
//! `Note` aggregate using only public APIs. Unit tests cover individual
//! extractors; these integration tests ensure the pipeline composes correctly
//! under realistic inputs.
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
#[expect(
    clippy::panic_in_result_fn,
    reason = "Integration tests use assertions in Result-returning functions"
)]
mod tests {
    use std::path::{Path, PathBuf};

    use lithos_core::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        fs::FsReader,
        note::{
            adapter::reader::{NoteReader, ParseOutcome},
            aggregate::{Note, NoteId},
            list::{ListDepth, ListItem, ListType},
            tag::Tag as NoteTag,
        },
    };
    use tempfile::TempDir;

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    struct Fixture {
        _dir: TempDir,
        outcome: ParseOutcome,
        note: Note,
    }

    fn test_config(root: PathBuf) -> TestResult<Config> {
        let raw = RawConfig::default();
        let root = VaultRoot::try_new(root)?;
        Ok(Config::build(&raw, VaultId::new(), root)?)
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
        let reader = NoteReader::new(&config);
        let fs_reader = FsReader::new(dir.path());

        let outcome = reader.parse(&fs_reader, note_path)?;
        let mut note =
            Note::try_new(NoteId::new(), note_path.to_string_lossy().as_ref())?;
        reader.apply(&fs_reader, &mut note, note_path)?;

        Ok(Fixture {
            _dir: dir,
            outcome,
            note,
        })
    }

    fn sorted_tag_paths_from_outcome(outcome: &ParseOutcome) -> Vec<Box<str>> {
        let mut tags: Vec<Box<str>> = outcome
            .tags()
            .iter()
            .map(NoteTag::full_path)
            .map(Into::into)
            .collect();
        tags.sort();
        tags
    }

    fn sorted_tag_paths_from_note(note: &Note) -> Vec<Box<str>> {
        let mut tags: Vec<Box<str>> =
            note.tags().map(|tag| tag.full_path().into()).collect();
        tags.sort();
        tags
    }

    /// Validates end-to-end parsing and application across all core entities.
    ///
    /// This test covers the full pipeline (file -> parse -> apply) using a
    /// complex markdown fixture to ensure counts and frontmatter presence are
    /// preserved in both the `ParseOutcome` and the `Note` aggregate.
    #[test]
    fn note_reader_full_pipeline_preserves_counts() -> TestResult {
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

        let fixture = build_fixture(markdown)?;

        assert_eq!(fixture.outcome.headings().len(), 3);
        assert_eq!(fixture.outcome.tasks().len(), 3);
        assert_eq!(fixture.outcome.links().len(), 4);
        assert_eq!(fixture.outcome.lists().len(), 3);
        assert_eq!(fixture.outcome.tags().len(), 7);

        let outcome_frontmatter =
            fixture.outcome.frontmatter().expect("frontmatter should exist");
        assert!(outcome_frontmatter.has_raw("title"));

        assert_eq!(fixture.note.headings().count(), 3);
        assert_eq!(fixture.note.tasks().count(), 3);
        assert_eq!(fixture.note.links().count(), 4);
        assert_eq!(fixture.note.lists().count(), 3);
        assert_eq!(fixture.note.tags().count(), 7);

        let note_frontmatter =
            fixture.note.frontmatter().expect("note frontmatter exists");
        assert!(note_frontmatter.has_raw("title"));

        Ok(())
    }

    /// Ensures task promotion retains linkage between tasks and list items.
    ///
    /// This verifies that a promoted checkbox becomes a task and the list item
    /// retains the correct `task_id` in both the parse output and applied note.
    #[test]
    fn note_reader_applies_task_linkage_to_list_items() -> TestResult {
        let markdown = "- [ ] #task Link me\n";

        let fixture = build_fixture(markdown)?;

        let outcome_task =
            fixture.outcome.tasks().first().expect("task exists");
        let outcome_list =
            fixture.outcome.lists().first().expect("list exists");
        let outcome_item = outcome_list.items().next().expect("item exists");
        assert_eq!(outcome_item.task_id(), Some(outcome_task.id()));
        assert!(matches!(outcome_item, ListItem::Checkbox { .. }));

        let note_task = fixture.note.tasks().next().expect("note task exists");
        let note_list = fixture.note.lists().next().expect("note list exists");
        let note_item = note_list.items().next().expect("note item exists");
        assert_eq!(note_item.task_id(), Some(note_task.id()));
        assert!(matches!(note_item, ListItem::Checkbox { .. }));

        Ok(())
    }

    /// Confirms ingestion promotes tasks and preserves list structure.
    ///
    /// This is the minimal ingestion scenario validating that task promotion
    /// and list tracking remain wired correctly in the full pipeline.
    #[test]
    fn note_reader_ingest_promotes_tasks_and_tracks_lists() -> TestResult {
        let markdown = concat!(
            "# Title\n\n",
            "- [ ] #task Review PR [priority:: 1]\n",
            "- [x] Buy milk\n\n",
            "1. First\n",
            "2. Second\n",
        );

        let fixture = build_fixture(markdown)?;

        let lists: Vec<_> = fixture.note.lists().collect();
        assert_eq!(lists.len(), 2, "expected unordered + ordered lists");

        let unordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list missing");
        assert_eq!(unordered.items().count(), 2, "unordered list item count");

        let first_item = unordered.items().next().expect("missing first item");
        let &ListItem::Checkbox {
            task_id,
            status,
            ..
        } = first_item
        else {
            return Err("expected checkbox item".into());
        };

        assert_eq!(status.value(), ' ', "expected unchecked status");
        assert!(task_id.is_some(), "expected promoted task id");
        assert_eq!(fixture.note.tasks().count(), 1, "expected one task");

        Ok(())
    }

    /// Verifies frontmatter tag extraction is visible in the applied note.
    ///
    /// This asserts that frontmatter tags are extracted, de-duplicated, and
    /// surfaced through both the parse outcome and the note aggregate.
    #[test]
    fn note_reader_frontmatter_tags_surface_in_note() -> TestResult {
        let markdown = "---\ntags: [alpha, beta]\n---\n\nBody text\n";

        let fixture = build_fixture(markdown)?;

        let outcome_tags = sorted_tag_paths_from_outcome(&fixture.outcome);
        let note_tags = sorted_tag_paths_from_note(&fixture.note);

        assert!(outcome_tags.contains(&"alpha".into()));
        assert!(outcome_tags.contains(&"beta".into()));
        assert!(note_tags.contains(&"alpha".into()));
        assert!(note_tags.contains(&"beta".into()));

        Ok(())
    }

    /// Ensures Unicode escapes are preserved through the pipeline.
    ///
    /// Integration tests should remain ASCII; Unicode is represented with
    /// escapes to validate tag and heading parsing without introducing literal
    /// non-ASCII source.
    #[test]
    fn note_reader_preserves_unicode_headings_and_tags() -> TestResult {
        let markdown = concat!(
            "# \u{1f44b} \u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}\n",
            "Here is a unicode tag: #\u{30bf}\u{30b0}\n",
        );

        let fixture = build_fixture(markdown)?;

        let heading =
            fixture.outcome.headings().first().expect("heading should exist");
        assert_eq!(
            heading.text(),
            "\u{1f44b} \u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}"
        );

        let outcome_tags = sorted_tag_paths_from_outcome(&fixture.outcome);
        let note_tags = sorted_tag_paths_from_note(&fixture.note);

        assert!(outcome_tags.contains(&"\u{30bf}\u{30b0}".into()));
        assert!(note_tags.contains(&"\u{30bf}\u{30b0}".into()));

        Ok(())
    }

    /// Ensures nested list depths are preserved end-to-end.
    ///
    /// This confirms list nesting depth survives parsing and application and
    /// is not flattened during pipeline composition.
    #[test]
    fn note_reader_preserves_list_depths() -> TestResult {
        let markdown = "1. First\n   - [ ] #task Nested\n";

        let fixture = build_fixture(markdown)?;

        assert_eq!(fixture.outcome.lists().len(), 2);

        let ordered = fixture
            .outcome
            .lists()
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Ordered { .. }))
            .expect("ordered list exists");
        assert_eq!(ordered.depth(), ListDepth::root());

        let unordered = fixture
            .outcome
            .lists()
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list exists");
        assert_eq!(unordered.depth().as_u8(), 1);

        Ok(())
    }
}
