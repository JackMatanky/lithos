---
title: tp.file.content Templater Function
aliases:
  - tp.file.content
  - The Templater tp.file.content Function
  - content
  - file.content
  - tp.file.content
  - templater_file.content
language:
  - javascript
plugin: templater
module: file
syntax: "tp.file.content"
url: https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfilecontent
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/templater, obsidian/tp/file/content
---
# The Templater `tp.file.content` Function

## Description

> [!function] Function Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Module: File
> Input::
> Output::
> Definition:: Retrieves the file's content.
>
> Link: [tp.file.content](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfilecontent)

---

## Syntax

```javascript
 tp.file.content
```

## Parameter Values

| Parameter | Type | Description |
|:--------- |:----:|:----------- |
|           |      |             |

## Additional Information

## Examples

```javascript
// File content:
tp.file.content
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
	AND contains(file.tags, "file")
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
