---
title: related_task_event_section_general
aliases:
  - General Related Tasks and Events Section
  - General Related Tasks and Events Section Dataview Tables
  - general related tasks and events section dataview tables
  - related task event section general
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-17T11:21
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# General Related Tasks and Events Section

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a file's related tasks and events section formatted with headings and tables.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const related_task_sect = "100_40_related_task_sect_general";

//---------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION 
//---------------------------------------------------------
// Retrieve the Related Tasks and Events Section template and content
temp_file_path = `${sys_temp_include_dir}${related_task_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const related_task_event_section = include_arr;

```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION 
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_task_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_task_event_section = include_arr;
```

#### Referenced Templates

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const hr_line = hyphen.repeat(3);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const colon = String.fromCodePoint(0x3a);
const two_colon = colon.repeat(2);
const two_percent = String.fromCodePoint(0x25).repeat(2);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

//Text Formatting
const head_one = `${hash}${space}`;
const head_two = `${hash.repeat(2)}${space}`;
const head_three = `${hash.repeat(3)}${space}`;
const head_four = `${hash.repeat(4)}${space}`;
const cmnt_ob_start = `${two_percent}${space}`;
const cmnt_ob_end = `${space}${two_percent}`;
const cmnt_html_start = `${less_than}${excl}${two_hyphen}${space}`;
const cmnt_html_end = `${space}${two_hyphen}${great_than}`;
const tbl_start =`${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end =`${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${hyphen}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${great_than}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_start}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;

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
${tbl_start}${action_item}${tbl_pipe}${meeting}${tbl_end}${two_new_line}`;

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
// RELATED TASKS AND EVENTS SECTION
//---------------------------------------------------------
const three_head = `${hash.repeat(3)}${space}`;

heading = `${three_head}Outgoing Task and Events Links${two_new_line}`;
comment = `${cmnt_html_start}Link related tasks and events here${cmnt_html_end}${two_new_line}`;
const outlink = `${heading}${comment}`;

heading = `${three_head}Projects${two_new_line}`;
query = await tp.user.dv_task_linked({
  type: "project",
  status: "",
  relation: "link",
  md: "false",
});
const project = `${heading}${query}${two_new_line}`;

heading = `${three_head}Parent Tasks${two_new_line}`;
query = await tp.user.dv_task_linked({
  type: "parent_task",
  status: "",
  relation: "link",
  md: "false",
});
const parent = `${heading}${query}${two_new_line}`;

heading = "### Child Tasks";
query = await tp.user.dv_task_linked({
  type: "task",
  status: "",
  relation: "link",
  md: "false",
});
const child = `${heading}${query}${two_new_line}`;

const related_task_section = `${new_line}${buttons_table}${button}${outlink}${project}${parent}${child}${hr_line}${new_line}`;

tR += related_task_section;
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
3. [[53_10_act_week_review_preview|Weekly Review and Preview]]
4. [[54_00_meeting|Meeting Template]]
5. [[90_00_note|General Note Template]]
6. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
7. [[90_11_note_quote|Quote Fleeting Note Template]]
8. [[90_12_note_idea|Idea Fleeting Note Template]]
9. [[90_20_note_literature(X)|General Literature Note Template]]
10. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
11. [[90_31_note_question|QEC Question Note Template]]
12. [[90_32_note_evidence|QEC Evidence Note Template]]
13. [[90_33_note_conclusion|QEC Conclusion Note Template]]
14. [[90_40_note_lit_psa(X)|PSA Note Template]]
15. [[90_41_note_problem|PSA Problem Note Template]]
16. [[90_42_note_steps|PSA Steps Note Template]]
17. [[90_43_note_answer|PSA Answer Note Template]]
18. [[90_50_note_info(X)|General Info Note Template]]
19. [[90_51_note_concept|Concept Note Template]]
20. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[100_40_related_task_sect_general]]

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
