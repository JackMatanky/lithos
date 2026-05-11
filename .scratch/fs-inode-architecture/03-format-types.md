---
title: 03-fs-format-types
category: enhancement
label: needs-triage
status: pending
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
