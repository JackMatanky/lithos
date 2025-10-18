---
title: task_parent_task_by_path_or_suggester
aliases:
  - Parent Task by Path or Suggester
  - Set Parent Task
  - Set Parent Task by Path or Suggester
plugin: templater
language:
  - javascript
module:
  - system
  - file
  - user
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-01T13:41
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/folder
---
# Parent Task by Path or Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: If a task is already in a parent task's folder, set the task's parent task based on the folder path; otherwise, set the parent task with the suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
const projects_dir = `40_projects/`;

//---------------------------------------------------------
// CHECK FILE LOCATION AND ASSIGN RELEVANT VARIABLES
//---------------------------------------------------------
// Get file's folder's relative path
const folder_path = `${tp.file.folder(true)}/`;
// Split the folder's path by backslash
const folder_path_split = folder_path.split(`/`);
// Get the split folder path array's length
const folder_path_length = folder_path_split.length;

//---------------------------------------------------------
// SET PARENT TASK BY FILE PATH OR SUGGESTER
//---------------------------------------------------------
// Initialize the task's parent task variable
let parent_task_obj;
let parent_task_value;
let parent_task_name;

// Check if the parent directory equals projects_dir, 40_projects/
// and the folder path array's length is equal to or greater than four
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 4) {
  // If true, assign the project to 
  // the split folder path array's fourth element
  parent_task_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[3],
    file_class: "task",
    type: "parent_task",
  });
  parent_task_value = parent_task_obj[1].value;
  parent_task_name = parent_task_obj[1].key;
} else {
  // If false, return an array of parent task folder names
  // filtered by the project
  // TODO: DEFINE PROJECT VARIABLE
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_value,
    file_class: "task",
    type: "parent_task",
  });

  // Choose a parent task from the array of parent task folder names
  parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    "Parent Task?"
  );
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
};

const parent_task_link = `${parent_task_value}|${parent_task_name}`;
const parent_task_value_link = `${new_line}${ul_yaml}"${parent_task_link}"`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
// >>>TODO: DEFINE <projects_dir> VARIABLE<<<
// >>>TODO: DEFINE <project_value> VARIABLE<<<
//---------------------------------------------------------
// FILE PATH VARIABLES
//---------------------------------------------------------
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split(`/`);
const folder_path_length = folder_path_split.length;

//---------------------------------------------------------
// SET PARENT TASK BY FILE PATH OR SUGGESTER
//---------------------------------------------------------
let parent_task_obj;
let parent_task_value;
let parent_task_name;
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 4) {
  parent_task_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[3],
    file_class: "task",
    type: "parent_task",
  });
  parent_task_value = parent_task_obj[1].value;
  parent_task_name = parent_task_obj[1].key;
} else {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_value,
    file_class: "task",
    type: "parent_task",
  });
  parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    "Parent Task?"
  );
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
}
const parent_task_link = `${parent_task_value}|${parent_task_name}`;
const parent_task_value_link = `${new_line}${ul_yaml}"${parent_task_link}"`;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[52_00_task_event|General Tasks and Events Template]]
2. [[53_00_action_item|Action Item Template]]
3. [[54_00_meeting|Meeting Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[file_name_alias_by_class_type|Markdown File Names Filtered by Directory, File Class, and Type Suggester]]
2. [[task_context_by_path_or_suggester|Set Task Context by Path or Suggester]]
3. [[task_context_project_by_path_or_suggester|Set Project by Path or Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[file_name_alias_by_class_type|Markdown File Names Filtered by Directory, File Class, and Type]]
2. [[related_parent_task_suggester|Related Parent Task Suggester]]
3. [[related_project_suggester|Related Project Suggester]]

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
