---
title: 03-fs-format-types
category: enhancement
label: ready-for-agent
status: ready-for-agent
date_created: 2026-05-11
---

## Type

AFK

## Labels

- needs-triage

## What to build

Create fs/format.rs: public FileFormat enum (refactored from FormatKind) with expanded variants including Pdf, Image, Document, Archive, and FileExtensionRef<'a> borrowed view.

Make FileFormat public (was pub(crate) FormatKind). Add new format categories for multimap query support.

## Acceptance criteria

- [ ] FileFormat enum public (was pub(crate))
- [ ] Variants: Json, Toml, Yaml, Markdown, Image, Pdf, Document, Archive, Binary, Unknown
- [ ] Image: png, jpg, jpeg, gif, webp, svg, bmp, ico
- [ ] Document: doc, docx, odt, rtf, txt
- [ ] Archive: zip, tar, gz, rar, 7z, wasm
- [ ] FileExtensionRef<'a> borrowed view
- [ ] from_extension() detection
- [ ] is_markdown(), is_structured() helpers
- [ ] rkyv archived type support
- [ ] Tests for format detection coverage
- [ ] Update fs/mod.rs exports

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
