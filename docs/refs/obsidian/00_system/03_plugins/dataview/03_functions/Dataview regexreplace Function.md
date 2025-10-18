---
title: Dataview regexreplace Function
aliases:
  - regexreplace()
  - regexreplace
  - dataview_regexreplace()
  - regexreplace Dataview Function
  - The Dataview regexreplace() Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: string_operation
syntax: "regexreplace(string, Pattern, replacement)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#regexreplacestring-pattern-replacement
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-06-05T19:17
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/regexreplace, dv/function/regexreplace, regex
---
# The Dataview `regexreplace()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Query Function  
> Class: String Operation  
> Input:: String  
> Output:: String  
> Definition:: Replaces all instances where the *regex* `pattern` matches in `string`, with `replacement`.
>  
> Link: [regexreplace](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#regexreplacestring-pattern-replacement)

---

## Syntax

```javascript
regexreplace(string, Pattern, replacement)
```

## Parameter Values

| Parameter   |  Type  | Description                                       |
|:----------- |:------:|:------------------------------------------------- |
| string      | string | The string to check                               |
| pattern     | string | RegEx string pattern                              |
| replacement |        | The string to replace with the matching substring |

## Additional Information

Replaces all instances where the *regex* `pattern` matches in `string`, with `replacement`. This uses the JavaScript replace method under the hood, so you can use special characters like `$1` to refer to the first capture group, and so on.

## Examples

```javascript
regexreplace("yes", "[ys]", "a") = "aea" 

regexreplace("Suite 1000", "\d+", "-") = "Suite -"
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
	Definition AS Definition
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
