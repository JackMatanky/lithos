---
title: related_dir_sect
aliases:
  - Related Directory Section
  - Related Directory Section Dataview Tables
  - related directory section dataview tables
  - related dir section
plugin: templater
language:
  - javascript
module:
  - file
cssclasses: null
type: snippet
file_class: pkm_code
date_created: 2023-07-17T11:13
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Related Directory Section

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a file's related directory section formatted with headings and tables.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const related_dir_sect = "100_50_related_dir_sect";

//---------------------------------------------------------
// RELATED DIRECTORY SECTION 
//---------------------------------------------------------
// Retrieve the Related Directory Section template and content
temp_file_path = `${sys_temp_include_dir}${related_dir_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_directory_section = include_arr;

```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// RELATED DIRECTORY SECTION 
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_dir_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_directory_section = include_arr;
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
const call_ul_indent = `${call_start}${four_space}-${space}`;
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
// RELATED DIRECTORY BUTTON
//---------------------------------------------------------
comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;

const button_name = `name 📇Related Directory Files${new_line}`;
const button_type = `type append template${new_line}`;
const button_action = `action 100_50_dvmd_related_dir_sect${new_line}`;
const button_replace = `replace [1, 2]${new_line}`;
const button_color = `color yellow${new_line}`;

const button = `${comment}${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}`;

//---------------------------------------------------------
// RELATED DIRECTORY SECTION 
//---------------------------------------------------------
const three_head = `${hash.repeat(3)}${space}`;

// Outgoing Contacts
heading = `${three_head}Outgoing Contact Links${two_new_line}`;
comment = `${cmnt_html_start}Link related contacts here${cmnt_html_end}${two_new_line}`;
const contact_outlink = `${heading}${comment}`;

// Contacts
heading = `${three_head}Contacts${two_new_line}`;
query = await tp.user.dv_dir_linked("contact", "false");
const dir_contact = `${heading}${query}${two_new_line}`;

// Outgoing Organizations
heading = `${three_head}Outgoing Organization Links${two_new_line}`;
comment = `${cmnt_html_start}Link related organizations here${cmnt_html_end}${two_new_line}`;
const org_outlink = `${heading}${comment}`;

// Organizations
heading = `${three_head}Organizations${two_new_line}`;
query = await tp.user.dv_dir_linked("organization", "false");
const dir_org = `${heading}${query}${two_new_line}`;

const dir_section = `${new_line}${button}${contact_outlink}${dir_contact}${org_outlink}${dir_org}${hr_line}${new_line}`;

tR += dir_section;
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

1. [[100_50_related_dir_sect]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[related_task_event_section_general|Related Tasks and Events Section]]
2. [[related_task_event_section_proj_suggester|Related Tasks and Events Section with Related Project Suggester]]
3. [[related_lib_sect|Related Library Section]]
4. [[related_lib_sect_related_file|Related Library Section with Related Content Suggester]]
5. [[related_pkm_section|Related Personal Knowledge Section]]
6. [[related_note_section|Related Note Section]]
7. [[50_00_proj_related_section|Project Related Section]]

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
