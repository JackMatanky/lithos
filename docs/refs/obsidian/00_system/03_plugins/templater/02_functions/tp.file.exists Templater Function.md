---
title: tp.file.exists Templater Function
aliases:
  - tp.file.exists()
  - tp.file.exists
  - The Templater tp.file.exists() Function
  - exists
  - file.exists
  - tp.file.exists
  - templater_file.exists
language:
  - javascript
plugin: templater
module: file
syntax: "tp.file.exists(filename: string)"
url: https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfileexistsfilename-string
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/templater, obsidian/tp/file/exists
---
# The Templater `tp.file.exists()` Function

## Description

> [!function] Function Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Module: File
> Input::
> Output::
> Definition:: The filename of the file we want to check existence. The fullpath to the file, relative to the Vault and containing the extension, must be provided. e.g. MyFolder/SubFolder/MyFile.
>
> Link: [tp.file.exists](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfileexistsfilename-string)

---

## Syntax

```javascript
tp.file.exists(filename: string)
```

## Parameter Values

| Parameter |  Type  | Description                                                       |
|:--------- |:------:|:----------------------------------------------------------------- |
| filename  | string | The filename of the file we want to check existence, e.g. MyFile. |

## Additional Information

## Examples

```javascript
// File existence:
await tp.file.exists("MyFolder/MyFile.md")

// File existence of current file:
await tp.file.exists(tp.file.folder(true)+"/"+tp.file.title+".md")
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
