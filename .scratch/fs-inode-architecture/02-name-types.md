---
title: 02-fs-name-types
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
