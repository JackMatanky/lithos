---
title: task_context_project_by_path_or_suggester
aliases:
  - Context and Project by Path or Suggester
  - Set Context and Project by Path or Suggester
  - Set Context and Project
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
# Context and Project by Path or Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: If a task is already in a project's folder, set the task's context and project based on the folder path; otherwise, set the task context and project with the suggester.

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
// SET PROJECT BY FILE PATH OR SUGGESTER
//---------------------------------------------------------
// Initialize task project variables
let project_obj;
let project_value;
let project_name;
let project_dir;
let context_value;

// Check if the parent directory equals projects_dir, 40_projects/
// and the folder path array's length is equal to or greater than three
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 3) {
  // If true, assign the project to 
  // the split folder path array's third element
  project_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[2],
    file_class: "task",
    type: "project",
  });
  project_value = project_obj[1].value;
  project_name = project_obj[1].key;
  project_dir = `${projects_dir}${folder_path_split[1]}/${folder_path_split[2]}/`;
} else {
  // If false, return an array of project folder names
  // filtered by the context directory
  // TODO: DEFINE CONTEXT DIRECTORY
  project_obj_arr = await tp.user.file_name_alias_by_class_type_status({
    dir: projects_dir,
    file_class: "task",
    type: "project",
    status: "active",
  });

  // Choose a project from the array of project folder names
  project_obj = await tp.system.suggester(
    (item) => item.key,
    project_obj_arr,
    false,
    "Project?"
  );
  project_value = project_obj.value;
  project_name = project_obj.key;
  project_name_ext = `${project_value}.md`;
  project_dir = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${project_name_ext}`))
    .map((file) => file.path)[0]
    .replace(project_name_ext, "")
};

const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;

const context_value = project_dir.split("/")[1].replace(/^\d\d_/g, "");
let context_name;
if (context_value.startsWith("habit")) {
  context_name = "Habits and Rituals";
} else {
  context_name =
    context_value.charAt(0).toUpperCase() + context_value.substring(1);
};
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
const folder_path_split = folder_path.split("/");
const folder_path_length = folder_path_split.length;

//---------------------------------------------------------
// SET CONTEXT AND PROJECT BY FILE PATH OR SUGGESTER
//---------------------------------------------------------
let project_obj;
let project_value;
let project_name;
let project_dir;

if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 3) {
  project_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[2],
    file_class: "task",
    type: "project",
  });
  project_value = project_obj[1].value;
  project_name = project_obj[1].key;
  project_dir = `${projects_dir}${folder_path_split[1]}/${folder_path_split[2]}/`;
} else {
  project_obj_arr = await tp.user.file_name_alias_by_class_type_status({
    dir: projects_dir,
    file_class: "task",
    type: "project",
    status: "active",
  });
  project_obj = await tp.system.suggester(
    (item) => item.key,
    project_obj_arr,
    false,
    "Project?"
  );
  project_value = project_obj.value;
  project_name = project_obj.key;
  project_name_ext = `${project_value}.md`;
  project_dir = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${project_name_ext}`))
    .map((file) => file.path)[0]
    .replace(project_name_ext, "")
}
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;

const context_value = project_dir.split("/")[1].replace(/^\d\d_/g, "");
let context_name;
if (context_value.startsWith("habit")) {
  context_name = "Habits and Rituals";
} else {
  context_name =
    context_value.charAt(0).toUpperCase() + context_value.substring(1);
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

1. [[51_00_parent_task|General Parent Task Template]]
2. [[52_00_task_event|General Tasks and Events Template]]
3. [[53_00_action_item|Action Item Template]]
4. [[54_00_meeting|Meeting Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[file_name_alias_by_class_type|Markdown File Names Filtered by Directory, File Class, and Type Suggester]]
2. [[task_context_by_path_or_suggester|Set Task Context by Path or Suggester]]
3. [[task_parent_task_by_path_or_suggester|Parent Task by Path or Suggester]]

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
	Definition AS Definition
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
