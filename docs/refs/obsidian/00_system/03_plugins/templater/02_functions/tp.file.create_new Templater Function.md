---
title: tp.file.create_new Templater Function
aliases:
  - tp.file.create_new()
  - tp.file.create_new
  - The Templater tp.file.content Function
  - create_new
  - file.create_new
  - tp.file.create_new
  - templater_file.create_new
language:
  - javascript
plugin: templater
module: file
syntax: "tp.file.create_new(template: TFile ⎮ string, filename?: string, open_new: boolean = false, folder?: TFolder)"
url: https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfilecreate_newtemplate-tfile--string-filename-string-open_new-boolean--false-folder-tfolder
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/templater, obsidian/tp/file/create_new
---
# The Templater `tp.file.create_new()` Function

## Description

> [!function] Function Details
> 
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Module: File  
> Input::  
> Output::  
> Definition:: Creates a new file using a specified template or with a specified content.
>  
> Link: [tp.file.create_new](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/file-module.html#tpfilecreate_newtemplate-tfile--string-filename-string-open_new-boolean--false-folder-tfolder)

---

## Syntax

```javascript
tp.file.create_new(
	template: TFile ⎮ string, 
	filename?: string, 
	open_new: boolean = false, 
	folder?: TFolder)
```

## Parameter Values

| Parameter |      Type       | Description                                                                                                                                                                                                                               |
|:--------- |:---------------:|:----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| template  | TFile or string | Either the template used for the new file content, or the file content as a string. If it is the template to use, you retrieve it with `tp.file.find_tfile(TEMPLATENAME)`                                                                   |
| filename  |     string      | The filename of the new file, defaults to "Untitled".                                                                                                                                                                                     |
| open_new  |     boolean     | Whether to open or not the newly created file. Warning: if you use this option, since commands are executed asynchronously, the file can be opened first and then other commands are appended to that new file and not the previous file. |
| folder    |     TFolder     | The folder to put the new file in, defaults to obsidian's default location. If you want the file to appear in a different folder, specify it with `app.vault.getAbstractFileByPath("FOLDERNAME")`                                         |

## Additional Information

## Examples

```javascript
// File creation: 
await tp.file.create_new("MyFileContent", "MyFilename")).basename
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
