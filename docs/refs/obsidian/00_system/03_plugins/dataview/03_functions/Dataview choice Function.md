---
title: Dataview choice Function
aliases:
  - choice()
  - choice
  - dataview_choice()
  - choice Dataview Function
  - The Dataview choice() Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: utility_function
syntax: "choice(bool, left, right)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#choicebool-left-right
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-06-06T20:08
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/choice, dv/function/choice, conditional_logic
---
# The Dataview `choice()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Query Function  
> Class: Utility Function  
> Input:: boolean  
> Output::  
> Definition:: A primitive if statement. If the first argument is truthy, returns left; otherwise, returns right.
>  
> Link: [choice](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#choicebool-left-right)

---

## Syntax

```javascript
choice(bool, left, right)
```

## Parameter Values

| Parameter |  Type   | Description                       |
|:--------- |:-------:|:--------------------------------- |
| bool      | boolean | Either true, false, or comparison |
| left      |  value  |                                   |
| right     |  value  |                                   |

## Additional Information

A primitive if statement - if the first argument is truthy, returns left; otherwise, returns right.

## Examples

```js
choice(true, "yes", "no") = "yes"
choice(false, "yes", "no") = "no"
choice(x > 4, y, z) = y if x > 4, else z
```

- assign task type

```javascript
choice(contains(T.text, "_action_item"), "Action Item", choice(contains(T.text, "_meeting"), "Meeting", choice(contains(T.text, "_habit"), "Habit", choice(contains(T.text, "_morning_ritual"), "Morn Rit.", choice(contains(T.text, "_workday_startup_ritual"), "Work Start Rit.", choice(contains(T.text, "_workday_shutdown_ritual"), "Work End Rit.", "Eve Rit.")))))) AS Type,
```

```javascript
choice(contains(T.text, "_action_item"), "🔨", choice(contains(T.text, "_meeting"), "🤝", choice(contains(T.text, "_habit"), "🤖", choice(contains(T.text, "_morning_ritual"), "🍵", choice(contains(T.text, "_workday_startup_ritual"), "🌇", choice(contains(T.text, "_workday_shutdown_ritual"), "🌆", "🛌")))))) AS Type,
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
	string(file.frontmatter.module) AS Module,
	Definition AS Definition
WHERE 
	file.name != this.file.name
	AND (file.frontmatter.file_class = "pkm_code_function")
	AND (file.frontmatter.plugin = this.file.frontmatter.plugin)
SORT file.frontmatter.module, file.name
```

#### By Tag

<!-- Add tags in choice function as needed  -->  
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
	AND (contains(file.tags, "choice")
	OR contains(file.tags, "boolean"))
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
	AND (choice(file.outlinks, this.file.link)
	OR choice(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
