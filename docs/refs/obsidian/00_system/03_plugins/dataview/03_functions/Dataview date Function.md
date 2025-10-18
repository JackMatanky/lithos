---
title: Dataview date Function
aliases:
  - date()
  - date
  - dataview_date()
  - date Dataview Function
  - The Dataview date() Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: constructor
syntax: "date(any)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#dateany
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/date, dv/function/date
---
# The Dataview `date()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Query Function  
> Class: Constructor  
> Input::  
> Output::  
> Definition:: Parses a date from the provided string, date, or link object, if possible, returning null otherwise.  
>  
> Link: [date](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#dateany)

---

## Syntax

```javascript
date(any)
date(text, format)
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
| value     |      |             |

## Additional Information

`date(text, format)` parses a date from text to luxon DateTime with the specified format. Note localised formats might not work. Uses [Luxon](https://moment.github.io/luxon/#/formatting?id=table-of-tokens) tokens.

## Examples

```js
date("2020-04-18") = <date object representing April 18th, 2020>
date([[2021-04-16]]) = <date object for the given page, refering to file.day>

date("12/31/2022", "MM/dd/yyyy") => DateTime for Decemeber 31th, 2022
date("210313", "yyMMdd") => DateTime for March 13th, 2021
date("946778645000", "x") => DateTime for "2000-01-02T03:04:05"
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
	AND contains(file.tags, "date")
SORT file.frontmatter.language, file.name
LIMIT 20
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
