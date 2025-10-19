// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> YAML FRONTMATTER FIELDS <<<<<
//-------------------------------------------------------------------
// File name
const file_name = "file.name";

// YAML title
const yaml_title = "file.frontmatter.title";

// YAML alias
const yaml_alias = "file.frontmatter.aliases[0]";

// YAML file class
const yaml_class = "file.frontmatter.file_class";

// YAML type
const yaml_type = "file.frontmatter.type";

// YAML status
const yaml_status = "file.frontmatter.status";

// YAML task start date
const yaml_task_start = "file.frontmatter.task_start";

// YAML task end date
const yaml_task_end = "file.frontmatter.task_end";

// YAML context
const yaml_context = "file.frontmatter.context";

// YAML project
const yaml_proj = "file.frontmatter.project";

// YAML parent task
const yaml_parent_task = "file.frontmatter.parent_task";

// YAML organization
const organization = "file.frontmatter.organization AS Organization";

// YAML contact
const contact = "file.frontmatter.contact AS Contact";

// YAML date created
const yaml_date_created = "file.frontmatter.date_created";

// Tags
const tags = "file.etags AS Tags";

//-------------------------------------------------------------------
// SECT: >>>>> INLINE DATAVIEW FIELDS <<<<<
//-------------------------------------------------------------------
// Objective statement
const objective = "Objective AS Objective";

const outcome_re_test = `regextest("\\w", Outcome)`;
const feeling_re_test = `regextest("\\w", Feeling)`;
const outcome = `("**Outcome**: " + Outcome)`;
const feeling = `("**Feeling**: " + Feeling)`;

// Outcome and feeling
const outcome_feeling = `choice(${outcome_re_test} AND ${feeling_re_test},
    list(${outcome}, ${feeling}),
    choice(${outcome_re_test} AND !${feeling_re_test},
      ${outcome},
      "NULL")
    ) AS Result`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELD FUNCTIONS <<<<<
//-------------------------------------------------------------------
// Task Context
const context = `join(map(split(${yaml_context}, "_"),
      (x) => upper(x[0]) + substring(x, 1)),
      " and ")
    AS Context`;

// Project
const project = `choice(length(${yaml_proj}) < 2, ${yaml_proj}[0], flat(${yaml_proj})) AS Project`;

// Parent Task
const parent_task = `choice(length(${yaml_parent_task}) < 2, ${yaml_parent_task}[0], flat(${yaml_parent_task})) AS "Parent Task"`;

// SECT: >>>>> PROJECT AND PARENT TASK FIELDS <<<<<
// Title link
const title_link = `link(${file_name}, ${yaml_alias})`;

// Title link for DV markdown query
const md_title_link = `"[[" + ${file_name} + "\|" + ${yaml_alias} + "]]"`;

// File status
const file_status = `default(((x) => {
      "done": "✔️Done",
      "in_progress": "👟In progress",
      "to_do": "🔜To do",
      "schedule": "📅Schedule",
      "on_hold": "🤌On hold",
      "applied": "📨Applied💼",
      "offer": "📝Job Offer💼",
      "rejected": "🚫Rejected💼"
    }[x])(${yaml_status}), "❌Discarded")
    AS Status`;

// Date span
const task_start_re_test = `regextest("\\d", ${yaml_task_start})`;
const task_start_remove_alpha = `regexreplace(${yaml_task_start}, "[^\\d-]", "")`;
const task_start_datefmt = `dateformat(date(${task_start_remove_alpha}), "yy-MM-dd")`;
const task_end_re_test = `regextest("\\d", ${yaml_task_end})`;
const task_end_remove_alpha = `regexreplace(${yaml_task_end}, "[^\\d-]", "")`;
const task_end_datefmt = `dateformat(date(${task_end_remove_alpha}), "yy-MM-dd")`;
const date_span = `choice((${task_start_re_test} AND ${task_end_re_test}),
		(${task_start_datefmt} + " → " + ${task_end_datefmt}),
		choice(${task_start_re_test},
			(${task_start_datefmt} + " → Present"),
			"NULL"))
	AS Dates`;

// SECT: >>>>> CHILD TASK FIELDS <<<<<
// Task tag regex
const task_tag_regex = "#task";

// Task name regex
const task_name_regex =
  "#task\\s(.+)_(action_|meeting|phone_call|video_call|interview|lecture|appointment|event|hangout|habit|gathering|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+";

