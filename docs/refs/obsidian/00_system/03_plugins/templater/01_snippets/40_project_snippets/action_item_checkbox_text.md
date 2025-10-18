---
title: action_item_checkbox_text
aliases:
  - Action Item Checkbox Text
  - action item checkbox text
plugin: templater
language:
  - javascript
module: 
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-13T18:34
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Action Item Checkbox Text

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a task checkbox for an action item based on the task's status.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>TODO: DEFINE <status_value> VARIABLE<<<
// >>>TODO: DEFINE <status_symbol> VARIABLE<<<
// >>>TODO: DEFINE <task_tag> VARIABLE<<<
// >>>TODO: DEFINE <title> VARIABLE<<<
// >>>TODO: DEFINE <type_value> VARIABLE<<<
// >>>TODO: DEFINE <start_time> VARIABLE<<<
// >>>TODO: DEFINE <end_time> VARIABLE<<<
// >>>TODO: DEFINE <duration_est> VARIABLE<<<
// >>>TODO: DEFINE <reminder_date> VARIABLE<<<
// >>>TODO: DEFINE <date> VARIABLE<<<
//---------------------------------------------------------  
// ACTION ITEM CHECKBOX TEXT
//---------------------------------------------------------
let task_checkbox;
if (status_value == `done`) {
	task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type_value} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${moment().format(`YYYY-MM-DD`)} 📅 ${date} ✅ ${date}`
} else {
	task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type_value} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${moment().format(`YYYY-MM-DD`)} 📅 ${date}`
};
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// ACTION ITEM CHECKBOX TEXT
//---------------------------------------------------------
let task_checkbox;
if (status_value == `done`) {
	task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type_value} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${moment().format(`YYYY-MM-DD`)} 📅 ${date} ✅ ${date}`
} else {
	task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type_value} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${moment().format(`YYYY-MM-DD`)} 📅 ${date}`
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

1. [[53_00_action_item]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[rename_untitled_file_prompt|Rename Untitled File Prompt]]
2. [[53_00_action_item_tag_type_file_class|Action Item Tag, Type, and File Class]]
3. [[nl_date_and_time|NL Date and Time]]
4. [[52_00_task_times|Task Times]]
5. [[task_status_and_symbol_suggester|Task Status and Symbol]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[54_meeting_checkbox_text|Meeting Checkbox Text]]

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
