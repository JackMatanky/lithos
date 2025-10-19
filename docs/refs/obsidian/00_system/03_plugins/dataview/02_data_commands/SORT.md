---
title: The Dataview SORT Data Command
aliases:
  - SORT
  - where
  - dataview_SORT
  - The Dataview SORT Data Command
language:
  - sql
plugin: dataview
module:
  - data_command
url: https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#sort
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: sql, obsidian, obsidian/dataview/sort, dv/query/sort
---

tags:: #sql #obsidian #obsidian/dataview/sort #dv/query/sort

reference: [SORT](https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#sort)

---

# `SORT`

## Description

> [!info]
> Plugin: Dataview
> Module: Data Commands
> Definition:: Sorts all results by one or more fields.

## Syntax

```sql
SORT field1 [ASCENDING/DESCENDING/ASC/DESC], …, fieldN [ASC/DESC]
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
|           |      |             |

## Additional Information

You can also give multiple fields to sort by. Sorting will be done based on the first field. Then, if a tie occurs, the second field will be used to sort the tied fields. If there is still a tie, the third sort will resolve it, and so on.

## Examples

```sql
SORT date [ASCENDING/DESCENDING/ASC/DESC]
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
