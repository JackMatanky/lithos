---
title: tp.system.suggester Templater Function
aliases:
  - tp.system.suggester()
  - tp.system.suggester
  - The Templater tp.system.suggester() Function
  - suggester
  - system.suggester
  - tp.system.suggester
  - templater_system.suggester
language:
  - javascript
plugin: templater
module: system
syntax: 'tp.system.suggester(text_items: string[] | ((item: T) => string), Items: T[], throw_on_cancel: Boolean = False, Placeholder: String = "", Limit?: Number = undefined)'
url: https://silentvoid13.github.io/Templater/internal-functions/internal-modules/system-module.html#tpsystemsuggestertext_items-string--item-t--string-items-t-throw_on_cancel-boolean--false-placeholder-stringlimit-number--undefined
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/templater, obsidian/tp/system/suggester
---
# The Templater `tp.system.suggester()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Module: System  
> Input::  
> Output::  
> Definition:: Spawns a suggester prompt and returns the user's chosen item.  
>  
> Link: [tp.system.suggester](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/system-module.html#tpsystemsuggestertext_items-string--item-t--string-items-t-throw_on_cancel-boolean--false-placeholder-string---limit-number--undefined)

---

## Syntax

```javascript
tp.system.suggester(
  text_items: string[] | ((item: T) => string), 
  Items: T[], 
  throw_on_cancel: Boolean = False, 
  Placeholder: String = "", 
  Limit?: Number = undefined
)
```

## Parameter Values

| Parameter       |  Type   | Description                                                                                                                                                                    |
|:--------------- |:-------:|:------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| text_items      | string  | Array of strings representing the text that will be displayed for each item in the suggester prompt. This can also be a function that maps an item to its text representation. |
| items           |  array  | Array containing the values of each item in the correct order.                                                                                                                 |
| throw_on_cancel | boolean | Throws an error if the prompt is canceled, instead of returning a `null` value                                                                                                 |
| placeholder     | string  | Placeholder string of the prompt                                                                                                                                               |
| limit           | number  | Limit the number of items rendered at once (useful to improve performance when displaying large lists)                                                                         |

## Additional Information

## Examples

```javascript
// Mood today: 
tp.system.suggester(
	["Happy", "Sad", "Confused"], 
	["Happy", "Sad", "Confused"]
);

// Picked file: 
await tp.system.suggester(
	(item) => item.basename, 
	app.vault.getMarkdownFiles()
)).basename
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
	AND contains(file.tags, "system")
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
