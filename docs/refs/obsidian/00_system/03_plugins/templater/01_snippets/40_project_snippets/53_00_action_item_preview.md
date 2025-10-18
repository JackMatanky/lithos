---
title: 53_00_action_item_preview
aliases:
  - Action Item Preview
  - Before Action Item Preview
  - action item preview
  - before action item preview
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
# Before Action Preview

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a callout for a Before Action Item Preview.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const before_action_item_preview = "43_action_item_preview";

//---------------------------------------------------------  
// BEFORE ACTION ITEM PREVIEW
//---------------------------------------------------------
// Retrieve the Action Item Preview template and content
temp_file_path = `${sys_temp_include_dir}${before_action_item_preview}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const action_item_preview = include_arr;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// BEFORE ACTION ITEM PREVIEW
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${before_action_item_preview}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const action_item_preview = include_arr;
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
// BEFORE ACTION ITEM PREVIEW
//--------------------------------------------------------- 
const three_head = `${hash.repeat(3)}${space}`;
const heading = `${three_head}Preview${space}${two_new_line}`;

const call_title = `${call_start}[!task_preview]${space}Action Item Preview${new_line}${call_start}${new_line}`;

const problem = `${call_start}1.${space}What is the problem to solve?${new_line}${call_ul_indent}**Problem**::${new_line}${call_start}${new_line}`;

const desire = `${space}2.${space}What do I want?${new_line}${call_ul_indent}**Desire**::${new_line}${call_start}${new_line}`;

const plan = `${space}3.${space}What will I do?${new_line}${call_ul_indent}**Plan**::${new_line}${call_start}${new_line}`;

const refrain = `${space}4.${space}What won't I do?${new_line}${call_ul_indent}**Refrain**::`;

const action_item_preview = `${heading}${call_title}${problem}${desire}${plan}${refrain}`;

tR += action_item_preview;
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

1. [[53_action_item_preview]]
2. [[task_event_plan|Action Plan]]
3. [[53_00_action_item_review|After Action Review]]

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
