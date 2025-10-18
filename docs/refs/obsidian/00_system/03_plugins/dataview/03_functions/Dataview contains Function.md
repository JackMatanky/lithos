---
title: Dataview contains Function
aliases:
  - contains()
  - contains
  - dataview_contains()
  - contains Dataview Function
  - The Dataview contains() Function
language:
  - javascript
plugin: dataview
module:
  - query_function
class: object_array_string_operation
syntax: "contains(object|list|string, value)"
url: https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#containsobjectliststring-value
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/contains, dv/function/contains
---
# The Dataview `contains()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Dataview]]  
> Language: [[JavaScript]]  
> Module: Query Function  
> Class: Object, Array, and String Operation  
> Input::  
> Output::  
> Definition:: Checks if the given container type has the given value in it.
>  
> Link: [contains](https://blacksmithgu.github.io/obsidian-dataview/reference/functions/#containsobjectliststring-value)

---

## Syntax

```javascript
contains(object|list|string, value)
```

## Parameter Values

| Parameter |          Type           | Description |
|:--------- |:-----------------------:|:----------- |
| arg1      | object, list, or string |             |
| value     |                         |             |

## Additional Information

This function behave slightly differently based on whether the first argument is an object, a list, or a string. This function is case-sensitive.

## Examples

- For objects, checks if the object has a key with the given name. For example,

```js
contains(file, "ctime") = true
contains(file, "day") = true (if file has a date in its title, false otherwise)
    ```

- For lists, checks if any of the array elements equals the given value. For example,

```js
contains(list(1, 2, 3), 3) = true
contains(list(), 1) = false
```

- For strings, checks if the given value is a substring (i.e., inside) the string.

```js
contains("hello", "lo") = true
contains("yes", "no") = false
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
	AND contains(file.tags, "contains")
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