// Inline data field regex
const inline_field_regex = "([.*$)";

// Task page section regex
const task_sect_regex = "(.+>\\s)|(\\]\\])$";

// Task page section
const task_sect = "T.link";

// Task title
const task_title = `regexreplace(T.text, "${task_name_regex}", "$1")`;

// Task link
const task_link = `link(${task_sect}, ${task_title}) AS Task`;

// Task section shortened
const task_sect_short = `regexreplace(string(${task_sect}), "${task_sect_regex}", "")`;

// Title link for DV markdown query
const md_task_link = `"[[" + ${file_name} + "#" + ${task_sect_short} + "\|" + ${task_title} + "]]" AS Task`;

// Task type
const task_type = `choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_video"), "📹Call",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_lecture"), "🧑‍🏫Lecture",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🦿Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))))
    AS Type`;

// Task status
const task_status = `choice((T.status = "-"), "❌Discard",
      choice((T.status = "<"), "⏹️Canceled",
      choice((T.status = "x"), "✔️Done",
        "🔜To do")))
    AS Status`;

// Due or completed date
const task_due = "T.due";
const task_done = "T.completion";
const due_date = `dateformat(${task_due}, "yy-MM-dd")`;
const done_date = `dateformat(${task_done}, "yy-MM-dd")`;
const task_date = `choice(T.status = "x", ${done_date}, ${due_date})
    AS Date`;

// Time span
const task_start = `T.time_start`;
const task_end = `T.time_end`;
const time_span = `(${task_start} + " - " + ${task_end}) AS Time`;

// Time duration estimate
const dur_lt_sixty = "T.duration_est < 60";
const dur_lt_sixty_result = `T.duration_est + "m"`;
const dur_eq_sixty_cond = "T.duration_est = 60";
const dur_eq_sixty_result = `T.duration_est + "h"`;
const dur_mod_sixty = "T.duration_est % 60";
const dur_mod_sixty_cond = "T.duration_est % 60 = 0";
const dur_mod_sixty_result = `(T.duration_est/60) + "h"`;
const task_duration_est = `dur(
      choice(${dur_lt_sixty}, ${dur_lt_sixty_result},
      choice(${dur_mod_sixty_cond},
        ${dur_mod_sixty_result},
        (T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"))
    ) AS Estimate`;
const task_duration_est_fmt = `choice(${dur_lt_sixty}, durationformat(dur(${dur_lt_sixty_result}), "m 'min'"),
    choice(T.duration_est = 60, durationformat(dur(T.duration_est + "h"), "h 'hr'"),
    choice(T.duration_est % 60 = 0, durationformat(dur((T.duration_est/60) + "h"), "h 'hrs'"),
    choice(T.duration_est < 120,
      durationformat(dur((T.duration_est - 60) + "m 1h"), "h 'hr' m 'min'"),
      durationformat(dur((T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"), "h 'hrs' m 'min'")
    ))))`;
const task_estimate = `${task_duration_est_fmt} AS Estimate`;

// Time duration
const task_status_date = `choice(T.status = "x", ${task_done}, ${task_due})`;
const task_status_date_fmt = `dateformat(${task_status_date}, "yyyy-MM-dd")`;
const task_duration_act = `dur(
      date(${task_status_date_fmt} + "T" + ${task_end}) -
      date(${task_status_date_fmt} + "T" + ${task_start})
    ) AS Duration_ACT`;

const task_est_accuracy = `choice(T.status = "-", "❌Discarded",
      choice(T.status = "<", "⏹️Canceled",
      (choice(Estimate = Duration_ACT, "👍On Time",
      choice(Estimate > Duration_ACT,
        "🟢" + (Estimate - Duration_ACT),
        "❗" + (Duration_ACT - Estimate))
      ) + " (" + Estimate_FMT + ")")
    )) AS Accuracy`;

// Project and parent task file type
const par_file_type = `choice(contains(${yaml_type}, "project"), "🏗️Project", "⚒️Parent Task") AS Type`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//-------------------------------------------------------------------
// Projects directory
const proj_personal_dir = `"41_personal"`;
const proj_education_dir = `"42_education"`;
const proj_professional_dir = `"43_professional"`;
const proj_work_dir = `"44_work"`;
const proj_habit_ritual_dir = `"45_habit_ritual"`;

const projects_dir = `${proj_personal_dir}
    OR ${proj_education_dir}
    OR ${proj_professional_dir}
    OR ${proj_work_dir}
    OR ${proj_habit_ritual_dir}`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTER <<<<<
