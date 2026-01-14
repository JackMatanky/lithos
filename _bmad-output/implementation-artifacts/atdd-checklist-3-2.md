# ATDD Checklist: Story 3.2 - Note Bounded Context

## Status: RED Phase (Acceptance Tests Generated & Failing)

- [x] **Infrastructure & Filesystem**
  - [x] Create `crates/domain/src/models/note/` directory.
  - [x] Update `crates/domain/src/models/mod.rs` and `crates/domain/src/lib.rs` for new module.
  - [x] Update `crates/domain/src/errors.rs` with Note-specific error variants.

- [ ] **Note Entity (Aggregate Root)**
  - [x] [RED] `returns_error_when_path_is_empty`
  - [x] [RED] `returns_error_when_path_is_absolute`
  - [x] [RED] `returns_error_when_path_contains_traversal`
  - [x] [RED] `returns_error_when_path_missing_md_extension`
  - [x] [RED] `generates_monotonic_uuid_v7_ids` (Stubbed)
  - [ ] [RED] `preserves_id_when_path_is_updated`
  - [ ] [RED] `emits_note_created_event_on_construction`

- [ ] **Frontmatter Subentity**
  - [x] [GREEN] `parses_iso8601_date_successfully` (Enum construction works)
  - [ ] [RED] `parses_momentjs_date_successfully`
  - [ ] [RED] `falls_back_to_string_for_unknown_date_format`
  - [x] [GREEN] `converts_numeric_values_correctly`
  - [x] [GREEN] `converts_boolean_values_correctly`

- [ ] **Tag Subentity**
  - [x] [RED] `parses_hierarchical_tag_successfully`
  - [ ] [RED] `returns_error_for_invalid_tag_characters`
  - [x] [RED] `returns_error_for_empty_tag_segments`
  - [ ] [RED] `returns_error_for_leading_or_trailing_slashes`

- [ ] **Link Subentity**
  - [x] [RED] `parses_wikilink_with_alias_successfully`
  - [ ] [RED] `parses_wikilink_with_header_successfully`
  - [x] [RED] `tracks_link_position_in_document`

- [ ] **Heading Subentity**
  - [x] [RED] `validates_heading_levels_1_to_6`
  - [x] [RED] `returns_error_for_invalid_heading_level_0`
  - [x] [RED] `returns_error_for_invalid_heading_level_7`

- [ ] **Task Subentity**
  - [x] [RED] `parses_all_task_status_variants`
  - [ ] [RED] `tracks_task_position_in_document`

- [ ] **Section Subentity**
  - [ ] [RED] `associates_section_with_optional_heading`
  - [x] [GREEN] `calculates_content_range_correctly`

- [ ] **Fixtures & Support**
  - [ ] Create `crates/domain/src/models/note/fixtures.rs` with `TEST_NOTE_ID`.
  - [ ] Implement `test_builder!` macro usage (or stub).

- [ ] **Validation & Quality**
  - [ ] Verify `crates/domain` has zero I/O dependencies (Purity Test).
  - [ ] Achieve 80%+ test coverage (target).
