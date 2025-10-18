---
title: task_do_or_due_date_suggester
aliases:
  - Task Do or Due Date
  - Task Do or Due Date Suggester
  - suggester_task_do_or_due_date
plugin: templater
language:
  - javascript
module:
  - system
  - file
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T16:15
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Task Do or Due Date Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const do_due_date = `50_task_do_due_date`;

//---------------------------------------------------------  
// SET DO/DUE DATE
//---------------------------------------------------------
// Retrieve the Do or Due Date template and content
temp_file_path = `${sys_temp_include_dir}${do_due_date}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const due_do_value = include_arr[0];
const due_do_name = include_arr[1];
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET DO/DUE DATE
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${do_due_date}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const due_do_value = include_arr[0];
const due_do_name = include_arr[1];
```

#### Referenced Template

```javascript
//---------------------------------------------------------  
// SET DO/DUE DATE
//---------------------------------------------------------
const due_do_obj_arr = [
  { key: "DO Date", value: "do" },
  { key: "DUE Date", value: "due" },
];

const due_do_obj = await tp.system.suggester(  
  (item) => item.key,  
  due_do_obj_arr,  
  false,  
  "Do or Due Date?"
);

const due_do_value = due_do_obj.value;
const due_do_name = due_do_obj.key;

tR += due_do_value
tR += ","
tR += due_do_name
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
3. [[50_10_proj_personal|Personal Project Template]]
4. [[50_30_proj_education|Education Project Template]]
5. [[50_31_proj_ed_course(X)|Education Course Project Template]]
6. [[50_32_proj_ed_book|Education Book Project Template]]
7. [[50_40_proj_professional|Professional Project Template]]
8. [[50_50_proj_work|Work Project Template]]
9. [[51_00_parent_task|General Parent Task Template]]
10. [[52_00_task_event|General Tasks and Events Template]]
11. [[53_00_action_item|Action Item Template]]
12. [[54_00_meeting|Meeting Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[50_task_do_due_date]]

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
