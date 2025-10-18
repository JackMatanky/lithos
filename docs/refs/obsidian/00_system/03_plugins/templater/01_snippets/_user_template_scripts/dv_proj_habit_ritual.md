---
title: dv_proj_habit_ritual
aliases:
  - Habits and Rituals Project Tasks by Type Dataview Table
  - Dataview Table of Habits and Rituals Project Tasks by Type
  - habit_ritual_project_tasks_by_type_dv_table
  - dv proj habit ritual
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
date_created: 2023-07-03T11:21
date_modified: 2023-10-25T16:23
tags: obsidian/templater, obsidian/dataview, javascript
---
# Habits and Rituals Project Tasks by Type Dataview Table

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]], [[Dataview]]  
> Language: [[JavaScript]]  
> Input:: Directory Path  
> Output:: Dataview Table  
> Description:: Return a dataview table of a habits and rituals project's tasks according to type. Primarily used to review the number of completed and discarded habits and rituals.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SECT: >>>>> DATA FIELD VARIABLES <<<<<
//---------------------------------------------------------
// regex for task tag, task type, and inline field
const tasl_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;

// Task name and link
const task_link = `link(T.section, regexreplace(regexreplace(T.text, "${tasl_tag_regex}|${task_type_regex}${inline_field_regex}", ""), "_$", "")) 
	AS Task`;

// Task type
const task_type = `choice(contains(T.text, "_action_item"), "🔨Task",
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

// Task status
const task_status = `choice((T.status = "x"),
		"✔️Done",
		"❌Discarded")
	AS Status`;

// Due date
const due_date = `T.due`;
const done_date = `T.completion`;
const task_date = `choice((T.status = "x"),
		${done_date},
		${due_date})
	AS Date`;

// Time span
const start = `T.time_start`;
const end = `T.time_end`;
const time_span = `choice((T.status = "x"),
		(${start} + " - " + ${end}),
		"❌Discarded")
	AS Time`;

// Time estimate
const time_estimate = `(T.duration_est + " min") AS Estimate`;

// Time duration
const task_duration = `dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)))`;
const task_estimate_dur = `dur(T.duration_est + " minutes")`;
const task_estimate_accuracy = `choice((T.status = "x"), 
		(choice((${task_estimate_dur} = ${task_duration}), 
			"👍On Time",
			choice((${task_estimate_dur} > ${task_duration}),
				"🟢" + (${task_estimate_dur} - ${task_duration}),
				"❗" + (${task_duration} - ${task_estimate_dur})))),
		"❌Discarded") 
	AS Accuracy`;

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

// Task checkbox
const checkbox_filter = `regextest("${tasl_tag_regex}", T.text)`;

// Discarded child task status filter
const due_filter = `T.status != " "`;

//---------------------------------------------------------
// SECT: HABIT AND RITUAL PROJECT TASKS DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

// VAR TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"

async function dv_proj_habit_ritual(habit_ritual) {
  const habit_ritual_arg = `${habit_ritual}`;
  const habit_ritual_filter = `contains(T.text, ${habit_ritual_arg})`;

  const dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${task_link},
	${task_status}, 
	${task_date},
	${time_span},
	${time_estimate},
	${task_estimate_accuracy}
FROM
	${template_dir}
FLATTEN
	file.tasks AS T
WHERE
	${folder_filter}
	AND ${checkbox_filter}
	AND ${due_filter}
    AND ${habit_ritual_filter}
SORT 
	T.due,
	T.time_start ASC
${three_backtick}`;

  return dataview_query;
}

module.exports = dv_proj_habit_ritual;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// HABIT AND RITUAL PROJECT TASKS BY TYPE DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
const dv_proj_habit_ritual_table = await tp.user.dv_proj_habit_ritual(habit_ritual);
```

#### Example

```javascript
//---------------------------------------------------------
// HABIT AND RITUAL PROJECT TASKS BY TYPE DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
const habits = await tp.user.dv_proj_habit_ritual("habit");
const morning_rituals = await tp.user.dv_proj_habit_ritual("morning_ritual");
const workday_startup_rituals = await tp.user.dv_proj_habit_ritual("workday_startup_ritual");
const workday_shutdown_rituals = await tp.user.dv_proj_habit_ritual("workday_shutdown_ritual");
const evening_rituals = await tp.user.dv_proj_habit_ritual("evening_ritual");
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_21_proj_habit_ritual_month|Monthly Habits and Rituals Project Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_proj_habit_ritual.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_proj_task|Project Tasks by Status Dataview Table]]
2. [[dv_task_date_status_action(X)|Tasks by Date and Status Dataview Table]]

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
