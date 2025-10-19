---
title: dv_week_habit_rit
aliases:
  - Weekly Habit and Ritual Tasks Dataview Table
  - weekly habit and ritual tasks dataview table
  - Weekly Habit and Ritual Tasks by Status Dataview Table
  - Dataview Table of Weekly Habit and Ritual Tasks by Status
  - habit_rit_tasks_daily_dv_table
  - dv week habit rit
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
date_created: 2023-07-16T12:40
date_modified: 2023-10-25T16:23
tags: obsidian/templater, obsidian/dataview, javascript
---
# Weekly Habit and Ritual Tasks by Status Dataview Table

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]], [[Dataview]]
> Language: [[JavaScript]]
> Input:: Date, String
> Output:: Dataview Table
> Description:: Return a dataview table of weekly habit and ritual tasks according to status. Primarily used in the weekly note.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SECT: >>>>> DATA FIELD VARIABLES <<<<<
//---------------------------------------------------------
// regex for task tag, task type, and inline field
const task_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;

// Task name and link
const task_link = `link(T.section,
		regexreplace(
			regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""),
		"_$", ""))
	AS Task`;

// Task type
const task_type = `choice(contains(T.text, "_action_item"),	"🔨Task",
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
const task_status = `choice((T.status != "-"),
		(choice((T.status = "x"),
			"✔️Done",
			"🔜To do")),
		"❌Discard")
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

// Task parent task
const parent_task = `parent-task AS "Parent Task"`;

// Task project
const project = `Project AS Project`;

//---------------------------------------------------------
// DATA SOURCE
//---------------------------------------------------------
// Template directory
const template_dir = `-"00_system/05_templates"`;

//---------------------------------------------------------
// DATA FILTER
//---------------------------------------------------------
// Task checkbox
const task_checkbox_filter = `regextest("${task_tag_regex}", T.text)`;

// File class filter
const class_filter = `contains(file.frontmatter.file_class, "task_habit_ritual")`;

// Due status filter
const due_filter = `T.status = " "`;

// Due status filter
const not_due_filter = `T.status != " "`;

//---------------------------------------------------------
// DAILY NOTE, PREVIEW, AND REVIEW DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

// VAR TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// VAR STATUS OPTIONS: "due", "done"

async function dv_week_habit_rit({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
}) {
  const type_arg = `${type}`;
  const status_arg = `${status}`;

  const type_filter = `contains(file.frontmatter.type, "${type_arg}")`;
  const date_start_filter = `date(${due_date}) >= date(${date_start})`;
  const date_end_filter = `date(${due_date}) <= date(${date_end})`;

  let dataview_query;
  if (status_arg == "due") {
    // Table for tasks DUE on date
    dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${task_link},
	${start} AS Start,
	${end} AS End,
	${time_estimate},
	${parent_task}
FROM
	${template_dir}
FLATTEN
	file.tasks AS T
WHERE
	${task_checkbox_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${due_filter}
	AND ${date_start_filter}
	AND ${date_end_filter}
SORT
	${due_date},
	${start} ASC
${three_backtick}`;
  } else {
    // Table for tasks COMPLETED or REVIEWING tasks on date
    dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${task_link},
	${task_status},
	${task_date},
	${time_span},
	${time_estimate},
	${task_estimate_accuracy},
	${parent_task}
FROM
	${template_dir}
FLATTEN
	file.tasks AS T
WHERE
	${task_checkbox_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${not_due_filter}
	AND ${date_start_filter}
	AND ${date_end_filter}
SORT
	${due_date},
	${start} ASC
${three_backtick}`;
  }
  return dataview_query;
}

module.exports = dv_week_habit_rit;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// HABIT AND RITUALS DATAVIEW TABLES
//---------------------------------------------------------
// TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// STATUS OPTIONS: "due", "done"
const dataview_habit_rit_table = await tp.user.dv_week_habit_rit({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
});
```

#### Examples

```javascript
//---------------------------------------------------------
// HABIT AND RITUALS DATAVIEW TABLES
//---------------------------------------------------------
// >>>>> WEEKLY NOTE <<<<<
// TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// STATUS OPTIONS: "due", "done"
const week_habit_due = await tp.user.dv_week_habit_rit({
  type: "habit",
  status: "due",
  start_date: date_start,
  end_date: date_end,
});

const week_morn_rit_due = await tp.user.dv_week_habit_rit({
  type: "morning_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
});

const week_work_start_due = await tp.user.dv_week_habit_rit({
  type: "workday_startup_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
});

const week_work_shut_due = await tp.user.dv_week_habit_rit({
  type: "workday_shutdown_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
});

const week_eve_rit_due = await tp.user.dv_week_habit_rit({
  type: "evening_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
});

const week_habit_done = await tp.user.dv_week_habit_rit({
  type: "habit",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});

const week_morn_rit_done = await tp.user.dv_week_habit_rit({
  type: "morning_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});

const week_work_start_done = await tp.user.dv_week_habit_rit({
  type: "workday_startup_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});

const week_work_shut_done = await tp.user.dv_week_habit_rit({
  type: "workday_shutdown_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});

const week_eve_rit_done = await tp.user.dv_week_habit_rit({
  type: "evening_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_00_week|Weekly Calendar Template]]
2. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_week_habit_rit.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_proj_task|Project Tasks by Status Dataview Table]]

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
