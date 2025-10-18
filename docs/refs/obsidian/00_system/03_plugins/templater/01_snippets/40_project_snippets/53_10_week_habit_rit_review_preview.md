---
title: 53_10_week_habit_rit_review_preview
aliases:
  - Weekly Habits and Rituals Review and Preview Checklist
  - Weekly Habits and Rituals Review and Preview
  - weekly habits and rituals review and preview checklist
  - weekly habits and rituals review and preview
  - week habit rit review preview
plugin: templater
language:
  - javascript
module:
  - file
cssclasses: null
type: snippet
file_class: pkm_code
date_created: 2023-06-13T13:49
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Weekly Habits and Rituals Review and Preview Checklist

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return callouts of checklists for the Weekly Habits and Rituals Review and Preview Checklist.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include 
const week_habit_ritual_review_preview = "43_10_action_week_habit_ritual_review_preview";

//---------------------------------------------------------  
// WEEKLY HABITS AND RITUALS REVIEW AND PREVIEW CHECKLIST
//---------------------------------------------------------
// Retrieve the Weekly Habits and Rituals 
// Review and Preview Checklist template and content
temp_file_path = `${sys_temp_include_dir}${week_habit_ritual_review_preview}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const habit_ritual_review_preview_checklist = include_arr;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// WEEKLY HABITS AND RITUALS REVIEW AND PREVIEW CHECKLIST
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${week_habit_ritual_review_preview}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const habit_ritual_review_preview_checklist = include_arr;
```

#### Referenced Template

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
// WEEKLY HABITS AND RITUALS REVIEW AND PREVIEW CHECKLIST
//---------------------------------------------------------
const call_title = `${call_start}[!task_plan]${space}Weekly Habits and Rituals Review and Preview Plan${new_line}${call_start}${new_line}`;

const habit = `${call_start}1.${space}**Habits**${new_line}${call_check_indent}Review completed habits.${new_line}${call_check_indent}Review discarded habits.${new_line}${call_check_indent}Compare completed and discarded habits.${new_line}${call_check_indent}Write insights about comparison.${new_line}${call_check_indent}Write actionable lessons learned to implement.${new_line}${call_start}${new_line}`;

const morn_rit = `${call_start}2.${space}**Morning Rituals**${new_line}${call_check_indent}Review completed morning rituals.${new_line}${call_check_indent}Review discarded morning rituals.${new_line}${call_check_indent}Compare completed and discarded morning rituals.${new_line}${call_check_indent}Write insights about comparison.${new_line}${call_check_indent}Write actionable lessons learned to implement.${new_line}${call_start}${new_line}`;

const work_start = `${call_start}3.${space}**Workday Startup Rituals**${new_line}${call_check_indent}Review completed workday startup rituals.${new_line}${call_check_indent}Review discarded workday startup rituals.${new_line}${call_check_indent}Compare completed and discarded workday startup rituals.${new_line}${call_check_indent}Write insights about comparison.${new_line}${call_check_indent}Write actionable lessons learned to implement.${new_line}${call_start}${new_line}`;

const work_shut = `${call_start}4.${space}**Workday Shutdown Rituals**${new_line}${call_check_indent}Review completed workday shutdown rituals.${new_line}${call_check_indent}Review discarded workday shutdown rituals.${new_line}${call_check_indent}Compare completed and discarded workday shutdown rituals.${new_line}${call_check_indent}Write insights about comparison.${new_line}${call_check_indent}Write actionable lessons learned to implement.${new_line}${call_start}${new_line}`;

const eve_rit = `${call_start}5.${space}**Evening Rituals**${new_line}${call_check_indent}Review completed evening rituals.${new_line}${call_check_indent}Review discarded evening rituals.${new_line}${call_check_indent}Compare completed and discarded evening rituals.${new_line}${call_check_indent}Write insights about comparison.${new_line}${call_check_indent}Write actionable lessons learned to implement.${new_line}${call_start}${new_line}`;

const next_hab_rit = `${call_start}6.${space}**Upcoming Habits and Rituals**${new_line}${call_check_indent}If necessary, revise templates according to lessons learned during review.${new_line}${call_check_indent}Create habit and ritual files for upcoming week.`;

const habit_ritual_review_preview = `${call_title}${habit}${morn_rit}${work_start}${work_shut}${eve_rit}${next_hab_rit}`;

tR += habit_ritual_review_preview;
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

1. [[53_10_action_week_habit_ritual_review_preview]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[53_10_week_pdev_review_preview|Weekly PDEV Journals Review and Preview Checklist]]
2. [[53_10_week_task_event_review|Weekly Tasks and Events Review Checklist]]
3. [[53_10_week_task_event_preview|Weekly Tasks and Events Preview Checklist]]
4. [[53_10_week_habit_rit_review_preview|Weekly Habits and Rituals Review and Preview Checklist]]
5. [[53_10_week_lib_review_preview|Weekly Library Review and Preview Checklist]]
6. [[53_10_week_pkm_review_preview|Weekly PKM Review and Preview Checklist]]
7. [[53_00_action_item_preview|Before Action Preview]]
8. [[task_event_plan|Action Plan]]
9. [[53_00_action_item_review|After Action Review]]

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
