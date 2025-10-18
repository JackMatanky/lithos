---
title: DataviewJS dv.list Function
aliases:
  - dv.list()
  - dv.list
  - dataviewjs_dv.list()
  - dv.list DataviewJS Function
  - The DataviewJS dv.list() Function
language:
  - javascript
plugin: dataview
module:
  - dataviewjs
class: dataviews
syntax: "dv.list(elements)"
url: https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvlistelements
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-07-05T11:33
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/dataviewjs/list, dvjs/function/list
---
# The DataviewJS `dv.list()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: DataviewJS  
> Class: Dataviews  
> Input:: Elements  
> Output:: Array  
> Definition:: Render a dataview list of elements.
>  
> Link: [dv.list](https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvtableheaders-elements)

---

## Syntax

```javascript
dv.list(elements)
```

## Parameter Values

| Parameter | Type  | Description       |
|:--------- |:-----:|:----------------- |
| elements  | array | an array of items |

## Additional Information

Render a dataview list of elements; accept both vanilla arrays and data arrays.

## Examples

```js
// list of 1, 2, 3
dv.list([1, 2, 3])

// list of all file names
dv.list(dv.pages().file.name)

// list of all file links
dv.list(dv.pages().file.link)

// list of all books with rating greater than 7
dv.list(dv.pages("#book").where(p => p.rating > 7))
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
	AND contains(file.tags, "table")
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
