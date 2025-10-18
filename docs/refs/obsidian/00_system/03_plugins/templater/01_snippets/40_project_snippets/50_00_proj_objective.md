---
title: 50_00_proj_objective
aliases:
  - Project Objective
  - project objective
  - proj objective
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-22T14:38
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Project Objective

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a callout for the project objective

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const project_objective_callout = "40_project_objective_callout";

//---------------------------------------------------------  
// PROJECT OBJECTIVE
//---------------------------------------------------------
// Retrieve the Project Objective template and content
temp_file_path = `${sys_temp_include_dir}${project_objective_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const project_objective = include_arr;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// PROJECT OBJECTIVE
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${project_objective_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const project_objective = include_arr;
```

#### Referenced Template

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
// PROJECT OBJECTIVE
//---------------------------------------------------------
const call_title = `${call_start}[!objective]${space}Project Objective${two_space}${new_line}${call_start}${new_line}`;
const call_body = `${call_start}Write the objective as a sentence:${two_space}${new_line}${call_ul}**Objective**::`;

const project_objective = `${call_title}${call_body}`;

//---------------------------------------------------------  
// BEFORE ACTION SMART PREVIEW
//---------------------------------------------------------
const call_title = `${call_start}[!task_preview]${space}Project Preview${new_line}${call_start}${new_line}`;

const need = `${call_start}1.${space}**NEED**${new_line}${call_ul_indent}What is the problem or need?${new_line}${call_ul_indent}Why does the problem matter?${new_line}${call_start}${new_line}`;

const specific = `${call_start}2.${space}**SPECIFIC**${new_line}${call_ul_indent}What needs to be accomplished?${new_line}${call_ul_indent}What steps need to be taken to succeed?${new_line}${call_start}${new_line}`;

const measure = `${call_start}3.${space}**MEASURABLE**${new_line}${call_ul_indent}How can I measure my progress?${new_line}${call_ul_indent}How will I know if I have succeeded?${new_line}${call_start}${new_line}`;

const action = `${call_start}4.${space}**ACTIONABLE**${new_line}${call_ul_indent}Is this a realistic objective?${new_line}${call_ul_indent}Is the the allotted time enough considering the objective and other tasks?${new_line}${call_start}${new_line}`;

const relevant = `${call_start}5.${space}**RELEVANT**${new_line}${call_ul_indent}How does the objective align with my life's demands?${new_line}${call_ul_indent}How does the objective align with my life's needs?${new_line}${call_start}${new_line}`;

const time = `${call_start}6.${space}**TIME-BOUND**${new_line}${call_ul_indent}When is the start date?${new_line}${call_ul_indent}When is the end date?${new_line}${call_ul_indent}How frequently will I work toward the objective?${new_line}${call_start}${new_line}`;

const excite = `${call_start}7.${space}**EXCITING**${new_line}${call_ul_indent}Do I think this is important? Why?${new_line}${call_ul_indent}Does the objective interest me? Why?${new_line}${call_ul_indent}Am I excited to accomplish my objective?${new_line}${call_start}${new_line}`;

const risk = `${call_start}8.${space}**RISKY**${new_line}${call_ul_indent}How is the objective challenging?`;

const project_preview = `${call_title}${need}${specific}${measure}${action}${relevant}${time}${excite}${risk}`;

//---------------------------------------------------------  
// AFTER PROJECT KISS REVIEW
//---------------------------------------------------------
const three_head = `${hash.repeat(3)}${space}`;
const heading = `${three_head}Review${space}${two_new_line}`;

const call_title = `${call_start}[!task_review]${space}Project${space}Review${new_line}${call_start}${new_line}`;

const outcome = `${call_start}1.${space}What was the outcome?${space}How did it make me feel?${new_line}${call_ul_indent}**Outcome**::${new_line}${call_ul_indent}**Outcome emotion**::${new_line}${call_start}${new_line}`;

const keep = `${space}2.${space}What went well?${new_line}${call_ul_indent}**Keep**::${new_line}${call_start}${new_line}`;

const improve = `${space}3.${space}What can be improved?${new_line}${call_ul_indent}**Improve**::${new_line}${call_start}${new_line}`;

const start = `${space}4.${space}What can be started?${new_line}${call_ul_indent}**Start**::${new_line}${call_start}${new_line}`;

const stop = `${space}5.${space}What can be stopped?${new_line}${call_ul_indent}**Stop**::`;

const project_review = `${heading}${call_title}${outcome}${keep}${improve}${start}${stop}`;

tR += project_objective;
tR += project_preview;
tR += project_review;
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
4. [[50_20_proj_habit_ritual|Habits and Rituals Project Template]]
5. [[50_21_proj_habit_ritual_month|Monthly Habits and Rituals Project Template]]
6. [[50_30_proj_education|Education Project Template]]
7. [[50_31_proj_ed_course(X)|Education Course Project Template]]
8. [[50_32_proj_ed_book|Education Book Project Template]]
9. [[50_33_proj_ed_book_parent_chapter|Education Book Project and Chapter Parent Tasks Template]]
10. [[50_40_proj_professional|Professional Project Template]]
11. [[50_50_proj_work|Work Project Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[40_project_preview_review]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[50_00_proj_objective|Project Preview SMARTER Framework]]
2. [[50_00_proj_review_kiss|Project Review KISS Framework]]
3. [[53_00_action_item_preview|Before Action Preview]]
4. [[53_00_action_item_review|After Action Review]]

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
