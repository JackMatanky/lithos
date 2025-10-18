---
title: 52_00_task_move_file_to_directory
aliases:
  - Move Task File to Directory
  - Move Task File to Correct Directory
  - move_task_file_to_directory
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
# Move Task File to Directory

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Check the task file's folder path and if the file is in the wrong directory, move it to the correct folder.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>TODO: DEFINE <projects_dir> VARIABLE<<<
// >>>TODO: DEFINE <parent_task> VARIABLE<<<
// >>>TODO: DEFINE <folder_path> VARIABLE<<<
//---------------------------------------------------------
// MOVE FILE TO DIRECTORY
//---------------------------------------------------------
let directory;

if (parent_task == `null`) {
  directory = `${project_dir}`;
} else {
  directory = `${project_dir}${parent_task}/`;
}

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
// >>>TODO: DEFINE <projects_dir> VARIABLE<<<
// >>>TODO: DEFINE <parent_task> VARIABLE<<<
// >>>TODO: DEFINE <folder_path> VARIABLE<<<
//---------------------------------------------------------
// MOVE FILE TO DIRECTORY
//---------------------------------------------------------
let directory;
if (parent_task == `null`) {
  directory = `${project_dir}`;
} else {
  directory = `${project_dir}${parent_task}/`;
}
if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[52_00_task_event]]
2. [[53_00_action_item]]
3. [[54_00_meeting]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[task_context_project_by_path_or_suggester|Set Project by Path or Suggester]]
2. [[task_parent_task_by_path_or_suggester|Set Parent Task by Path or Suggester]]
3. [[52_00_task_titles_alias_file_name|Task Titles, Alias, and File Name]]

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
