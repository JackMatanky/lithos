---
title: tp.file.include Templater Function
aliases:
  - tp.file.include()
  - tp.file.include
  - The Templater tp.file.include() Function
  - include
  - file.include
  - tp.file.include
  - templater_file.include
language:
  - javascript
plugin: templater
module: file
syntax: "tp.file.include(include_link: string ⎮ TFile)"
url: https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfileincludeinclude_link-string--tfile
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/templater, obsidian/tp/file/include
---
# The Templater `tp.file.include()` Function

## Description

> [!function] Function Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Module: File
> Input::
> Output::
> Definition:: Includes the file's link content. Templates in the included content will be resolved.
>
> Link: [tp.file.include](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfileincludeinclude_link-string--tfile)

---

## Syntax

```javascript
tp.file.include(include_link: string ⎮ TFile)
```

## Parameter Values

| Parameter       |      Type       | Description                                                                                                                                |
|:--------------- |:---------------:|:------------------------------------------------------------------------------------------------------------------------------------------ |
| include_link    | Tfile or string | The link to the file to include, e.g. \[\[MyFile]], or a TFile object. Also supports sections or blocks inclusions, e.g. \[\[MyFile#Section1]] |

## Additional Information

## Examples

```javascript
// File include:
tp.file.include("[[Template1]]")

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
