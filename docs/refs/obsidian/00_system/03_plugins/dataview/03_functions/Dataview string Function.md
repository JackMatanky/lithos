---
title: Dataview string Function
aliases:
  - string()
  - string
  - dataview_string()
  - string Dataview Function
  - The Dataview string() Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: constructor
syntax: "string(any)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#containsobjectliststring-value
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/string, dv/function/string
---
# The Dataview `string()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Query Function  
> Class: Constructor  
> Input::  
> Output::  
> Definition:: Converts any value into a reasonable string representation.
>  
> Link: [string](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#stringany)

---

## Syntax

```javascript
string(any)
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
| any       |      |             |

## Additional Information

Converts any value into a "reasonable" string representation. This sometimes produces less pretty results than just directly using the value in a query - it is mostly useful for coercing dates, durations, numbers, and so on into strings for manipulation.

## Examples

```js
string(18) = "18" 
string(dur(8 hours)) = "8 hours" 
string(date(2021-08-15)) = "August 15th, 2021"
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
	AND contains(file.tags, "string")
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
