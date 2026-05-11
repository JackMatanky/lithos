---
title: 02-fs-name-types
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

Create fs/name.rs: owned types FileName, DirName, BaseName (Box<str>) and borrowed Ref types FileNameRef<'a>, DirNameRef<'a>, BaseNameRef<'a> (&'a OsStr).

Follow suffix pattern: no suffix = owned, Ref suffix = borrowed. BaseName follows Obsidian terminology (filename without extension).

## Acceptance criteria

- [ ] FileName(Box<str>) - owned filename
- [ ] DirName(Box<str>) - owned dirname
- [ ] BaseName(Box<str>) - owned basename (Obsidian term)
- [ ] FileNameRef<'a>(&'a OsStr) - borrowed filename view
- [ ] DirNameRef<'a>(&'a OsStr) - borrowed dirname view
- [ ] BaseNameRef<'a>(&'a OsStr) - borrowed basename view
- [ ] Zero-copy extraction methods from path types
- [ ] Conversion between owned and borrowed types
- [ ] Tests for creation and extraction
- [ ] Update fs/mod.rs exports

## Blocked by

None - can start immediately

## Agent Brief

**Category:** enhancement
**Summary:** Create owned and borrowed filename types with support for Obsidian-style "basename" terminology.

**Current behavior:**
Filenames are handled using generic `OsStr` or `String` types. This leads to ambiguity about whether a "name" includes an extension, and often results in unnecessary allocations when extracting just the name or the stem from a path.

**Desired behavior:**
Implement a suite of types for owned and borrowed name components following the `Ref` suffix pattern for borrowed views. Specifically, support "BaseName" which follows Obsidian terminology (the filename without its extension), as this is a core domain concept for wikilink resolution.

**Key interfaces:**
- `FileName`, `DirName`, `BaseName` — owned `Box<str>` representations
- `FileNameRef<'a>`, `DirNameRef<'a>`, `BaseNameRef<'a>` — borrowed `&'a OsStr` views
- Extraction methods on `FilePath` and `DirPath` to return these types zero-copy where possible

**Acceptance criteria:**
- [ ] All owned types use `Box<str>` for space efficiency
- [ ] All borrowed types wrap `&OsStr` for zero-copy views
- [ ] `BaseName` correctly extracts the file stem (Obsidian "basename")
- [ ] Suffix pattern is strictly followed: no suffix = owned, `Ref` suffix = borrowed
- [ ] Conversions between owned and borrowed types are implemented (`ToOwned`, `From`, etc.)
- [ ] Tests verify correct extraction from various path strings (with/without dots, hidden files, etc.)

**Out of scope:**
- Path validation (reserved for Issue 01)
- Extension-specific logic (reserved for Issue 03)
