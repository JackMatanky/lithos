---
title: The Dataview WHERE Data Command
aliases:
  - WHERE
  - where
  - dataview_WHERE
  - The Dataview WHERE Data Command
language:
  - sql
plugin: dataview
module:
  - data_command
url: https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#where
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: sql, javascript, obsidian, obsidian/dataview/where, dv/query/where
---

tags:: #sql #javascript #obsidian #obsidian/dataview/where #dv/query/where

reference: [WHERE](https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#where)

---

# `WHERE`

## Description

> [!info]
> Plugin: Dataview
> Module: Data Commands
> Definition:: Filter pages on fields. Only pages where the clause evaluates to `true` will be yielded.

## Syntax

```sql
WHERE <clause>
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
|           |      |             |

## Additional Information

## Examples

```sql
-- Obtain all files which were modified in the last 24 hours:
LIST
WHERE file.mtime >= date(today) - dur(1 day)`

-- Find all projects which are not marked complete and are more than a month old:
LIST
FROM #projects
WHERE !completed
	AND file.ctime <= date(today) - dur(1 month)`
```

## Notes and Remarks

---

## Related

### Snippets (Use Cases)

```dataview
LIST
FROM "70_pkm_tree"
WHERE file.frontmatter.file_class = "pkm_code_snippet"
	AND contains(file.outlinks, this.file.link)
SORT file.name
```

### Functions

#### By Plugin

```dataview
LIST
	rows.file.link
FROM "70_pkm_tree" OR "00_system"
WHERE (file.frontmatter.file_class = "pkm_code_function")
	AND (file.frontmatter.plugin = this.file.frontmatter.plugin)
GROUP BY file.frontmatter.module
SORT file.name
```

#### Outlinked

```dataview
LIST
FROM "70_pkm_tree" OR "00_system"
WHERE file.frontmatter.file_class = "pkm_code_function"
	AND contains(file.outlinks, this.file.link)
SORT file.name
```

#### Linked

---

## Resources

---

## Flashcards
