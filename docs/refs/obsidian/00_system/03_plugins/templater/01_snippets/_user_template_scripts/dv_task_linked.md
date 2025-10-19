---
title: dv_task_linked
aliases:
  - Linked Tasks and Goals Files Dataview Table
  - Dataview Table for Linked Tasks and Goals Files
  - dv task linked
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-12T13:46
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Linked Tasks and Goals Files Dataview Table

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a dataview table or markdown table for linked task and goal files based on type, status, and relation.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// SECT: >>>>> GENERAL FIELDS <<<<<
// File name
const file_name = "file.name";

// Title
const yaml_title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// File type
const yaml_type = `file.frontmatter.type`;

// File class
const yaml_class = `file.frontmatter.file_class`;

// Context
const context = `Context AS Context`;

// Organization
const org = `Organization AS Organization`;

// Contact
const contact = `Contact AS Contact`;

// Objective statement
const objective = `Objective AS Objective`;

// Outcome and emotion
const outcome = `list(("Outcome: " + Outcome), ("Emotion: " + outcome-emotion)) AS Result`;

// Tags
const tags = `file.etags AS Tags`;

// SECT: >>>>> PROJECT AND PARENT TASK FIELDS <<<<<
// Title link
const title_link = `link(file.link, ${alias})`;

// Title link for DV markdown query
const md_title_link = `"[[" + ${file_name} + "\|" + ${alias} + "]]"`;

// Project and parent task file type
const par_file_type = `choice(contains(${yaml_type}, "project"), "🏗️Project", "⚒️Parent Task") AS Type`;

// File status
const yaml_status = `file.frontmatter.status`;
const file_status = `choice(contains(${yaml_status}, "done"), "✔️Done",
	choice(contains(${yaml_status}, "in_progress"), "👟In progress",
	choice(contains(${yaml_status}, "to_do"), "🔜To do",
	choice(contains(${yaml_status}, "schedule"), "📅Schedule",
	choice(contains(${yaml_status}, "on_hold"), "🤌On hold", "❌Discarded")))))
	AS Status`;

// Date span
const yaml_date_start = `file.frontmatter.task_start`;
const yaml_date_end = `file.frontmatter.task_end`;
const date_span = `choice((regextest("\d", ${yaml_date_start}) AND regextest("\d", ${yaml_date_end})),
		(${yaml_date_start} + " → " + ${yaml_date_end}),
		choice(regextest("\d", ${yaml_date_start}),
			(${yaml_date_start} + " → Present"),
			"null"))
	AS Dates`;

// Project
const yaml_proj = `file.frontmatter.project`;
const project = `Project AS Project`;

// Parent Task field
const yaml_parent_task = `file.frontmatter.parent_task`;
const parent_task = `parent-task AS "Parent Task"`;

// SECT: >>>>> CHILD TASK FIELDS <<<<<
// Task tag regex
const task_tag_regex = "(#task)";

