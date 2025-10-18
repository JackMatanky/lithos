---
title: dv_proj_task
aliases:
  - Project Tasks by Status Dataview Table
  - Dataview Table of Project Tasks by Status
  - project_tasks_by_status_dv_table
  - dv_proj_task_status
  - dv_proj_task
plugin:
  - templater
  - dataview
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-11T13:37
date_modified: 2023-10-25T16:23
tags: obsidian/templater, obsidian/dataview, javascript
---
# Project Tasks by Status Dataview Table

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]], [[Dataview]]  
> Language: [[JavaScript]]  
> Input:: Directory Path  
> Output:: Dataview Table  
> Description:: Return a dataview table of a project's tasks according to type and status.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SECT: >>>>> PARENT TASK FIELD VARIABLES <<<<<
//---------------------------------------------------------
// Parent task title name and link
const parent_task_link = `link(file.link, file.frontmatter.aliases[0]) AS "Parent Task"`;
const parent_task_value_link = `${new_line}${ul_yaml}"${parent_task_link}"`;

// Status
const yaml_status = `file.frontmatter.status`;
const parent_task_status = `choice(${yaml_status} = "done",
	"✔️Done",
	choice(${yaml_status} = "in_progress",
		"👟In progress",
		"🔜To do"))
	AS Status`;

// Date span
const date_start = `file.frontmatter.task_start`;
const date_end = `file.frontmatter.task_end`;
const date_span = `choice((regextest(".", ${date_start}) AND regextest(".", ${date_end})), 
		(${date_start} + " → " + ${date_end}),
		choice(regextest(".", ${date_start}),
			(${date_start} + " → Present"),
			"null")) 
	AS Dates`;

const contact = `Contact AS Contact`;

//---------------------------------------------------------
// SECT: >>>>> CHILD TASK FIELD VARIABLES <<<<<
//---------------------------------------------------------
// regex for task tag, task type, and inline field
const child_tag_regex = `(#task)`;
const child_type_regex = `(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;

// Task name and link
const child_link = `link(T.section,
		regexreplace(
			regexreplace(T.text, "${child_tag_regex}|${child_type_regex}${inline_field_regex}", ""),
		"_$", "")) 
	AS Task`;

// Task type
const child_type = `choice(contains(T.text, "_action_item"), "🔨Task",
	choice(contains(T.text, "_meeting"), "🤝Meeting",
	choice(contains(T.text, "_phone_call"), "📞Call",
	choice(contains(T.text, "_interview"), "💼Interview",
	choice(contains(T.text, "_appointment"), "⚕️Appointment",
	choice(contains(T.text, "_event"), "🎊Event",
	choice(contains(T.text, "_gathering"), "✉️Gathering",
	choice(contains(T.text, "_hangout"), "🍻Hangout",
	choice(contains(T.text, "_habit"), "🤖Habit",
	choice(contains(T.text, "_morning_ritual"),	"🍵Rit.",
	choice(contains(T.text, "_workday_startup_ritual"), "🌇Rit.",
	choice(contains(T.text, "_workday_shutdown_ritual"), "🌆Rit.", "🛌Rit.")))))))))))) 
	AS Type`;

// Due date
const due_date = `T.due AS Due`;
const done_date = `T.completion AS Date`;

// Time span
const start = `T.time_start`;
const end = `T.time_end`;
const time_span = `(${start} + " - " + ${end}) AS Time`;

// Time estimate
const child_estimate = `(T.duration_est + " min") AS Estimate`;

// Time duration
const child_duration = `dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)))`;
const child_estimate_dur = `dur(T.duration_est + " minutes")`;
const child_estimate_accuracy = `choice((${child_estimate_dur} = ${child_duration}),
	"👍On Time",
	choice(
		(${child_estimate_dur} > ${child_duration}),
			"🟢" + (${child_estimate_dur} - ${child_duration}),
			"❗" + (${child_duration} - ${child_estimate_dur}))) 
