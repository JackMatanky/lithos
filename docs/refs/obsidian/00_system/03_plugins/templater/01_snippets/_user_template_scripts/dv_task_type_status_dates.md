---
title: dv_task_type_status_dates
aliases:
  - Tasks and Events by Type, Status, and Dates Dataview Tables
  - tasks and events by type, status, and dates dataview table
  - Dataview Tables for Tasks and Events by Type, Status, and Dates
  - dataview tables for tasks and events by type, status, and dates
  - dv task type status dates
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-24T07:50
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Tasks and Events by Type, Status, and Dates Dataview Tables

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a dataview tables for project, parent tasks, child tasks, or habit and ritual child task by types, status, date, or between dates.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// SECT: >>>>> PROJECT AND PARENT TASK FIELDS <<<<<
// Title
const yaml_title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias})`;

// Title link for DV markdown query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]"`;

// File type
const yaml_type = `file.frontmatter.type`;
const file_type = `choice(contains(${yaml_type}, "project"), "🏗️Project", "⚒️Parent Task") AS Type`;

// File status
const yaml_status = `file.frontmatter.status`;
const proj_status = `choice(contains(${yaml_status}, "done"), "✔️Done",
	choice(contains(${yaml_status}, "in_progress"), "👟In progress",
	choice(contains(${yaml_status}, "to_do"), "🔜To do",
	choice(contains(${yaml_status}, "schedule"), "📅Schedule",
	choice(contains(${yaml_status}, "on_hold"), "🤌On hold", "❌Discarded")))))
	AS Status`;

// Date span
const yaml_date_start = `file.frontmatter.task_start`;
const yaml_date_end = `file.frontmatter.task_end`;
const date_span = `choice((regextest(".", ${yaml_date_start}) AND regextest(".", ${yaml_date_end})),
		(${yaml_date_start} + " → " + ${yaml_date_end}),
		choice(regextest(".", ${yaml_date_start}),
			(${yaml_date_start} + " → Present"),
			"null"))
	AS Dates`;

// Context
const context = `Context AS Context`;

// Project
const yaml_proj = `file.frontmatter.project`;
const project = `Project AS Project`;

// Parent Task field
const yaml_parent_task = `file.frontmatter.parent_task`;
const parent_task = `parent-task AS "Parent Task"`;

// Organization
const org = `Organization AS Organization`;

// Contact
const contact = `Contact AS Contact`;

// Objective statement
const objective = `Objective AS Objective`;

// Outcome
const outcome = `outcome AS Result`;

// File creation date
const yaml_creation_date = `file.frontmatter.date_created`;

// Tags
const tags = `file.etags AS Tags`;

// SECT: >>>>> CHILD TASK FIELD VARIABLES <<<<<
// regex for task tag, task type, and inline field
const task_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;

// Task link
const task_link = `link(T.section, regexreplace(regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""), "_$", "")) AS Task`;