// Task type regex
const task_type_regex = `(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;

// Inline data field regex
const inline_field_regex = `\\[.*$`;

// Task page section regex
const task_sect_regex = `.+>|\\]\\]$`;

// Task page section
const task_sect = "T.section";

// Task title
const task_title = `regexreplace(regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""), "_$", "")`;

// Task link
const task_link = `link(${task_sect}, ${task_title}) AS Task`;

// Task section shortened
const task_sect_short = `regexreplace(regexreplace(string(${task_sect}), "${task_sect_regex}", ""), "^ ", "")`;

// Title link for DV markdown query
const md_task_link = `"[[" + ${file_name} + "#" + ${task_sect_short} + "\|" + regexreplace(${task_title}, "^ ", "") + "]]" AS Task`;

// Task type
const task_type = `choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type`;

// Task status
const task_status = `choice((T.status != "-"),
        (choice((T.status = "x"), "✔️Done", "🔜To do")),
        "❌Discard")
    AS Status`;

// Due or completed date
const task_due = "T.due";
const task_done = "T.completion";
const due_date = `dateformat(${task_due}, "yy-MM-dd")`;
const done_date = `dateformat(${task_done}, "yy-MM-dd")`;
const task_date = `choice((T.status != "-"),
        (choice((T.status = "x"), ${done_date}, ${due_date})),
        "❌Discard")
    AS Date`;

// Time span
const task_start = `T.time_start`;
const task_end = `T.time_end`;
const time_span = `(${task_start} + " - " + ${task_end}) AS Time`;

// Time duration estimate
const task_duration_est = `dur(choice(T.duration_est < 60, T.duration_est + " m",
    choice((T.duration_est % 60) = 0, (T.duration_est/60) + " h",
    (T.duration_est % 60) + " m " + floor(T.duration_est/60) + " h")))`;
const task_estimate = `${task_duration_est} AS Estimate`;

// Time duration
const full_task_start = `date(dateformat(${task_done}, "yyyy-MM-dd") + "T" + ${task_start})`;
const full_task_end = `date(dateformat(${task_done}, "yyyy-MM-dd") + "T" + ${task_end})`;
const task_duration_act = `dur((${full_task_end}) - (${full_task_start}))`;

const task_est_accuracy = `choice(T.status = "x",
    (choice((${task_duration_est} = ${task_duration_act}), "👍On Time",
        choice((${task_duration_est} > ${task_duration_act}),
	  	    ("🟢" + (${task_duration_est} - ${task_duration_act}) + "➖"),
	  	    ("❗" + (${task_duration_act} - ${task_duration_est}) + "➕")))
        + "(" + ${task_duration_est} + ")"),
	"❌Discarded")
	AS Accuracy`;

//---------------------------------------------------------
// DATA SOURCE
//---------------------------------------------------------
// Project directory
const proj_dir = `"40_projects"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTER <<<<<
//---------------------------------------------------------
// File class filter
const class_filter = `contains(${yaml_class}, "task")`;

// Current file filter
const current_file_filter = `${file_name} != this.${file_name}`;

// Folder directory filter
const folder_filter = `contains(file.path, this.file.folder)`;

// Task checkbox
const task_checkbox_filter = `regextest("${task_tag_regex}", T.text)`;

// Task due filter
const due_filter = `T.status = " "`;

// Task completed filter
const done_filter = `T.completed`;

// Discarded status filter
const discard_filter = `T.status != "-"`;

//---------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//---------------------------------------------------------
const status_sort = `choice(${yaml_status} = "done", 1,
    choice(${yaml_status} = "in_progress", 2,
    choice(${yaml_status} = "to_do", 3,
    choice(${yaml_status} = "schedule", 4,
    choice(${yaml_status} = "on_hold", 5, 6)))))`;

const type_sort = `choice(contains(${yaml_type}, "project"), 1, 2)`;

//---------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE: "project", "parent_task", "task"
// VAR STATUS: "due", "done", "null"
// VAR RELATION: "linked", "parent", "child_par", "child_task", "sibling"
// EXP: "linked" for linked all tasks external to their project;
// EXP: "parent" for higher task types;
// EXP: "child_par" for parent tasks inside a project;
// EXP: "child_task" for child tasks inside a project or parent task;
// EXP: "child_hab_rit" for child tasks inside a habits and rituals project or parent task;
// EXP: "child_habit" for habits inside a habits and rituals project or parent task;
// EXP: "child_morn_rit" for morning rituals inside a habits and rituals project or parent task;
// EXP: "child_work_start" for workday startup rituals inside a habits and rituals project or parent task;
// EXP: "child_work_stop" for workday shutdown rituals inside a habits and rituals project or parent task;
// EXP: "child_eve_rit" for evening rituals inside a habits and rituals project or parent task;
// EXP: "sibling" for parent or child tasks of the same project or parent task, respectively.