//-------------------------------------------------------------------
// File class filter
const class_filter = `contains(${yaml_class}, "task")`;

// Current file filter
const current_file_filter = `${file_name} != this.${file_name}`;

// Folder directory filter
const folder_filter = `contains(file.path, this.file.folder)`;

// File outlink filter
const outlink_filter = `contains(file.outlinks, this.file.link)`;

// File inlink filter
const inlink_filter = `contains(file.inlinks, this.file.link)`;

// Task checkbox
const task_checkbox_filter = `regextest("${task_tag_regex}", T.text)`;

// Task due filter
const due_filter = `T.status = " "`;

// Task due filter
const not_due_filter = `T.status != " "`;

// Task completed filter
const done_filter = `T.completed`;

// Discarded status filter
const discard_filter = `T.status != "-"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//-------------------------------------------------------------------
const status_sort = `choice(${yaml_status} = "done", 1,
    choice(${yaml_status} = "in_progress", 2,
    choice(${yaml_status} = "to_do", 3,
    choice(${yaml_status} = "schedule", 4,
    choice(${yaml_status} = "on_hold", 5, 6)))))`;

const type_sort = `choice(contains(${yaml_type}, "project"), 1, 2)`;

//-------------------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE: "project", "parent_task", "child_task"
// VAR STATUS: "due", "done", "null"
// VAR RELATION: "linked", "parent", "child_par", "child_task", "sibling"
// EXP: "linked" for all linked tasks external to their project;
// EXP: "in_link" for linked tasks external to their project;
// EXP: "parent" for higher task types;
// EXP: "child_par" for parent tasks inside a project;
// EXP: "child_task" for child tasks inside a project or parent task;
// EXP: "hab_rit" for child tasks inside a habits and rituals project or parent task;
// EXP: "habit" for habits inside a habits and rituals project or parent task;
// EXP: "morning_ritual" for morning rituals inside a habits and rituals project or parent task;
// EXP: "workday_startup_ritual" for workday startup rituals inside a habits and rituals project or parent task;
// EXP: "workday_shutdown_ritual" for workday shutdown rituals inside a habits and rituals project or parent task;
// EXP: "evening_ritual" for evening rituals inside a habits and rituals project or parent task;
// EXP: "sibling" for parent or child tasks of the same project or parent task, respectively.
// EXP: add "in_" to any relation to return inlinks only

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
    if (relation_arg.startsWith("link") || relation_arg.startsWith("in_link")) {
      data_field = `${title_link} AS Project,
    ${file_status},
    ${date_span},
    ${context},
    ${objective},
    ${outcome_feeling}`;
    } else if (
      relation_arg.startsWith("child_par") ||
      relation_arg.startsWith("in_child_par")
    ) {
      data_field = `${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${contact},
    ${objective},
    ${outcome_feeling}`;
    } else if (
      relation_arg.startsWith("child_t") ||
      relation_arg.startsWith("in_child_t")
    ) {
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
    ${outcome_feeling},
    ${parent_task}`;
      }
    } else if (
      relation_arg.startsWith("hab_rit") ||
      relation_arg.startsWith("in_hab_rit")
    ) {
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
    ${parent_task}`;
      }
    } else if (
      relation_arg.startsWith("habit") ||
      relation_arg.startsWith("morning_ritual") ||
      relation_arg.startsWith("workday_startup_ritual") ||
      relation_arg.startsWith("workday_shutdown_ritual") ||
      relation_arg.startsWith("evening_ritual") ||
      relation_arg.startsWith("in_habit") ||
      relation_arg.startsWith("in_morning_ritual") ||
      relation_arg.startsWith("in_workday_startup_ritual") ||
      relation_arg.startsWith("in_workday_shutdown_ritual") ||
      relation_arg.startsWith("in_evening_ritual")
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
    if (relation_arg.startsWith("link") || relation_arg.startsWith("in_link")) {
      data_field = `${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${project},
    ${objective},
    ${outcome_feeling}`;
    } else if (
      relation_arg.startsWith("par") ||
      relation_arg.startsWith("in_par")
    ) {
      data_field = `${title_link} AS Project,
    ${file_status},
    ${date_span},
    ${objective},
    ${outcome_feeling}`;
    } else if (
      relation_arg.startsWith("sib") ||
      relation_arg.startsWith("in_sib")
    ) {
      data_field = `${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${objective},
    ${outcome_feeling}`;
    } else if (
      relation_arg.startsWith("child") ||
      relation_arg.startsWith("in_child")
    ) {
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
    ${outcome_feeling}`;
      }
    } else if (
      relation_arg.startsWith("hab_rit") ||
      relation_arg.startsWith("in_hab_rit")
    ) {
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
  } else if (type_arg.startsWith("child")) {
    if (relation_arg.startsWith("link") || relation_arg.startsWith("in_link")) {
      data_field = `${task_link},
    ${task_type},
    ${task_status},
    ${task_date},
    ${project}`;
    } else if (
      relation_arg.startsWith("par") ||
      relation_arg.startsWith("in_par")
    ) {
      data_field = `${title_link} AS Title,
    ${par_file_type},
    ${file_status},
    ${date_span},
    ${objective},
    ${outcome_feeling}`;
    } else if (
      relation_arg.startsWith("sib") ||
      relation_arg.startsWith("in_sib")
    ) {
      data_field = `${task_link},
    ${task_type},
    ${task_status},
    ${task_date},
    ${outcome_feeling}`;
    }
  }

  let filter;
  let type_filter;
  let relation_filter;
  if (relation_arg.startsWith("link")) {
    if (type_arg.startsWith("proj")) {
      type_filter = `contains(${yaml_class}, "project")`;
      relation_filter = `(${outlink_filter}
    OR ${inlink_filter})
    AND !${folder_filter}`;
    } else if (type_arg.startsWith("par")) {
      type_filter = `contains(${yaml_class}, "parent")`;
      relation_filter = `(${outlink_filter}
    OR ${inlink_filter})
    AND !${folder_filter}`;
    } else if (type_arg.startsWith("child")) {
      type_filter = `contains(${yaml_class}, "child")`;
      relation_filter = `(${outlink_filter}
    OR ${inlink_filter})
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
    } else if (type_arg.startsWith("child")) {
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
      relation_filter = `filter(${yaml_proj}, (project) =>
      contains(this.${yaml_proj}, project))`;
    } else if (type_arg.startsWith("child")) {
      type_filter = `contains(${yaml_class}, "child")`;
      relation_filter = `filter(${yaml_proj}, (project) =>
      contains(this.${yaml_proj}, project))
    AND filter(${yaml_parent_task}, (parent) =>
      contains(this.${yaml_parent_task}, parent))
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
  } else if (relation_arg.startsWith("hab_rit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("habit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "habit")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("morning_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "morning_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("workday_startup_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "workday_startup_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("workday_shutdown_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "workday_shutdown_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("evening_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "evening_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("child")) {
    type_filter = `contains(${yaml_class}, "child")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  }

  if (relation_arg.startsWith("in_link")) {
    relation_filter = `${outlink_filter}
    AND !${inlink_filter}
    AND !${folder_filter}`;
    if (type_arg.startsWith("proj")) {
      type_filter = `contains(${yaml_class}, "project")`;
    } else if (type_arg.startsWith("par")) {
      type_filter = `contains(${yaml_class}, "parent")`;
    } else if (type_arg.startsWith("child")) {
      type_filter = `contains(${yaml_class}, "child")`;
      relation_filter = `${relation_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_par")) {
    if (type_arg.startsWith("par")) {
      type_filter = `contains(${yaml_class}, "project")`;
      relation_filter = `contains(this.${yaml_proj}, file.name)
    AND !${inlink_filter}`;
    } else {
      type_filter = `(contains(${yaml_class}, "project")
    OR contains(${yaml_class}, "parent"))`;
      relation_filter = `(contains(this.${yaml_proj}, file.name)
    OR contains(this.${yaml_parent_task}, file.name))
    AND !${inlink_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_sib")) {
    if (type_arg.startsWith("par")) {
      type_filter = `contains(${yaml_class}, "parent")`;
      relation_filter = `(contains(${yaml_proj}, this.${yaml_proj}[0])
    OR contains(${yaml_proj}, this.${yaml_proj}[1]))
    AND !${inlink_filter}`;
    } else {
      type_filter = `contains(${yaml_class}, "child")`;
      relation_filter = `(contains(${yaml_parent_task}, this.${yaml_parent_task}[0])
    OR contains(${yaml_parent_task}, this.${yaml_parent_task}[1]))
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_child_par")) {
    type_filter = `contains(${yaml_class}, "parent")`;
    relation_filter = `contains(${yaml_proj}, this.${file_name})
    AND ${folder_filter}
    AND !${inlink_filter}`;
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_hab_rit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND !${inlink_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND !${inlink_filter}
    AND ${not_due_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_habit")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "habit")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_morning_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "morning_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_workday_startup_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "workday_startup_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_workday_shutdown_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "workday_shutdown_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_evening_ritual")) {
    type_filter = `contains(${yaml_class}, "habit_ritual")
    AND contains(${yaml_type}, "evening_ritual")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (relation_arg.startsWith("in_child")) {
    type_filter = `contains(${yaml_class}, "child")`;
    if (status_arg == "due") {
      relation_filter = `${folder_filter}
    AND ${due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    } else {
      relation_filter = `${folder_filter}
    AND ${not_due_filter}
    AND !${inlink_filter}
    AND ${task_checkbox_filter}`;
    }
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  }

  let sort;
  if (
    (type_arg.startsWith("proj") &&
      (relation_arg.startsWith("link") ||
        relation_arg.startsWith("in_link"))) ||
    (type_arg.startsWith("par") &&
      (relation_arg.startsWith("par") || relation_arg.startsWith("in_par")))
  ) {
    sort = `${yaml_title} ASC`;
  } else if (
    (type_arg.startsWith("proj") &&
      (relation_arg.startsWith("child_par") ||
        relation_arg.startsWith("in_child_par"))) ||
    (type_arg.startsWith("par") &&
      (relation_arg.startsWith("link") ||
        relation_arg.startsWith("in_link") ||
        relation_arg.startsWith("sib") ||
        relation_arg.startsWith("in_sib")))
  ) {
    sort = `${yaml_task_start},
    ${yaml_title} ASC`;
  } else if (
    (type_arg.startsWith("child") &&
      (relation_arg.startsWith("link") ||
        relation_arg.startsWith("in_link") ||
        relation_arg.startsWith("sib") ||
        relation_arg.startsWith("in_sib"))) ||
    ((type_arg.startsWith("proj") || type_arg.startsWith("par")) &&
      (relation_arg.startsWith("child_task") ||
        relation_arg.startsWith("in_child_task") ||
        relation_arg.startsWith("hab_rit") ||
        relation_arg.startsWith("in_hab_rit") ||
        relation_arg.startsWith("habit") ||
        relation_arg.startsWith("in_habit") ||
        relation_arg.startsWith("morning_ritual") ||
        relation_arg.startsWith("in_morning_ritual") ||
        relation_arg.startsWith("workday_startup_ritual") ||
        relation_arg.startsWith("in_workday_startup_ritual") ||
        relation_arg.startsWith("workday_shutdown_ritual") ||
        relation_arg.startsWith("in_workday_shutdown_ritual") ||
        relation_arg.startsWith("evening_ritual") ||
        relation_arg.startsWith("in_evening_ritual")))
  ) {
    sort = `${task_due},
    ${task_start} ASC`;
  } else if (
    type_arg.startsWith("child") &&
    (relation_arg.startsWith("par") || relation_arg.startsWith("in_par"))
  ) {
    sort = `${type_sort},
    ${yaml_title} ASC`;
  }

  if (
    !(
      (type_arg.startsWith("proj") &&
        (relation_arg.startsWith("link") ||
          relation_arg.startsWith("in_link") ||
          relation_arg.startsWith("child_par") ||
          relation_arg.startsWith("in_child_par"))) ||
      (type_arg.startsWith("par") &&
        (relation_arg.startsWith("link") ||
          relation_arg.startsWith("in_link") ||
          relation_arg.startsWith("sib") ||
          relation_arg.startsWith("in_sib"))) ||
      relation_arg.startsWith("par")
    )
  ) {
    if (status_arg == "due") {
      dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${projects_dir}
FLATTEN
    file.tasks AS T
WHERE
    ${filter}
SORT
    ${sort}
${three_backtick}`;
    } else {
      dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${projects_dir}
FLATTEN
    file.tasks AS T
FLATTEN
    ${task_duration_est}
FLATTEN
    ${task_duration_est_fmt} AS Estimate_FMT
FLATTEN
    ${task_duration_act}
WHERE
    ${filter}
SORT
    ${sort}
${three_backtick}`;
    }
  } else {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${projects_dir}
WHERE
    ${filter}
SORT
    ${sort}
${three_backtick}`;
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
