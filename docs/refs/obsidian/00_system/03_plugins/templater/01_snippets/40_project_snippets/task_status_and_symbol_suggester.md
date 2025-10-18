---
title: task_status_symbol_suggester
aliases:
  - Task Status and Symbol
  - Task Status and Symbol Suggester
  - suggester_task_status_symbol
plugin: templater
language:
  - javascript
module:
  - system
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T16:16
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Task Status and Symbol Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Set the task status name for metadata and status symbol for the checkbox.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript 
// Template file to include
const task_status = `50_task_status`;

//---------------------------------------------------------  
// SET TASK STATUS AND SYMBOL
//---------------------------------------------------------
// Retrieve the Task Status template and content
temp_file_path = `${sys_temp_include_dir}${task_status}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const status_value = include_arr[0];
const status_name = include_arr[1];
const status_symbol = include_arr[2];
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET TASK STATUS AND SYMBOL
//---------------------------------------------------------
// Retrieve the Task Status template and content
temp_file_path = `${sys_temp_include_dir}${task_status}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const status_value = include_arr[0];
const status_name = include_arr[1];
const status_symbol = include_arr[2];
```

#### Referenced Template

```javascript
//---------------------------------------------------------  
// SET TASK STATUS AND SYMBOL
//---------------------------------------------------------
const status_obj_arr = [
  { key: `To do`, value: `to_do`, symbol: ` ` },
  { key: `In progress`, value: `in_progress`, symbol: `/` },
  { key: `Done`, value: `done`, symbol: `x` },
  { key: `Schedule`, value: `schedule`, symbol: `?` },
];

const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  `Status?`
);

const status_value = status_obj.value;
const status_name = status_obj.key;
const status_symbol = status_obj.symbol;

tR += status_value
tR += ","
tR += status_name
tR += ","
tR += status_symbol
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
2. [[51_00_parent_task|General Parent Task Template]]
3. [[52_00_task_event|General Tasks and Events Template]]
4. [[53_00_action_item|Action Item Template]]
5. [[54_00_meeting|Meeting Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[50_task_status]]

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

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]

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
