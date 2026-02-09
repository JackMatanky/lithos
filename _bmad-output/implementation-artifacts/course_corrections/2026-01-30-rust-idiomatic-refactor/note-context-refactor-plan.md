# Note Context Refactor Implementation Plan

**Date**: 2026-02-08
**Context**: Course correction for Note Bounded Context
**Specs**:
- `docs/design/004-note-models.md`
- `docs/design/005-note-cqrs.md`
- `docs/design/006-note-frontmatter.md`
- `docs/design/007-note-list-task.md`

## Overview

This plan aligns the `lithos-core/src/note` context with the reviewed design specifications. The goal is to enforce idiomatic Rust patterns, type-driven design, and strict rkyv compatibility, while enabling rich task/list modeling.

## Validation Status

- [x] **004-note-models.md**: Coherent. Defines core aggregate and value objects.
- [x] **005-note-cqrs.md**: Coherent. Defines split error types and zero-copy query strategy.
- [x] **006-note-frontmatter.md**: Coherent. Integrates `FieldValue` and strict/lenient extraction.
- [x] **007-note-list-task.md**: Coherent. Defines `List`, `Task`, `TaskMetadata` and usage of `FieldValue`.

## Implementation Phases

### Phase 1: Shared Primitives - ✅ COMPLETE
**Goal**: Establish the `FieldValue` primitive used by Frontmatter and Tasks.
- [x] Create `lithos-core/src/note/value.rs`.
- [x] Implement `FieldValue` enum (String, Number, Boolean, Date, Array, Object).
- [x] Implement `From` implementations or helper constructors.
- [x] Add unit tests for `FieldValue`.

### Phase 2: Domain Value Objects - ✅ COMPLETE
**Goal**: Implement the leaf node value objects.
- [x] **Tag**: Update `lithos-core/src/note/tag.rs` (Tag, TagPath).
- [x] **Link**: Update `lithos-core/src/note/link.rs` (Link, Target, Anchor).
- [x] **Structure**: Update `lithos-core/src/note/structure.rs` (Heading, Section).
- [x] **List**: Create `lithos-core/src/note/list.rs` (List, ListItem, ListType) per Spec 007.

### Phase 3: Task & Metadata - ✅ COMPLETE
**Goal**: Implement Task entity and Metadata container.
- [x] Create `lithos-core/src/note/task.rs` per Spec 007.
- [x] Implement `TaskMetadata` using `FieldValue`.
- [x] Implement `Task` struct (promoted entity).
- [x] Implement `Task::from_checkbox` (logic stub or full impl depending on Config availability).

### Phase 4: Frontmatter - ✅ COMPLETE
**Goal**: Refactor Frontmatter to use `FieldValue`.
- [x] Update `lithos-core/src/note/frontmatter.rs` per Spec 006.
- [x] Replace internal Map with `HashMap<String, FieldValue>`.
- [x] Implement strict accessors (`try_get`, etc.).
- [x] Implement leniency wrappers.

### Phase 5: Aggregate Root - ✅ COMPLETE
**Goal**: Assemble the `Note` aggregate.
- [x] Update `lithos-core/src/note/aggregate.rs` per Spec 004.
- [x] Update fields to `lists: Vec<List>`, `tasks: Vec<Task>`.
- [x] Ensure `NotePath` validation logic is present.
- [x] Ensure rkyv derives are correct.

### Phase 6: CQRS Ports & Errors - ✅ COMPLETE
**Goal**: Define the boundaries.
- [x] Update `lithos-core/src/note/error.rs` to include `FrontmatterError`, etc.
- [x] Update `lithos-core/src/note/ports.rs` with `Command` and `Query` traits.
- [x] Define `NoteCommandError` and `NoteQueryError` in `lithos-core/src/note/error.rs` (or `ports.rs` if circular deps arise).

### Phase 7: Infrastructure (CQRS Impl) - ✅ COMPLETE
**Goal**: Connect to DB.
- [x] Update `lithos-core/src/note/command.rs`.
- [x] Update `lithos-core/src/note/query.rs`.
- [x] Implement zero-copy helpers (`with_archived_by_id`).

### Phase 8: Verification - ✅ COMPLETE
- [x] Run `cargo test -p lithos-core --lib note`.
- [x] Verify no circular dependencies.
- [x] Verify rkyv derives compile.

### Phase 9: Lean & Performant Optimizations - 📋 PROPOSED
**Goal**: Address performance gaps and non-idiomatic patterns identified during review.

#### 9.1 GAT-based Ports (High Priority)
- [ ] Refactor `note/ports.rs` traits to use Generic Associated Types for zero-copy reads.
- [ ] Update `Query::find_by_id` to return `ArchivedGuard`.
- [ ] Run `mise run verify`.

#### 9.2 Zero-Copy Command Indexing (High Priority)
- [ ] Refactor `NoteCommand::delete` and `update` to use zero-copy `get` for old state.
- [ ] Remove `get_owned` calls in index cleanup logic.
- [ ] Verify performance improvement via benchmarks.

#### 9.3 Memory & Allocation Optimization (Medium Priority)
- [ ] Switch `FieldValue` to `Box<str>` for keys and string values.
- [ ] Refactor frontmatter helpers to return `Option<&str>` instead of owned `String`.
- [ ] Optimize `task.rs`: replace `format!` and `to_string()` in parsing loops with direct string slices and char matching.
- [ ] Optimize `tag.rs`: avoid `Vec` collection in `Segments::validate`.

#### 9.4 Micro-Refactors (Low Priority)
- [ ] Update constructors to use `impl Into<Box<str>>`.
- [ ] Optimize `NotePath::new` normalization.
- [ ] Clean up clippy suppressions in tests.

## Dependencies

- Requires `TaskConfig` from Config context (mockable for now if needed).
- Requires `pulldown-cmark` for parser adapters (separate from domain models).

## Rollout Strategy

1. Create git worktree `refactor/note-context`.
2. Execute phases sequentially.
3. Commit after each phase.
