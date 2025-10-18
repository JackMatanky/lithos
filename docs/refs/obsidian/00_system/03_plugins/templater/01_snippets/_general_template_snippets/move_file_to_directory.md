---
title: move_file_to_directory
aliases:
  - Move File to Directory
  - Move File to Correct Directory
  - move file to directory
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T15:10
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/folder
---
# Move File to Correct Directory

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Check the file's folder path and if the file is in the wrong directory, move it to the correct folder.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// TODO: Define <directory> variable, include trailing backslash
// TODO: Define <file_name> variable
//---------------------------------------------------------  
// MOVE FILE TO DIRECTORY
//---------------------------------------------------------
const directory = <directory>
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {  
   await tp.file.move(`${directory}${file_name}`);  
};
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// MOVE FILE TO DIRECTORY
//---------------------------------------------------------
const directory = <directory>
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {  
   await tp.file.move(`${directory}${file_name}`);  
};
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[25_10_daily_reflection]]
2. [[25_12_daily_reflection_today_preset]]
3. [[61_contact]]
4. [[62_organization]]
5. [[90_00_note]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[journal_titles_alias_and_file_name|Journal Titles, Alias, and File Name]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

### All Snippet Links

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

### Outgoing Function Links

<!-- Link related functions here -->

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.definition AS Definition
WHERE 
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
