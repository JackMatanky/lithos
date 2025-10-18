---
title: task_event_section_project
aliases:
  - Project Tasks and Events Section
  - Project Tasks and Events Section Dataview Tables
  - project tasks and events section dataview tables
  - task event section project
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-18T09:27
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Project Tasks and Events Section

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a project's tasks and events section formatted with headings and tables.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const related_task_sect_proj = "140_00_related_task_sect_proj";

//---------------------------------------------------------
// PROJECT TASKS AND EVENTS SECTION
//---------------------------------------------------------
// Retrieve the Tasks and Events Section 
// for Project template and content
temp_file_path = `${sys_temp_include_dir}${related_task_sect_proj}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_project_task_event_section = include_arr;

```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// PROJECT TASKS AND EVENTS SECTION
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_task_sect_proj}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_project_task_event_section = include_arr;
```

#### Referenced Templates

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const head = String.fromCodePoint(0x23);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const four_space = space.repeat(4);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const hr_line = String.fromCodePoint(0x2d).repeat(3);
const cmnt_ob_start = `${String.fromCodePoint(37).repeat(2)}${space}`;
const cmnt_ob_end = `${space}${String.fromCodePoint(37).repeat(2)}`;
const colon = String.fromCodePoint(0x3a);
const two_colon = colon.repeat(2);
const ul = `${String.fromCodePoint(0x2d)}${space}`;
const tbl_start =`|${space}`;
const tbl_end =`${space}|`;
const tbl_pipe = `${space}|${space}`;
const call_start = `>${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${four_space}${ul}`;
const call_check = `${call_ul}[${space}]${space}`;
const call_check_indent = `${call_ul_indent}[${space}]${space}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;

const head_one = `${hash}${space}`;
const head_two = `${hash.repeat(2)}${space}`;
const head_three = `${hash.repeat(3)}${space}`;
const head_four = `${hash.repeat(4)}${space}`;

//---------------------------------------------------------
// GENERAL VARIABLES
//---------------------------------------------------------
let heading = "";
let comment = "";
let query = "";

//---------------------------------------------------------
// RELATED TASKS AND EVENTS BUTTONS
//---------------------------------------------------------
const project = `${backtick}button-project${backtick}`;
const parent_task = `${backtick}button-parent${backtick}`;
const action_item = `${backtick}button-action-item${backtick}`;
const meeting = `${backtick}button-meeting${backtick}`;

const buttons_table = `${tbl_start}${project}${tbl_pipe}${parent_task}${tbl_end}
|:----------------------- |:---------------------- |
${tbl_start}${action_item}${tbl_pipe}${meeting}${tbl_end}`;

//---------------------------------------------------------
// RELATED TASKS AND EVENTS BUTTON
//---------------------------------------------------------
comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;

const button_name = `name ✅Related Tasks and Events${new_line}`;
const button_type = `type append template${new_line}`;
const button_action = `action 100_40_dvmd_related_task_sect${new_line}`;
const button_replace = `replace [1, 2]${new_line}`;
const button_color = `color blue${new_line}`;

const button = `${comment}${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}`;

//---------------------------------------------------------
// PROJECT TASKS AND EVENTS SECTION
//---------------------------------------------------------
const three_head = `${hash.repeat(3)}${space}`;

heading = `${three_head}Parent Tasks${two_new_line}`;
query = await tp.user.dv_task_linked({
  type: "project",
  status: "",
  relation: "parent",
  md: "false",
});
const parent = `${heading}${query}${two_new_line}`;

heading = `${three_head}Remaining Tasks${two_new_line}`;
query = await tp.user.dv_task_linked({
  type: "task",
  status: "due",
  relation: "child",
  md: "false",
})
const remaining = `${heading}${query}${two_new_line}`;

heading = `${three_head}Completed Tasks${two_new_line}`;
query = await tp.user.dv_task_linked({
  type: "task",
  status: "",
  relation: "child",
  md: "false",
});
const completed = `${heading}${query}${two_new_line}`;

const task_section = `${new_line}${parent}${remaining}${completed}${hr_line}${new_line}`;

tR += task_section;
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
2. [[51_20_parent_habit_ritual_month|Monthly Habits Parent Task Template]]
3. [[51_22_parent_month_morn_rit|Monthly Morning Rituals Parent Task Template]]
4. [[51_23_parent_month_work_start_rit|Monthly Workday Startup Rituals Parent Task Template]]
5. [[51_24_parent_month_work_shut_rit|Monthly Workday Shutdown Rituals Parent Task Template]]
6. [[51_25_parent_month_eve_rit|Monthly Evening Rituals Parent Task Template]]
7. [[51_32_parent_ed_book_chapter|Education Book Chapter Template]]
8. [[51_41_parent_job_application|Job Application Parent Task Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[140_00_related_task_sect_proj]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[related_task_event_section_general|Related Tasks and Events Section]]
2. [[related_task_event_section_proj_suggester|Related Tasks and Events Section with Related Project Suggester]]
3. [[related_dir_sect|Related Directory Section]]
4. [[related_lib_sect|Related Library Section]]
5. [[related_lib_sect_related_file|Related Library Section with Related Content Suggester]]
6. [[related_pkm_section|Related Personal Knowledge Section]]
7. [[related_note_section|Related Note Section]]
8. [[50_00_proj_related_section|Project Related Section]]

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

1. [[tp.file.include Templater Function|The Templater tp.file.include() Function]]

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
