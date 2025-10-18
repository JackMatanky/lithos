---
title: Dataview dateformat  Function
aliases:
  - dateformat ()
  - dateformat 
  - dataview_dateformat ()
  - dateformat  Dataview Function
  - The Dataview dateformat () Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: constructor
syntax: "dateformat(date|datetime, string)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#dateformatdatedatetime-string
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/dateformat, dv/function/dateformat
---
# The Dataview `dateformat ()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Query Function  
> Class: Constructor  
> Input::  
> Output::  
> Definition:: Format a Dataview date using [[Luxon]] tokens as formatting strings.
>  
> Link: [date](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#dateany)

---

## Syntax

```javascript
dateformat(date|datetime, string)
```

## Parameter Values

| Parameter        |  Type  | Description                          |
|:---------------- |:------:|:------------------------------------ |
| date or datetime |        |                                      |
| format           | string | String comprised of [[Luxon]] tokens |

## Additional Information

`date(text, format)` parses a date from text to luxon DateTime with the specified format. Note localised formats might not work. Uses [Luxon](https://moment.github.io/luxon/#/formatting?id=table-of-tokens) tokens.

## Examples

```js
dateformat(file.ctime,"yyyy-MM-dd") = "2022-01-05"
dateformat(file.ctime,"HH:mm:ss") = "12:18:04"
dateformat(date(now),"x") = "1407287224054"
dateformat(file.mtime,"ffff") = "Wednesday, August 6, 2014, 1:07 PM Eastern Daylight Time"
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
