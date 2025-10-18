---
title: The Dataview FLATTEN Data Command
aliases:
  - FLATTEN
  - flatten
  - dataview_FLATTEN
  - The Dataview FLATTEN Data Command
language:
  - sql
plugin: dataview
module:
  - data_command
class:
syntax: "FLATTEN field"
url: https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#flatten
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: sql, obsidian, obsidian/dataview/flatten, dv/query/flatten, data_type/array, array/reshape
---
# The Dataview `FLATTEN` Data Command

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Data Commands  
> Class:  
> Input:: Array  
> Output::  
> Definition:: Flatten an array in every row, yielding one result row per entry in the array.
>  
> Link: [FLATTEN](https://blacksmithgu.github.io/obsidian-dataview/queries/data-commands/#flatten)

---

## Syntax

```sql
FLATTEN field
FLATTEN (computed_field) AS name
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
|           |      |             |

## Additional Information

`FLATTEN` makes it easier to operate on nested lists since you can then use simpler where conditions on them as opposed to using functions like `map()` or `filter()`.

## Examples

For example, flatten the `authors` field in each literature note to give one row per author:

```sql
```dataview 
TABLE authors 
FROM #LiteratureNote  
FLATTEN authors  
```

| File                                           | authors             |
| ---------------------------------------------- | ------------------- |
| stegEnvironmentalPsychologyIntroduction2018 SN | Steg, L.            |
| stegEnvironmentalPsychologyIntroduction2018 SN | Van den Berg, A. E. |
| stegEnvironmentalPsychologyIntroduction2018 SN | De Groot, J. I. M.  |
| Soap Dragons SN                                | Robert Lamb         |
| Soap Dragons SN                                | Joe McCormick       |
| smithPainAssaultSelf2007 SN                    | Jonathan A. Smith   |
| smithPainAssaultSelf2007 SN                    | Mike Osborn         |

A good use of this would be when there is a deeply nested list that you want to use more easily. For example, `file.lists` or `file.tasks`. Note the simpler query though the end results are slightly different (grouped vs non-grouped). You can use a `GROUP BY file.link` to achieve identical results but would need to use `rows.T.text` as described earlier.

```sql
```dataview
TABLE
T.text as "Task Text"
FROM "Scratchpad"
FLATTEN file.tasks as T
WHERE T.text
```

```sql
```dataview
TABLE 
  filter(file.tasks.text, (t) => t) as "Task Text"
FROM "Scratchpad"
WHERE file.tasks.text
```

## Notes and Remarks

---

## Related

### Snippets (Use Cases)

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE 
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Functions

#### By Plugin

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.module AS Module,
	Definition AS Definition
WHERE 
	file.name != this.file.name
	AND (file.frontmatter.file_class = "pkm_code_function")
	AND (file.frontmatter.plugin = this.file.frontmatter.plugin)
SORT file.frontmatter.module, file.name
```

#### By Tag

<!-- Add tags in contains function as needed  -->  
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition,
	string(file.frontmatter.language) AS Language,
	sort(file.etags) AS Tags
WHERE 
	file.name != this.file.name
	AND file.frontmatter.file_class = "pkm_code_function"
	AND contains(file.tags, "flatten")
SORT file.frontmatter.language, file.name
LIMIT 10
```

#### Outgoing Function Links

<!-- Link related functions here -->

#### All Function Links

<!-- Excluding functions of the same module  -->  
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.definition AS Definition
WHERE 
	file.name != this.file.name
	AND file.frontmatter.module != this.file.frontmatter.module 
	AND file.frontmatter.file_class = "pkm_code_function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
