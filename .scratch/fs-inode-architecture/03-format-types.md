---
title: 03-fs-format-types
category: enhancement
label: completed
status: completed
date_created: 2026-05-11
date_completed: 2026-05-12
---

## Type

AFK

## Labels

- completed

## What to build

Create fs/format.rs: public FileFormat enum (refactored from FormatKind) with expanded variants including Pdf, Image, Document, Archive, and FileExtensionRef<'a> borrowed view.

Make FileFormat public (was pub(crate) FormatKind). Add new format categories for multimap query support.

## Acceptance criteria

- [x] FileFormat enum public (was pub(crate))
- [x] Variants: Json, Toml, Yaml, Markdown, Image, Pdf, Document, Archive, Binary, Unknown
- [x] Image: png, jpg, jpeg, gif, webp, svg, bmp, ico
- [x] Document: doc, docx, odt, rtf, txt
- [x] Archive: zip, tar, gz, rar, 7z, wasm
- [x] FileExtensionRef<'a> borrowed view
- [x] from_extension() detection
- [x] is_markdown(), is_structured() helpers
- [x] rkyv archived type support
- [x] Tests for format detection coverage
- [x] Update fs/mod.rs exports

## Blocked by

None - can start immediately

## Agent Brief

**Category:** enhancement
**Summary:** Refactor `FormatKind` into a public `FileFormat` enum with expanded support for rich media and archives.

**Current behavior:**
The `FormatKind` enum is internal to the filesystem module (`pub(crate)`) and only supports `Json`, `Toml`, `Yaml`, and `Markdown`. This prevents other modules (like the vault storage layer) from performing rich queries based on file type (e.g., "list all images").

**Desired behavior:**
Promote and rename `FormatKind` to a public `FileFormat` enum. Expand its variants to include broad categories like `Image`, `Pdf`, `Document`, and `Archive`. Implement robust detection from file extensions and include a borrowed `FileExtensionRef` view.

**Key interfaces:**
- `FileFormat` — public enum with variants: `Json`, `Toml`, `Yaml`, `Markdown`, `Image`, `Pdf`, `Document`, `Archive`, `Binary`, `Unknown`
- `FileExtensionRef<'a>` — borrowed `&'a OsStr` view of the file extension
- `FileFormat::from_extension()` — the primary detection logic

**Acceptance criteria:**
- [ ] `FileFormat` is public and `rkyv`-enabled for zero-copy deserialization
- [ ] `Image` variant covers: `png`, `jpg`, `jpeg`, `gif`, `webp`, `svg`, `bmp`, `ico`
- [ ] `Document` variant covers: `doc`, `docx`, `odt`, `rtf`, `txt`
- [ ] `Archive` variant covers: `zip`, `tar`, `gz`, `rar`, `7z`, `wasm`
- [ ] `is_markdown()` and `is_structured()` helper methods are implemented
- [ ] Tests verify correct detection for all supported extensions, including case-insensitivity

**Out of scope:**
- Content-based detection (magic bytes)
- Migrating existing `FormatKind` consumers (reserved for Issue 09)

## Implementation Notes

**File:** `lithos-core/src/fs/format.rs`

**Implemented Types:**

**FileFormat enum (public, rkyv-enabled):**
```rust
pub enum FileFormat {
    Json,        // .json
    Toml,        // .toml
    Yaml,        // .yaml, .yml
    Markdown,    // .md, .markdown
    Image,       // png, jpg, jpeg, gif, webp, svg, bmp, ico
    Pdf,         // .pdf
    Document,    // doc, docx, odt, rtf, txt
    Archive,     // zip, tar, gz, rar, 7z, wasm
    Binary,      // fallback for other binary formats
    Unknown,     // unrecognized extension
}
```

**FileExtensionRef<'a>:**
- Zero-copy borrowed view wrapping `&'a OsStr`
- Used for extension extraction without allocation

**Key Methods:**
- `FileFormat::from_extension(ext: &OsStr) -> FileFormat` - Primary detection logic
- `FileFormat::is_markdown() -> bool` - Check if Markdown variant
- `FileFormat::is_structured() -> bool` - Check if Json/Toml/Yaml
- Case-insensitive extension matching

**Supported Extensions:**
- **Image:** png, jpg, jpeg, gif, webp, svg, bmp, ico
- **Document:** doc, docx, odt, rtf, txt
- **Archive:** zip, tar, gz, rar, 7z, wasm
- **Structured:** json, toml, yaml, yml
- **Markdown:** md, markdown

**Backward Compatibility:**
- `FileFormat` re-exported as `FormatKind` in `fs/types.rs`
- Maintains existing API surface during Phase 1
- Migration to new name deferred to Issue 09

**Integration:**
- Updated `FsReader::classify_path()` to use `FileFormat::from_extension()`
- Updated non-exhaustive pattern matches in `fs/reader.rs` to handle new variants
- Fixed test expectations: `Binary` → `Image`/`Pdf` for respective extensions

**Tests:** 2 new tests:
- `should_detect_various_formats` - Verifies all format categories
- `should_identify_structured_formats` - Verifies Json/Toml/Yaml detection

**Module Integration:**
- Registered in `fs/mod.rs`
- Exported: `FileFormat`, `FileExtensionRef`
- Backward compat: `FormatKind` alias in `fs/types.rs`

**Status:** ✅ Complete - All acceptance criteria met