async function dv_task_linked({
  type: type,
  status: status,
  relation: relation,
  md: md,
}) {
  const type_arg = `${type}`;
  const status_arg = `${status}`;
  const relation_arg = `${relation}`;
  const md_arg = `${md}`;

  let dataview_query;

  let data_field;
  if (type_arg.startsWith("proj")) {
    if (relation_arg.startsWith("link")) {
      data_field = `${title_link} AS Project,
    ${file_status},
    ${date_span},
    ${context},
    ${objective},
    ${outcome}`;
    } else if (relation_arg.startsWith("child_par")) {
      data_field = `${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${contact},
    ${objective},
    ${outcome}`;
    } else if (relation_arg.startsWith("child_task")) {
      if (status_arg == "due") {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${task_start} AS Start,
    ${task_end} AS End,
    ${task_estimate},
    ${parent_task}`;
      } else {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${time_span},
    ${task_est_accuracy},
    ${outcome},
    ${parent_task}`;
      }
    } else if (relation_arg.startsWith("child_hab_rit")) {
      if (status_arg == "due") {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${task_start} AS Start,
    ${task_end} AS End,
    ${task_estimate}`;
      } else {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${time_span},
    ${task_est_accuracy}`;
      }
    } else if (
      relation_arg.startsWith("child_habit") ||
      relation_arg.startsWith("child_morn_rit") ||
      relation_arg.startsWith("child_work_start") ||
      relation_arg.startsWith("child_work_stop") ||
      relation_arg.startsWith("child_eve_rit")
    ) {
      if (status_arg == "due") {
        data_field = `${task_link},
    ${task_date},
    ${task_start} AS Start,
    ${task_end} AS End,
    ${task_estimate}`;
      } else {
        data_field = `${task_link},
    ${task_date},
    ${time_span},
    ${task_est_accuracy}`;
      }
    }
  } else if (type_arg.startsWith("par")) {
    if (relation_arg.startsWith("link")) {
      data_field = `${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${project},
    ${objective},
    ${outcome}`;
    } else if (relation_arg.startsWith("par")) {
      data_field = `${title_link} AS Project,
    ${file_status},
    ${date_span},
    ${objective},
    ${outcome}`;
    } else if (relation_arg.startsWith("sib")) {
      data_field = `${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${objective},
    ${outcome}`;
    } else if (relation_arg.startsWith("child_task")) {
      if (status_arg == "due") {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${task_start} AS Start,
    ${task_end} AS End,
    ${task_estimate}`;
      } else {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${time_span},
    ${task_est_accuracy},
    ${outcome}`;
      }
    } else if (relation_arg.startsWith("child_hab_rit")) {
      if (status_arg == "due") {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${task_start} AS Start,
    ${task_end} AS End,
    ${task_estimate}`;
      } else {
        data_field = `${task_link},
    ${task_type},
    ${task_date},
    ${time_span},
    ${task_est_accuracy}`;
      }
    }
  } else if (type_arg.startsWith("task")) {
    if (relation_arg.startsWith("link")) {
      data_field = `${task_link},
    ${task_type},
    ${task_status},
    ${task_date},
    ${project}`;
    } else if (relation_arg.startsWith("par")) {
      data_field = `${title_link} AS Title,
    ${par_file_type},
    ${file_status},
    ${date_span},
    ${objective},
    ${outcome}`;
    } else if (relation_arg.startsWith("sib")) {
      data_field = `${task_link},
    ${task_type},
    ${task_status},
    ${task_date},
    ${outcome}`;
    }
  }

  let filter;
  let type_filter;
  let relation_filter;
  if (relation_arg.startsWith("link")) {
    if (type_arg.startsWith("proj")) {
      type_filter = `contains(${yaml_class}, "project")`;
      relation_filter = `(contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !${folder_filter}`;
    } else if (type_arg.startsWith("par")) {
      type_filter = `contains(${yaml_class}, "parent")`;
      relation_filter = `(contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !${folder_filter}`;
    } else if (type_arg.startsWith("task")) {
      type_filter = `(contains(${yaml_class}, "action")
    OR contains(${yaml_class}, "meeting")
    OR contains(${yaml_class}, "habit_ritual"))`;
      relation_filter = `(contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !${folder_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("par")) {
    if (type_arg.startsWith("par")) {
      type_filter = `contains(${yaml_class}, "project")`;
      relation_filter = `contains(this.${yaml_proj}, file.name)`;
    } else {
      type_filter = `(contains(${yaml_class}, "project")
    OR contains(${yaml_class}, "parent"))`;
      relation_filter = `(contains(this.${yaml_proj}, file.name)
    OR contains(this.${yaml_parent_task}, file.name))`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("sib")) {
    if (type_arg.startsWith("par")) {
      type_filter = `contains(${yaml_class}, "parent")`;
      relation_filter = `contains(${yaml_proj}, this.${yaml_proj})`;
    } else {
      type_filter = `(contains(${yaml_class}, "action")
    OR contains(${yaml_class}, "meeting")
    OR contains(${yaml_class}, "habit_ritual"))`;
      relation_filter = `contains(${yaml_parent_task}, this.${yaml_parent_task})
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child_par")) {
    type_filter = `contains(${yaml_class}, "parent")`;
    relation_filter = `contains(${yaml_proj}, this.${file_name})
    AND ${folder_filter}`;
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child_hab_rit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
      AND ${due_filter}
      AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
      AND !${due_filter}
      AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child_habit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "habit")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND !${due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child_morn_rit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "morning_rit")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND !${due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child_work_start")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "workday_start")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND !${due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child_work_stop")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "workday_stop")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND !${due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child_eve_rit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "evening_rit")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND !${due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child")) {
    type_filter = `(contains(${yaml_class}, "action")
    OR contains(${yaml_class}, "meeting")
    OR contains(${yaml_class}, "habit_ritual"))`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND !${due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  }

  let sort;
  if (
    (type_arg.startsWith("proj") && relation_arg.startsWith("link")) ||
    (type_arg.startsWith("par") && relation_arg.startsWith("par"))
  ) {
    sort = `${yaml_title} ASC`;
  } else if (
    (type_arg.startsWith("proj") && relation_arg.startsWith("child_par")) ||
    (type_arg.startsWith("par") &&
      (relation_arg.startsWith("link") || relation_arg.startsWith("sib")))
  ) {
    sort = `${yaml_date_start},
    ${yaml_title} ASC`;
  } else if (
    (type_arg.startsWith("task") &&
      (relation_arg.startsWith("link") || relation_arg.startsWith("sib"))) ||
    ((type_arg.startsWith("proj") || type_arg.startsWith("par")) &&
      relation_arg.startsWith("child_task"))
  ) {
    sort = `${task_due},
    ${task_start} ASC`;
  } else if (type_arg.startsWith("task") && relation_arg.startsWith("par")) {
    sort = `${type_sort} ASC`;
  }

  if (
    !(
      (type_arg.startsWith("proj") &&
        (relation_arg.startsWith("link") ||
          relation_arg.startsWith("child_par"))) ||
      (type_arg.startsWith("par") &&
        (relation_arg.startsWith("link") || relation_arg.startsWith("sib"))) ||
      relation_arg.startsWith("par")
    )
  ) {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${proj_dir}
FLATTEN
    file.tasks AS T
WHERE
    ${filter}
SORT
    ${sort}`;
  } else {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${proj_dir}
WHERE
    ${filter}
SORT
    ${sort}`;
  }

  if (md_arg == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    let md_query;

    if (
      (type_arg.startsWith("proj") &&
        (relation_arg.startsWith("link") ||
          relation_arg.startsWith("child_par"))) ||
      (type_arg.startsWith("par") &&
        (relation_arg.startsWith("link") || relation_arg.startsWith("sib"))) ||
      relation_arg.startsWith("par")
    ) {
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

module.exports = dv_task_linked;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// LINKED TASK AND GOAL FILES DATAVIEW TABLE
//---------------------------------------------------------
// MD: "true", "false"
// TYPE: "value", "outcome", "project", "parent_task", "task"
// STATUS: "due", "done", "null"
// RELATION: "linked", "parent", "child", "sibling"
// "linked" for linked all tasks external to their project;
// "parent" for higher task types;
// "child" for parent or child tasks inside a project;
// "sibling" for parent or child tasks of the same project or parent task, respectively.
const linked_task_file_table = await tp.user.dv_task_linked({
  type: type,
  status: status,
  relation: relation,
  md: md,
});
```

#### Examples

```javascript
//---------------------------------------------------------
// LINKED TASK AND GOAL FILES DATAVIEW TABLE
//---------------------------------------------------------
// ALL RELATED GOAL FILES TABLES

// ALL LINKED TASK FILES TABLES
const linked_projects = await tp.user.dv_task_linked({
  type: "project",
  status: "",
  relation: "linked",
  md: "false",
});
const linked_parent_tasks = await tp.user.dv_task_linked({
  type: "parent_task",
  status: "",
  relation: "linked",
  md: "false",
})
const linked_child_tasks = await tp.user.dv_task_linked({
  type: "task",
  status: "",
  relation: "linked",
  md: "false",
});

// PROJECT PARENT AND CHILD TASKS
const project_parent_tasks = await tp.user.dv_task_linked({
  type: "project",
  status: "",
  relation: "child_par",
  md: "false",
});
const project_child_task_due = await tp.user.dv_task_linked({
  type: "project",
  status: "due",
  relation: "child_task",
  md: "false",
});
const project_child_task_not_due = await tp.user.dv_task_linked({
  type: "project",
  status: "null",
  relation: "child_task",
  md: "false",
});

// PARENT TASK PROJECTS, CHILD TASKS, AND SIBLINGS
const parent_task_project = await tp.user.dv_task_linked({
  type: "parent_task",
  status: "",
  relation: "parent",
  md: "false",
});
const parent_task_sibling = await tp.user.dv_task_linked({
  type: "parent_task",
  status: "",
  relation: "sibling",
  md: "false",
});
const parent_task_child_task_due = await tp.user.dv_task_linked({
  type: "parent_task",
  status: "due",
  relation: "child_task",
  md: "false",
});
const parent_task_child_task_not_due = await tp.user.dv_task_linked({
  type: "parent_task",
  status: "null",
  relation: "child_task",
  md: "false",
});

// CHILD TASK PROJECT, PARENT TASK, AND SIBLINGS
const child_task_project_parent_task = await tp.user.dv_task_linked({
  type: "task",
  status: "",
  relation: "parent",
  md: "false",
});
const child_task_sibling = await tp.user.dv_task_linked({
  type: "task",
  status: "",
  relation: "sibling",
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

1. [[50_00_project|General Project Template]]
2. [[51_00_parent_task|General Parent Task Template]]
3. [[90_00_note|General Note Template]]
4. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
5. [[90_11_note_quote|Quote Fleeting Note Template]]
6. [[90_12_note_idea|Idea Fleeting Note Template]]
7. [[90_20_note_literature(X)|General Literature Note Template]]
8. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
9. [[90_31_note_question|QEC Question Note Template]]
10. [[90_32_note_evidence|QEC Evidence Note Template]]
11. [[90_33_note_conclusion|QEC Conclusion Note Template]]
12. [[90_40_note_lit_psa(X)|PSA Note Template]]
13. [[90_41_note_problem|PSA Problem Note Template]]
14. [[90_42_note_steps|PSA Steps Note Template]]
15. [[90_43_note_answer|PSA Answer Note Template]]
16. [[90_50_note_info(X)|General Info Note Template]]
17. [[90_51_note_concept|Concept Note Template]]
18. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_task_linked.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_linked_file(X)|Linked File Dataview Table]]
2. [[dv_dir_linked|Linked Directory Files Dataview Table]]
3. [[dv_lib_linked|Linked Library Files Dataview Table]]
4. [[dv_pkm_linked|Linked Personal Knowledge Files Dataview Table]]

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
