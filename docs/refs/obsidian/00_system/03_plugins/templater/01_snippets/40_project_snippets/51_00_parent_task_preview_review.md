---
title: 41_parent_task_preview_review
aliases:
  - Parent Task Preview and Review
  - Before Parent Task Preview and After Parent Task Review
  - parent task preview and review
  - before parent task preview and after parent task review
  - parent task preview review
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-13T13:49
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Parent Task Preview and Review

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a parent tasks preview and review sections with callouts.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const parent_task_preview_review = "41_parent_task_preview_review";

//---------------------------------------------------------  
// PARENT TASK PREVIEW AND REVIEW
//---------------------------------------------------------
// Retrieve the Parent Task Preview template and content
temp_file_path = `${sys_temp_include_dir}${parent_task_preview_review}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const parent_task_preview_review = include_arr;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// PARENT TASK PREVIEW AND REVIEW
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${parent_task_preview_review}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const parent_task_preview_review = include_arr;
```

#### Referenced Template

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
let call_title = "";
let query = "";

//---------------------------------------------------------  
// PARENT TASK PREVIEW AND REVIEW
//--------------------------------------------------------- 
const three_head = `${hash.repeat(3)}${space}`;

// Preview
heading = `${three_head}Preview${two_new_line}`;
call_title = `${call_start}[!task_preview]${space}Parent Task Preview${new_line}${call_start}${new_line}`;
const problem = `${call_start}1.${space}Which of the project's needs does the parent task address?${new_line}${call_ul_indent}**Problem**::${new_line}${call_start}${new_line}`;
const solution = `${call_start}2.${space}What is the problem's solution?${new_line}${call_ul_indent}**Solution**::${new_line}${call_start}${new_line}`;
const objective = `${call_start}3.${space}What is the parent task's objective?${new_line}${call_ul_indent}**Objective**::`;
const preview = `${heading}${call_title}${problem}${solution}${objective}${two_new_line}`;

// Review
heading = `${three_head}Review${two_new_line}`;
call_title = `${call_start}[!task_review]${space}Parent Task Review${new_line}${call_start}${new_line}`;
const outcome = `${call_start}1.${space}What was the outcome?${space}How did it make me feel?${new_line}${call_ul_indent}**Outcome**::${new_line}${call_ul_indent}**Outcome emotion**::${new_line}${call_start}${new_line}`;
const keep = `${space}2.${space}What went well?${new_line}${call_ul_indent}**Keep**::${new_line}${call_start}${new_line}`;
const improve = `${space}3.${space}What can be improved?${new_line}${call_ul_indent}**Improve**::${new_line}${call_start}${new_line}`;
const start = `${space}4.${space}What can be started?${new_line}${call_ul_indent}**Start**::${new_line}${call_start}${new_line}`;
const stop = `${space}5.${space}What can be stopped?${new_line}${call_ul_indent}**Stop**::`;
const review = `${heading}${call_title}${outcome}${keep}${improve}${start}${stop}`;

const preview_review = `${preview}${review}`;

tR += preview_review;
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
2. [[51_32_parent_ed_book_chapter|Education Book Chapter Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[00_system/06_template_include/41_parent_task_preview_review]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[50_00_proj_preview_smart|Project Preview SMARTER Framework]]
2. [[50_00_proj_review_kiss|Project Review KISS Framework]]
3. [[53_00_action_item_preview|Action Item Preview]]
4. [[53_00_action_item_review|After Action Item Review]]

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
