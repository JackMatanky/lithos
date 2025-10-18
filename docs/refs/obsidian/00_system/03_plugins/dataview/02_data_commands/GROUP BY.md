---
title: The Dataview GROUP BY Data Command
aliases:
  - GROUP BY
  - where
  - dataview_GROUP BY
  - The Dataview GROUP BY Data Command
language:
  - sql
plugin: dataview
module:
  - data_command
url: https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#group-by
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: sql, obsidian, obsidian/dataview/group_by, dv/query/group_by
---

tags:: #sql #obsidian #obsidian/dataview/group_by #dv/query/group_by

reference: [GROUP BY](https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#group-by)

---

# `GROUP BY`

## Description

> [!info]  
> Plugin: Dataview  
> Module: Data Commands  
> Definition:: Group all results on a field.

## Syntax

```sql
GROUP BY field 
GROUP BY (computed_field) AS name
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
|           |      |             |

## Additional Information

Yields one row per unique field value, which has 2 properties:

- one corresponding to the field being grouped on, and
- a `rows` array field which contains all of the pages that matched.

In order to make working with the `rows` array easier, Dataview supports field "swizzling". If you want the field `test` from every object in the `rows` array, then `rows.test` will automatically fetch the `test` field from every object in `rows`, yielding a new array. You can then apply aggregation operators like `sum()` or `flat()` over the resulting array.

## Examples

```sql

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
