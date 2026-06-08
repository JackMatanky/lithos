# Issue 01: Indexer scaffolding

**Status**: ready-for-agent
**Created**: 2026-06-09

## What to build

Introduce the `lithos-core::indexer` module as an empty-but-compiling bounded
context. Add the corresponding `CONTEXT.md` glossary stub and a `(planned)`
entry in `CONTEXT-MAP.md`. Update the `CONTEXT-MAP.md` Global Invariants to
reflect that filesystem scanning is now performed via the Indexer's
`ScannerPort`, not the FS context's `DirScanner`.

The deliverable is a compilable module boundary with correct documentation
anchors — no domain logic yet.

## Acceptance criteria

- [ ] `lithos-core/src/indexer/mod.rs` exists and is declared in
      `lithos-core/src/lib.rs`; the crate compiles with no warnings.
- [ ] `lithos-core/src/indexer/CONTEXT.md` exists with at minimum stub
      entries for: Filesystem Node, File Node, Directory Node, Index Scope,
      Index Status, Indexed Node, Deleted Node, Scanner Port.
- [ ] `CONTEXT-MAP.md` contains a `(planned)` entry for the Indexer context
      pointing at `lithos-core/src/indexer/CONTEXT.md`.
- [ ] `CONTEXT-MAP.md` Global Invariants no longer list `DirScanner` as the
      sole scanning mechanism; the entry is updated to reflect `ScannerPort`.
- [ ] `CONTEXT-MAP.md` Relationships section includes
      `Config -> Indexer (planned)` and
      `Indexer -> Schema, Note, Template (planned)`.
- [ ] All existing tests pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

None — can start immediately.
