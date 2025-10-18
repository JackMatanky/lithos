---
title: The Dataview LIMIT Data Command
aliases:
  - LIMIT
  - limit
  - dataview_LIMIT
  - The Dataview LIMIT Data Command
language:
  - sql
plugin: dataview
module:
  - data_command
url: https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#limit
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: sql, obsidian, obsidian/dataview/limit, dv/query/limit
---

tags:: #sql #obsidian #obsidian/dataview/limit #dv/query/limit

reference: [LIMIT](https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#limit)

---

# `LIMIT`

## Description

> [!info]  
> Plugin: Dataview  
> Module: Data Command  
> Definition:: Restrict the results to at most N values.

## Syntax

```sql
LIMIT n
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
|           |      |             |

## Additional Information

Commands are processed in the order they are written, so the following sorts the results *after* they have already been limited:

```
LIMIT 5
SORT date ASCENDING
```

## Examples

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
