---
title: Dataview object Function
aliases:
  - object()
  - object
  - dataview_object()
  - object Dataview Function
  - The Dataview object() Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: constructor
syntax: "object(key1, value1, …)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#objectkey1-value1
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/object, dv/function/object
---
# The Dataview `object()` Function

## Description

> [!function] Function Details
>
> Plugin: [[Dataview]]
> Language: [[JavaScript]]
> Module: Query Function
> Class: Constructor
> Input::
> Output::
> Definition:: Creates a new object with the given keys and values.
>
> Link: [object](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#objectobjectliststring-value)

---

## Syntax

```javascript
object(key1, value1, …)
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
| key       |      |             |
| value     |      |             |

## Additional Information

Keys and values should alternate in the call, and keys should always be strings/text.

## Examples

```js
object() => empty object
object("a", 6) => object which maps "a" to 6
object("a", 4, "c", "yes") => object which maps a to 4, and c to "yes"
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
	AND contains(file.tags, "object")
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
