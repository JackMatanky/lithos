---
title: Dataview regexmatch Function
aliases:
  - regexmatch()
  - regexmatch
  - dataview_regexmatch()
  - regexmatch Dataview Function
  - The Dataview regexmatch() Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: string_operation
syntax: "regexmatch(pattern, string)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#regexmatchpattern-string
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-06-05T19:17
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/regexmatch, dv/function/regexmatch, regex
---
# The Dataview `regexmatch()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Query Function  
> Class: String Operation  
> Input::  
> Output:: Boolean  
> Definition:: Checks if the given regex pattern matches the entire string, using the JavaScript regex engine.  
>  
> Link: [regexmatch](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#regexmatchpattern-string)

---

## Syntax

```javascript
regexmatch(pattern, string)
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
| pattern   |      |             |
| string    |      |             |

## Additional Information

- `regexmatch` differs from [[Dataview regextest Function|regextest()]] in that `regextest` can match just parts of the text while `regexmatch` must match the entire string.  

## Examples

```javascript
regexmatch("\w+", "hello") = true
regexmatch(".", "a") = true
regexmatch("yes|no", "maybe") = false
regexmatch("what", "what's up dog?") = false
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
	AND (contains(file.tags, "string")
	OR contains(file.tags, "regex"))
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
