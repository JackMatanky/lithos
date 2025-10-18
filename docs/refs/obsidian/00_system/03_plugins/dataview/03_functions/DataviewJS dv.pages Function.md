---
title: DataviewJS dv.pages Function
aliases:
  - dv.pages()
  - dv.pages
  - dataview_dv.pages()
  - dv.pages DataviewJS Function
  - The DataviewJS dv.pages() Function
language:
  - javascript
plugin: dataview
module:
  - dataviewjs
class: query
syntax: "dv.pages(source)"
url: https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvpagessource
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: obsidian/dataview, obsidian/dataviewjs, obsidian/dataview/dataviewjs/dv_pages, dvjs/dv_pages
---
# The DataviewJS `dv.pages()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: DataviewJS  
> Class: Query  
> Input::  
> Output::  
> Definition:: Take a single string argument, `source`, and return a data array of page objects.  
>  
> Link: [dv.pages](https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvpagessource)

---

## Syntax

```javascript
dv.pages(source)
```

## Parameter Values

| Parameter |  Type  | Description      |
|:--------- |:------:|:---------------- |
| source    | string | pages or tags    |

## Additional Information

Take a single string argument, `source`, which is the same form as a [query language source](https://blacksmithgu.github.io/obsidian-dataview/reference/sources). Return a [data array](https://blacksmithgu.github.io/obsidian-dataview/api/data-array) of page objects, which are plain objects with all of the page fields as values.

Note that folders need to be double-quoted inside the string (i.e., `dv.pages("folder")` does not work, but `dv.pages('"folder"')` does) - this is to exactly match how sources are written in the query language.  

## Examples

```javascript
dv.pages() => all pages in your vault 
dv.pages("#books") => all pages with tag 'books' 
dv.pages('"folder"') => all pages from folder "folder" 
dv.pages("#yes or -#no") => all pages with tag #yes, or which DO NOT have tag #no dv.pages('"folder" or #tag') => all pages with tag #tag, or from folder "folder"
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
	AND contains(file.tags, "pages")
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
