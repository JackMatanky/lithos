---
title: task_context_by_path_or_suggester
aliases:
  - Task Context by Path or Suggester
  - Set Task Context by Path or Suggester
  - Set Task Context
  - task_context_by_path_or_suggester
plugin: templater
language:
  - javascript
module:
  - system
  - file
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-01T13:41
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/folder
---
# Task Context by Path or Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: If a task is already in a folder, set the task's context based on the folder path; otherwise, set the task context with the suggester.

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
// SET TASK CONTEXT BY FILE PATH OR SUGGESTER
//---------------------------------------------------------
// Initialize task context variables
let context_dir;
let context_value;
let context_name;


// Check if the parent directory equals projects_dir, 40_projects/
// and the folder path array's length is equal to or greater than two 
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 2) {
  // If true, concatenate 40_projects/ and 
  // the folder path array's second element and 
  // assign them to the context directory 
  context_dir = `${projects_dir}${folder_path_split[1]}/`;
  // Assign the context to the split folder path array's
  // second element without the id numbers
  context_value = folder_path_split[1].slice(3);
  // If the context value starts with  "habit",
  // then assign context_name to "Habits and Rituals",
  // otherwise context_name equals context value 
  // with a capital first letter 
  if (context_value.startsWith(`habit`)) {
    context_name = `Habits and Rituals`;
  } else {
    context_name =
      context_value.charAt(0).toUpperCase() + context_value.substring(1);
  };
} else {
  // If false, choose an object from the array of task context objects
  const context_obj = await tp.user.task_context(tp);

  // Return the object's directory, value, and name
  context_dir = context_obj.directory;
  context_value = context_obj.value;
  context_name = context_obj.key;
}
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
// >>>TODO: DEFINE <projects_dir> VARIABLE<<<
//---------------------------------------------------------
// FILE PATH VARIABLES
//---------------------------------------------------------
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split(`/`);
const folder_path_length = folder_path_split.length;

//---------------------------------------------------------
// SET TASK CONTEXT BY FILE PATH OR SUGGESTER
//---------------------------------------------------------
let context_dir;
let context_value;
let context_name;
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 2) {
  context_dir = `${projects_dir}${folder_path_split[1]}/`;
  context_value = folder_path_split[1].slice(3);
  if (context_value.startsWith(`habit`)) {
    context_name = `Habits and Rituals`;
  } else {
    context_name =
      context_value.charAt(0).toUpperCase() + context_value.substring(1);
  }
} else {
  const context_obj = await tp.user.task_context(tp);
  context_dir = context_obj.directory;
  context_value = context_obj.value;
  context_name = context_obj.key;
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

1. [[50_00_project|General Project Template]]
2. [[50_01_project_parent_tasks|General Project with Parent Tasks Template]]
3. [[51_00_parent_task|General Parent Task Template]]
4. [[52_00_task_event|General Tasks and Events Template]]
5. [[53_00_action_item|Action Item Template]]
6. [[54_00_meeting|Meeting Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[task_context|Task Context Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[task_context_project_by_path_or_suggester|Project by Path or Suggester]]

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
