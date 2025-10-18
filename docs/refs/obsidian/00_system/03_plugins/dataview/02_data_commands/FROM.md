---
title: The Dataview FROM Data Command
aliases:
  - FROM
  - from
  - dataview_FROM
  - The Dataview FROM Data Command
language:
  - sql
plugin: dataview
module:
  - data_command
url: https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#from
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: sql, obsidian, obsidian/dataview/from, dv/query/from
---

tags:: #sql #obsidian #obsidian/dataview/from #dv/query/from

reference: [FROM](https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#from)

---

# `FROM`

## Description

> [!info]  
> Plugin: Dataview  
> Module: Data Commands  
> Definition:: The `FROM` statement determines what pages will initially be collected and passed onto the other commands for further filtering.

## Syntax

```sql
FROM #tag
FROM "folder"
FROM "path/to/file"
FROM [[note]]
FROM outgoing([[note]])
FROM #tag and "folder"
FROM #tag and -"folder"
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
|           |      |             |

## Additional Information

The `FROM` statement determines what pages will initially be collected and passed onto the other commands for further filtering. You can select from any [source](https://blacksmithgu.github.io/obsidian-dataview/queries/reference/sources), which currently means by folder, by tag, or by incoming/outgoing links.

- **Tags**: To select from a tag (and all its subtags), use `FROM #tag`.
- **Folders**: To select from a folder (and all its subfolders), use `FROM "folder"`.
- **Single Files**: To select from a single file, use `FROM "path/to/file"`.
- **Links**: You can either select links TO a file, or all links FROM a file.
- To obtain all pages which link TO `[[note]]`, use `FROM [[note]]`.
- To obtain all pages which link FROM `[[note]]` (i.e., all the links in that file), use `FROM outgoing([[note]])`.

You can compose these filters in order to get more advanced sources using `and` and `or`.

- For example, `#tag and "folder"` will return all pages in `folder` and with `#tag`.
- `[[Food]] or [[Exercise]]` will give any pages which link to `[[Food]]` OR `[[Exercise]]`.

You can also "negate" sources to obtain anything that does NOT match a source using `-`:

- `-#tag` will exclude files which have the given tag.
- `#tag and -"folder"` will only include files tagged `#tag` which are NOT in `"folder"`.

## Examples

```javascript

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