// Title link for DV markdown query
const md_task_link = `"[[" + file.name + "#" + regexreplace(string(L.section), ".+>|\]\]$", "") + "\|" + regexreplace(regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""), "_$", "") + "]]" AS Task`;

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
    choice(contains(T.text, "_morning_ritual"),    "🍵Rit.",
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

// Due or completed date
const task_creation = "T.created";
const task_due = "T.due";
const task_done = "T.completion";
const due_date = `dateformat(${task_due}, "yy-MM-dd")`;
const done_date = `dateformat(${task_done}, "yy-MM-dd")`;
const task_date = `choice((T.status != "-"),
        (choice((T.status = "x"),
            ${done_date},
            ${due_date})),
        "❌Discard")
    AS Date`;

// Time span
const task_start = `T.time_start`;
const task_end = `T.time_end`;
const time_span = `(${task_start} + " - " + ${task_end}) AS Time`;

// Time estimate
const task_estimate = `(T.duration_est + " min") AS Estimate`;

// Time duration
const task_duration = `dur((date(dateformat(${task_done}, "yyyy-MM-dd") + "T" + ${task_end})) - (date(dateformat(${task_done}, "yyyy-MM-dd") + "T" + ${task_start})))`;
const task_estimate_dur = `dur(T.duration_est + " minutes")`;
const task_estimate_accuracy = `choice((T.status = "x"),
    (choice((${task_estimate_dur} = ${task_duration}),
      "👍On Time",
      choice((${task_estimate_dur} > ${task_duration}),
	  	  ("🟢" + (${task_estimate_dur} - ${task_duration})),
	  	  ("❗" + (${task_duration} - ${task_estimate_dur})))),
	  "❌Discarded")
	  AS Accuracy`;

//---------------------------------------------------------
// SECT: >>>>> DATA SOURCES <<<<<
//---------------------------------------------------------
// Projects directory
const proj_dir = `"40_projects"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//---------------------------------------------------------
// SECT: >>>>> PROJECT AND PARENT TASK FILTERS <<<<<
// Task file class filter
const class_filter = `contains(file.frontmatter.file_class, "task")`;

// Schedule or undetermined status filter
const proj_undetermined_filter = `(contains(${yaml_status}, "undetermined")
	OR contains(${yaml_status}, "schedule"))`;

// SECT: >>>>> CHILD TASK FILTERS <<<<<
// Task checkbox
const task_checkbox_filter = `regextest("${task_tag_regex}", T.text)`;

// Task due filter
const task_due_filter = `T.status = " "`;

// Task completed filter
const task_done_filter = `T.completed`;

// Discarded status filter
const task_discard_filter = `T.status != "-"`;

//---------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//---------------------------------------------------------
const status_sort = `choice(${yaml_status} = "done", 1,
	choice(${yaml_status} = "in_progress", 2,
	choice(${yaml_status} = "to_do", 3,
	choice(${yaml_status} = "schedule", 4,
	choice(${yaml_status} = "on_hold", 5, 6)))))`;

//---------------------------------------------------------
// DATAVIEW TABLE FOR TASKS AND EVENTS
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR: TYPES: "project", "parent_task", "child_task", "task", "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// VAR COMPLETED STATUSES: "completed", "done"
// VAR ACTIVE STATUSES: "active", "to_do", "in_progress"
// VAR SCHEDULE STATUSES: "schedule", "on_hold"
// VAR CREATED STATUSES: "new", "created",
// VAR DETERMINE STATUSES: "undetermined", "determine", "schedule"

async function dv_task_type_status_dates({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  const type_arg = `${type}`;
  const status_arg = `${status}`;
  const start_date_arg = `${date_start}`;
  const end_date_arg = `${date_end}`;
  const md_arg = `${md}`;

  // TABLE DATA FIELDS
  let data_field;
  if (
    status_arg.startsWith("don") ||
    status_arg.startsWith("comp") ||
    status_arg.startsWith("rev")
  ) {
    // Data fields for COMPLETED, DONE, or REVIEW statuses
    if (type_arg.startsWith("pro")) {
      // PROJECT

      data_field = `${title_link} AS Project,
      ${date_span},
      ${context},
      ${objective},
      ${outcome}`;
    } else if (type_arg.startsWith("par")) {
      // PARENT TASK

      data_field = `${title_link} AS "Parent Task",
      ${date_span},
      ${objective},
      ${outcome},
      ${project}`;
    } else if (type_arg.startsWith("task") || type_arg.startsWith("child")) {
      // CHILD TASK

      data_field = `${task_link},
      ${task_type},
      ${time_span},
      ${task_estimate},
      ${task_estimate_accuracy},
      ${project}`;
    } else {
      // HABIT AND RITUAL CHILD TASKS

      data_field = `${task_link},
      ${task_status},
      ${task_date},
      ${time_span},
      ${task_estimate},
      ${task_estimate_accuracy},
      ${parent_task}`;
    }
  } else if (status_arg.startsWith("act") || status_arg.startsWith("due")) {
    if (type_arg.startsWith("pro")) {
      // PROJECT
      data_field = `${title_link} AS Project,
      ${proj_status},
      ${date_span},
      ${context},
      ${objective}`;
    } else if (type_arg.startsWith("par")) {
      // PARENT TASK
      data_field = `${title_link} AS "Parent Task",
      ${proj_status},
      ${date_span},
      ${objective},
      ${project}`;
    } else if (type_arg.startsWith("task") || type_arg.startsWith("child")) {
      data_field = `${task_link},
      ${task_type},
      ${task_start} AS Start,
      ${task_end} AS End,
      ${project}`;
    } else {
      data_field = `${task_link},
      ${due_date} AS Date,
      ${task_start} AS Start,
      ${task_end} AS End,
      ${task_estimate},
      ${parent_task}`;
    }
  } else if (status_arg.startsWith("pre")) {
    if (type_arg.startsWith("task") || type_arg.startsWith("child")) {
      data_field = `${task_link},
      ${task_type},
      ${due_date} AS Date,
      ${task_start} AS Start,
      ${task_end} AS End,
      ${project}`;
    }
  } else if (status_arg.startsWith("new") || status_arg.startsWith("cre")) {
    if (type_arg.startsWith("pro")) {
      // PROJECT
      data_field = `${title_link} AS Project,
      ${proj_status},
      ${date_span},
      ${context},
      ${objective}`;
    } else if (type_arg.startsWith("par")) {
      // PARENT TASK
      data_field = `${title_link} AS "Parent Task",
      ${proj_status},
      ${date_span},
      ${objective},
      ${project}`;
    } else if (type_arg.startsWith("task") || type_arg.startsWith("child")) {
      data_field = `${task_link},
      ${task_type},
      ${task_start} AS Start,
      ${task_end} AS End,
      ${project}`;
    }
  }

  // DATE FIELD FOR DATE FILTER
  let date_field;
  if (type_arg.startsWith("par") || type_arg.startsWith("pro")) {
    // DATE FIELDS for PROJECT and PARENT TASKS
    if (status_arg.startsWith("don") || status_arg.startsWith("comp")) {
      // DATE field for COMPLETED projects and parent tasks
      date_field = yaml_date_end;
    } else if (status_arg.startsWith("new") || status_arg.startsWith("cre")) {
      // DATE field for NEW projects and parent tasks
      date_field = yaml_creation_date;
    }
  } else if (
    type_arg.startsWith("task") ||
    type_arg.startsWith("child") ||
    type_arg.startsWith("hab") ||
    type_arg.startsWith("morn") ||
    type_arg.startsWith("work") ||
    type_arg.startsWith("eve")
  ) {
    // DATE FIELDS for CHILD TASKS
    if (
      status_arg.startsWith("don") ||
      status_arg.startsWith("comp") ||
      status_arg.startsWith("rev")
    ) {
      // DATE field for COMPLETED child tasks
      date_field = task_done;
    } else if (status_arg.startsWith("due") || status_arg.startsWith("pre")) {
      // DATE field for DUE or PREVIEW child tasks
      date_field = task_due;
    } else if (status_arg.startsWith("new") || status_arg.startsWith("cre")) {
      // DATE field for NEW child tasks
      date_field = task_creation;
    }
  }

  // DATE FILTER
  let date_filter = "null";
  if (end_date_arg == "" || end_date_arg == "week") {
    // SPECIFIC date
    date_filter = `date(${date_field}) = date(${start_date_arg})`;
  } else {
    // BETWEEN two dates
    date_filter = `date(${date_field}) >= date(${start_date_arg})
    AND date(${date_field}) <= date(${end_date_arg})`;
  }

  // FILTER
  let filter;
  if (type_arg.startsWith("par") || type_arg.startsWith("pro")) {
    // FILTERs for PROJECTS and PARENT TASKS

    if (status_arg.startsWith("don") || status_arg.startsWith("comp")) {
      // FILTER for COMPLETED projects and parent tasks

      filter = `contains(${yaml_type}, "${type_arg}")
      AND contains(${yaml_status}, "done")
      AND ${date_filter}`;
    } else if (status_arg.startsWith("act")) {
      // FILTER for ACTIVE projects and parent tasks

      filter = `contains(${yaml_type}, "${type_arg}")
      AND (contains(${yaml_status}, "to_do")
      OR contains(${yaml_status}, "in_progress"))`;
    } else if (status_arg.startsWith("sche") || status_arg.startsWith("on")) {
      // FILTER for projects and parent tasks to SCHEDULE or ON HOLD

      filter = `contains(${yaml_type}, "${type_arg}")
      AND (contains(${yaml_status}, "schedule")
      OR contains(${yaml_status}, "on_hold"))`;
    } else if (status_arg.startsWith("und") || status_arg.startsWith("det")) {
      // FILTER for UNDETERMINED projects and parent tasks

      filter = `contains(${yaml_type}, "${type_arg}")
      AND contains(${yaml_status}, "undetermined")`;
    } else if (status_arg.startsWith("new") || status_arg.startsWith("cre")) {
      // FILTER for NEW projects and parent tasks

      filter = `contains(${yaml_type}, "${type_arg}")
      AND ${date_filter}`;
    } else {
      filter = `contains(${yaml_type}, "${type_arg}")
      AND contains(${yaml_status}, "${status_arg}")`;
    }
  }
  if (type_arg.startsWith("task") || type_arg.startsWith("child")) {
    if (
      status_arg.startsWith("don") ||
      status_arg.startsWith("comp") ||
      status_arg.startsWith("rev")
    ) {
      filter = `${task_checkbox_filter}
      AND ${task_discard_filter}
      AND T.completed
      AND ${date_filter}`;
    } else if (status_arg.startsWith("due") && end_date_arg == "week") {
      filter = `${task_checkbox_filter}
      AND ${date_filter}`;
    } else if (status_arg.startsWith("due")) {
      filter = `${task_checkbox_filter}
      AND ${task_due_filter}
      AND ${date_filter}`;
    } else if (status_arg.startsWith("pre")) {
      filter = `${task_checkbox_filter}
      AND ${task_discard_filter}
      AND ${date_filter}
      OR (${task_checkbox_filter}
      AND ${task_discard_filter}
      AND ${task_due_filter}
      AND ${task_due} < date(${start_date_arg}))`;
    } else if (status_arg.startsWith("new") || status_arg.startsWith("cre")) {
      filter = `${task_checkbox_filter}
      AND ${task_discard_filter}
      AND ${date_filter}`;
    }
  } else if (
    type_arg.startsWith("hab") ||
    type_arg.startsWith("morn") ||
    type_arg.startsWith("work") ||
    type_arg.startsWith("eve")
  ) {
    if (
      status_arg.startsWith("don") ||
      status_arg.startsWith("comp") ||
      status_arg.startsWith("rev")
    ) {
      filter = `contains(${yaml_type}, "${type_arg}")
      AND ${task_checkbox_filter}
      AND !${task_due_filter}
      AND ${date_filter}`;
    } else if (status_arg.startsWith("due") && end_date_arg != "") {
      filter = `contains(${yaml_type}, "${type_arg}")
      AND ${task_checkbox_filter}
      AND ${date_filter}`;
    } else if (status_arg.startsWith("due")) {
      filter = `contains(${yaml_type}, "${type_arg}")
      AND ${task_checkbox_filter}
      AND ${task_due_filter}
      AND ${date_filter}`;
    } else if (status_arg.startsWith("pre")) {
      filter = `${task_checkbox_filter}
      AND ${task_discard_filter}
      AND ${date_filter}
      OR (${task_checkbox_filter}
      AND ${task_discard_filter}
      AND ${task_due_filter}
      AND ${task_due} < date(${start_date_arg}))`;
    } else if (status_arg.startsWith("new") || status_arg.startsWith("cre")) {
      filter = `${task_checkbox_filter}
      AND ${task_discard_filter}
      AND ${date_filter}`;
    }
  }

  // SORT fields by status
  let sort_field;
  if (status_arg.startsWith("don") || status_arg.startsWith("comp")) {
    if (type_arg.startsWith("pro")) {
      // PROJECT
      sort_field = `${yaml_date_end},
      ${yaml_title}`;
    } else if (type_arg.startsWith("par")) {
      sort_field = `${yaml_proj},
      ${yaml_date_end},
      ${yaml_title}`;
    }
  } else if (
    status_arg.startsWith("act") ||
    status_arg.startsWith("sche") ||
    status_arg.startsWith("on") ||
    status_arg.startsWith("und") ||
    status_arg.startsWith("det") ||
    status_arg.startsWith("new") ||
    status_arg.startsWith("cre")
  ) {
    if (type_arg.startsWith("pro")) {
      // PROJECT
      sort_field = `${status_sort},
      ${yaml_date_start},
      ${yaml_title}`;
    } else if (type_arg.startsWith("par")) {
      sort_field = `${status_sort},
      ${yaml_proj},
      ${yaml_date_start},
      ${yaml_title}`;
    }
  } else if (type_arg.startsWith("task") || type_arg.startsWith("child")) {
    sort_field = `${task_due},
    ${task_start}`;
  }

  let dataview_query;
  // >>>>> TASK TABLES <<<<<
  if (type_arg.startsWith("pro") || type_arg.startsWith("par")) {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
	  ${proj_dir}
WHERE
	  ${class_filter}
    AND ${filter}
SORT
    ${sort_field} ASC
${three_backtick}`;
  } else {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
	  ${proj_dir}
FLATTEN
    file.tasks AS T
WHERE
	  ${class_filter}
    AND ${filter}
SORT
    ${sort_field} ASC
${three_backtick}`;
  }

  if (md_arg == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    let md_query;

    if (type_arg.startsWith("pro") || type_arg.startsWith("par")) {
      md_query = String(
        dataview_query
          .replace(dataview_block_start_regex, "")
          .replace(dataview_block_end_regex, "")
          .replaceAll(/\n\s+/g, " ")
          .replaceAll(/\n/g, " ")
          .replace(title_link, md_title_link)
      );
    } else {
      md_query = String(
        dataview_query
          .replace(dataview_block_start_regex, "")
          .replace(dataview_block_end_regex, "")
          .replaceAll(/\n\s+/g, " ")
          .replaceAll(/\n/g, " ")
          .replace(task_link, md_task_link)
      );
    }

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
  }

  return dataview_query;
}

module.exports = dv_task_type_status_dates;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// TASKS AND EVENTS DATAVIEW TABLES
//---------------------------------------------------------
// TASKS: "project", "parent_task", "child_task", "task", "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// COMPLETED STATUSES: "completed", "done"
// ACTIVE STATUSES: "active", "to_do", "in_progress"
// SCHEDULE STATUSES: "schedule", "on_hold"
// DETERMINE STATUSES: "undetermined", "determine"
// CREATED STATUSES: "created", "new"
const dataview_task_table = await tp.user.dv_task_type_status_dates({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
});
```

#### Examples

```javascript
//---------------------------------------------------------
// PROJECTS BY STATUS AND DATES DATAVIEW TABLES
//---------------------------------------------------------
const week_active_proj = await tp.user.dv_week_task({
  type: "project",
  status: "active",
  start_date: "",
  end_date: "",
  md: "false",
});
const week_comp_proj = await tp.user.dv_week_task({
  type: "project",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

//---------------------------------------------------------
// PARENT TASKS BY STATUS AND DATES DATAVIEW TABLES
//---------------------------------------------------------
const week_active_parent_task = await tp.user.dv_week_task({
  type: "parent_task",
  status: "active",
  start_date: "",
  end_date: "",
  md: "false",
});
const week_comp_parent_task = await tp.user.dv_week_task({
  type: "parent_task",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

//---------------------------------------------------------
// WEEKLY CHILD TASKS BY STATUS AND DATE DATAVIEW TABLES
//---------------------------------------------------------
const tasks_sunday_due = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: sunday,
  end_date: "",
  md: "false",
});

const tasks_monday_due = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: monday,
  end_date: "",
  md: "false",
});

const tasks_tuesday_due = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: tuesday,
  end_date: "",
  md: "false",
});

const tasks_wednesday_due = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: wednesday,
  end_date: "",
  md: "false",
});

const tasks_thursday_due = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: thursday,
  end_date: "",
  md: "false",
});

const tasks_friday_due = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: friday,
  end_date: "",
  md: "false",
});

const tasks_saturday_due = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: saturday,
  end_date: "",
  md: "false",
});

const tasks_sunday_done = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: sunday,
  end_date: "",
  md: "false",
});

const tasks_monday_done = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: monday,
  end_date: "",
  md: "false",
});

const tasks_tuesday_done = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: tuesday,
  end_date: "",
  md: "false",
});

const tasks_wednesday_done = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: wednesday,
  end_date: "",
  md: "false",
});

const tasks_thursday_done = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: thursday,
  end_date: "",
  md: "false",
});

const tasks_friday_done = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: friday,
  end_date: "",
  md: "false",
});

const tasks_saturday_done = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: saturday,
  end_date: "",
  md: "false",
});

//---------------------------------------------------------
// PARENT TASKS BY STATUS AND DATES DATAVIEW TABLES
//---------------------------------------------------------
const week_habit_due = await tp.user.dv_task_type_status_dates({
  type: "habit",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_morn_rit_due = await tp.user.dv_task_type_status_dates({
  type: "morning_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_work_start_due = await tp.user.dv_task_type_status_dates({
  type: "workday_startup_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_work_shut_due = await tp.user.dv_task_type_status_dates({
  type: "workday_shutdown_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_eve_rit_due = await tp.user.dv_task_type_status_dates({
  type: "evening_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_habit_done = await tp.user.dv_task_type_status_dates({
  type: "habit",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_morn_rit_done = await tp.user.dv_task_type_status_dates({
  type: "morning_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_work_start_done = await tp.user.dv_task_type_status_dates({
  type: "workday_startup_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_work_shut_done = await tp.user.dv_task_type_status_dates({
  type: "workday_shutdown_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_eve_rit_done = await tp.user.dv_task_type_status_dates({
  type: "evening_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
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

1. [[31_01_day_periodic|Daily Calendar Periodic Note Template]]
2. [[31_00_day|Daily Calendar Template]]
3. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
4. [[32_00_week|Weekly Calendar Template]]
5. [[32_02_week_days|Weekly and Weekdays Calendar Template]]
6. [[55_21_daily_morn_rit|Daily Morning Ritual Task Template]]
7. [[55_22_today_morn_rit|Daily Morning Ritual Task Button Template]]
8. [[55_23_tomorrow_morn_rit|Tomorrow Morning Ritual Task Button Template]]
9. [[55_41_daily_work_shut_rit|Daily Workday Shutdown Ritual Task Template]]
10. [[55_42_today_work_shut_rit|Daily Workday Shutdown Ritual Task Button Template]]
11. [[55_43_tomorrow_work_shut_rit|Tomorrow Workday Shutdown Ritual Task Button Template]]
12. [[55_51_daily_eve_rit|Daily Evening Ritual Task Template]]
13. [[55_52_today_eve_rit|Daily Evening Ritual Task Button Template]]
14. [[55_53_tomorrow_eve_rit|Tomorrow Evening Ritual Task Button Template]]
15. [[32_08_cal_week_habit_ritual|Weekly Habits and Rituals Dataview Tables]]
16. [[32_09_cal_week_task_event|Weekly Tasks and Events Dataview Tables]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_task_type_status_dates.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_task_linked|Linked Tasks and Goals Files Dataview Table]]
2. [[dv_lib_status_dates|Library Content By Status and Dates Dataview Table]]
3. [[dv_pdev_attr_dates|Journals and Attributes Between Dates Dataview Table]]
4. [[dv_pdev_attr_dates|Journals and Attributes Between Dates Dataview Table]]

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
