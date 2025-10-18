---
title: tp.file.rename Templater Function
aliases:
  - tp.file.rename()
  - tp.file.rename
  - The Templater tp.file.rename() Function
  - rename
  - file.rename
  - tp.file.rename
  - templater_file.rename
language:
  - javascript
plugin: templater
module: file
syntax: "tp.file.rename(new_title: string)"
url: https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfilerenamenew_title-string
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/templater, obsidian/tp/file/rename
---
# The Templater `tp.file.rename()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Module: File  
> Input::  
> Output::  
> Definition:: Renames the file (keeps the same file extension).  
>  
> Link: [tp.file.rename](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfilerenamenew_title-string)

---

## Syntax

```javascript
tp.file.rename(new_title: string)
```

## Parameter Values

| Parameter |  Type  | Description         |
|:--------- |:------:|:------------------- |
| new_title | string | The new file title. |

## Additional Information

## Examples

```javascript
// File Rename: 
await tp.file.rename("MyNewName");

// Append a "2": 
await tp.file.rename(tp.file.title + "2");
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
