---
title: 02-fs-name-types
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

Create fs/name.rs: owned types FileName, DirName, BaseName (Box<str>) and borrowed Ref types FileNameRef<'a>, DirNameRef<'a>, BaseNameRef<'a> (&'a OsStr).

Follow suffix pattern: no suffix = owned, Ref suffix = borrowed. BaseName follows Obsidian terminology (filename without extension).

## Acceptance criteria

- [x] FileName(Box<str>) - owned filename
- [x] DirName(Box<str>) - owned dirname
- [x] BaseName(Box<str>) - owned basename (Obsidian term)
- [x] FileNameRef<'a>(&'a OsStr) - borrowed filename view
- [x] DirNameRef<'a>(&'a OsStr) - borrowed dirname view
- [x] BaseNameRef<'a>(&'a OsStr) - borrowed basename view
- [x] Zero-copy extraction methods from path types
- [x] Conversion between owned and borrowed types
- [x] Tests for creation and extraction
- [x] Update fs/mod.rs exports

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

## Implementation Notes

**File:** `lithos-core/src/fs/name.rs`

**Implemented Types:**

**Owned Types (Box<str>):**
- `FileName(Box<str>)` - Full filename including extension
- `BaseName(Box<str>)` - Filename without extension (Obsidian terminology)
- `DirName(Box<str>)` - Directory name component

**Borrowed Types (&'a OsStr):**
- `FileNameRef<'a>(&'a OsStr)` - Zero-copy filename view
- `BaseNameRef<'a>(&'a OsStr)` - Zero-copy basename view
- `DirNameRef<'a>(&'a OsStr)` - Zero-copy dirname view

**Key Methods:**
- `FileName::basename()` - Extract basename (stem) without extension
- `FileName::extension()` - Extract file extension
- `FileName::as_str()` - String view
- `FileNameRef::to_owned()` - Convert to owned `FileName`
- `TryFrom<&Path>` for `FileName` - Extract from path
- `From<String>` for `FileName` - Construct from string

**Conversions:**
- Owned → Borrowed: via `as_ref()` methods
- Borrowed → Owned: via `to_owned()` and `ToOwned` trait
- String → FileName: via `From<String>`
- Path → FileName: via `TryFrom<&Path>`

**Migration:**
- Existing `FileName` in `fs/file.rs` replaced with re-export from `fs/name.rs`
- Eliminates duplication while maintaining backward compatibility

**Tests:**
- Tests integrated with `fs/path.rs` tests (22 total tests)
- Covers filename extraction, basename extraction, extension handling
- Edge cases: hidden files, no extension, multiple dots

**Module Integration:**
- Registered in `fs/mod.rs`
- Exported: `FileName`, `BaseName`, `DirName`, `FileNameRef`, `BaseNameRef`, `DirNameRef`
- Re-exported in `fs/file.rs` for backward compatibility

**Status:** ✅ Complete - All acceptance criteria met