AS Accuracy`;

// Parent Task field
const parent_task = `file.frontmatter.parent_task AS "Parent Task"`;

//---------------------------------------------------------
// SECT: DATA SOURCE
//---------------------------------------------------------
// Template directory
const template_dir = `-"00_system/05_templates"`;

//---------------------------------------------------------
// SECT: DATA FILTER
//---------------------------------------------------------
// Same directory
const folder_filter = `contains(file.path, this.file.folder)`;

// Parent task type filter
const parent_type_filter = `contains(file.frontmatter.type, "parent")`;

// Discarded parent task status filter
const parent_discard_filter = `file.frontmatter.status != "discarded"`;

// Task checkbox
const child_checkbox_filter = `regextest("${child_tag_regex}", T.text)`;

// Discarded child task status filter
const child_discard_filter = `T.status != "-"`;

// Task completion filter
const status_filter = `T.completed`;

//---------------------------------------------------------
// SECT: PROJECT TASKS DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

// VAR TYPES: "parent", "child"
// VAR STATUS: "due", "done", "null"

async function dv_proj_task(type, status) {
  const type_arg = `${type}`;
  const status_arg = `${status}`;

  let dataview_query;
  if (type_arg != "parent") {
    if (status_arg == `due`) {
      // DUE Table for remaining project tasks
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${child_link},
	${child_type}, 
	${due_date},
	${start} AS Start,
	${end} AS End,
	${child_estimate},
	${parent_task}
FROM
	${template_dir}
FLATTEN
	file.tasks AS T
WHERE
	${folder_filter}
	AND ${child_checkbox_filter}
	AND ${child_discard_filter}
	AND !${status_filter}
SORT 
	T.due,
	T.time_start ASC
${three_backtick}`;
    } else if (status_arg == `done`) {
      // Table for COMPLETED project tasks
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${child_link},
	${child_type},
	${done_date},
	${time_span},
	${child_estimate},
	${child_estimate_accuracy},
	${parent_task}
FROM
	${template_dir}
FLATTEN 
	file.tasks AS T
WHERE
	${folder_filter}
	AND ${child_checkbox_filter}
	AND ${child_discard_filter}
	AND ${status_filter}
SORT 
	T.completion,
	T.time_start ASC
${three_backtick}`;
    }
  } else {
    // Project PARENT TASK Table
    dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${parent_task_link},
	${parent_task_status},
	${date_span},
	${contact}
FROM
	${template_dir}
WHERE
	${folder_filter}
	AND ${parent_type_filter}
	AND ${parent_discard_filter}
SORT 
	date_start ASC
${three_backtick}`;
  }
  return dataview_query;
}

module.exports = dv_proj_task;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// PROJECT TASKS BY STATUS DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "parent", "child"
// STATUS: "due", "done", "null"
const dv_proj_task_table = await tp.user.dv_proj_task(type, status);
```

#### Example

```javascript
//---------------------------------------------------------
// PROJECT TASKS BY STATUS DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "parent", "child"
// STATUS: "due", "done", "null"
const proj_parent_tasks = await tp.user.dv_proj_task("parent", "null");
const child_task_remaining = await tp.user.dv_proj_task("child", "due");
const child_task_completed = await tp.user.dv_proj_task("child", "done");
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
6. [[50_22_proj_habit_ritual_month_quickadd|Monthly Habits and Rituals Project QuickAdd Template]]
7. [[50_30_proj_education|Education Project Template]]
8. [[50_31_proj_ed_course(X)|Education Course Project Template]]
9. [[50_32_proj_ed_book|Education Book Project Template]]
10. [[50_40_proj_professional|Professional Project Template]]
11. [[50_50_proj_work|Work Project Template]]
12. [[51_00_parent_task|General Parent Task Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_proj_task.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_task_date_status_action(X)|Tasks by Date and Status Dataview Table]]

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
