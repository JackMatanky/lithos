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

// YAML calendar start date
const yaml_date_start = "file.frontmatter.date_start";

// YAML calendar end date
const yaml_date_end = "file.frontmatter.date_end";

// YAML task date
const yaml_task_date = "file.frontmatter.date";

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

// Outcome and feeling
const outcome = `choice(regextest("\\w", Outcome) AND regextest("\\w", Feeling), list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)),
    choice(regextest("\\w", Outcome) AND !regextest("\\w", Feeling), ("**Outcome**: " + Outcome), "NULL")
    ) AS Result`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELD FUNCTIONS <<<<<
//-------------------------------------------------------------------
// Task Context
const context = `choice(${yaml_context} = "habit_ritual", "Habits and Rituals",
    upper(substring(${yaml_context}, 0, 1)) + substring(${yaml_context}, 1))
    AS Context`;

// Task Hierarchy
const task_hierarcy = `choice(contains(${yaml_parent_task}[0], "null"), 
      ("**Project**: " + flat(${yaml_proj})),
      list(("**Project**: " + flat(${yaml_proj})), ("**Parent**: " + flat(${yaml_parent_task})))
    ) AS "Task Hierarchy"`;

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
const date_span = `choice((regextest("\\d", ${yaml_task_start}) AND regextest("\\d", ${yaml_task_end})), 
		(dateformat(date(regexreplace(${yaml_task_start}, "[^\\d-]", "")), "yy-MM-dd") + " → " + dateformat(date(regexreplace(${yaml_task_end}, "[^\\d-]", "")), "yy-MM-dd")),
		choice(regextest("\\d", ${yaml_task_start}),
			(dateformat(date(regexreplace(${yaml_task_start}, "[^\\d-]", "")), "yy-MM-dd") + " → Present"),
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

// Task dates
const task_file_date = `regexreplace(this.${yaml_task_date}, "[^\\d-]", "")`;
const task_creation = "T.created";
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
const task_duration_est = `dur(
      choice(T.duration_est < 60, T.duration_est + "m",
      choice(T.duration_est % 60 = 0,
        (T.duration_est/60) + "h",
        (T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"))
    ) AS Estimate`;
const task_duration_est_fmt = `choice(T.duration_est < 60, durationformat(dur(T.duration_est + "m"), "m 'min'"),
    choice(T.duration_est = 60, durationformat(dur(T.duration_est + "h"), "h 'hr'"),
    choice(T.duration_est % 60 = 0, durationformat(dur((T.duration_est/60) + "h"), "h 'hrs'"),
    choice(T.duration_est < 120,
      durationformat(dur((T.duration_est - 60) + "m 1h"), "h 'hr' m 'min'"),
      durationformat(dur((T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"), "h 'hrs' m 'min'")
    ))))`;
const task_estimate = `${task_duration_est_fmt} AS Estimate`;

// Time duration
const task_duration_act = `dur(
      date(dateformat(choice(T.status = "x", ${task_done}, ${task_due}), "yyyy-MM-dd") + "T" + ${task_end}) -
      date(dateformat(choice(T.status = "x", ${task_done}, ${task_due}), "yyyy-MM-dd") + "T" + ${task_start})
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
const par_file_type = `choice(contains(${yaml_type}, "project"),
      "🏗️Project",
      "⚒️Parent Task"
    ) AS Type`;

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
// SECT: >>>>> DATA FILTERS <<<<<
//-------------------------------------------------------------------
// SECT: >>>>> GENERAL FIELDS <<<<<
// Task file class filter
const class_filter = `contains(${yaml_class}, "task")`;

// SECT: >>>>> PROJECT AND PARENT TASK FILTERS <<<<<
// Schedule or undetermined status filter
const proj_undetermined_filter = `(contains(${yaml_status}, "undetermined")
	OR contains(${yaml_status}, "schedule"))`;

// SECT: >>>>> CHILD TASK FILTERS <<<<<
// Task checkbox
const task_checkbox_filter = `regextest("${task_tag_regex}", T.text)`;

// Task due filter
const task_due_filter = `T.status = " "`;

// Task not due filter
const task_not_due_filter = `T.status != " "`;

// Task completed filter
const task_done_filter = `T.completed`;

// Canceled status filter
const task_cancel_filter = `T.status != "<"`;

// Discarded status filter
const task_discard_filter = `T.status != "-"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//-------------------------------------------------------------------
const status_sort = `default(((x) => {
      "done": 1,
      "in_progress": 2,
      "to_do": 3,
      "schedule": 4,
      "on_hold": 5
    }[x])(${yaml_status}), 6)`;

const task_status_sort = `default(((x) => {
      "x": 1,
      " ": 2,
      "-": 3
    }[x])(T.status), 4)`;

//-------------------------------------------------------------------
// DATAVIEW TABLE FOR TASKS AND EVENTS
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR: TYPES: "project", "parent_task", "child_task", "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// VAR STATUSES: "not_due", "done", "active", "to_do", "in_progress", "schedule", "on_hold", "new", "review", "preview", "eve_preview", "overdue"
// VAR DATES: "month", "week", "day"

async function dv_task_type_status_dates({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  // TABLE DATA FIELDS
  let data_field;
  if (type.startsWith("pro")) {
    if (status.startsWith("don") || status.startsWith("rev")) {
      // Data fields for DONE, or REVIEW statuses
      data_field = `${title_link} AS Project,
    ${date_span},
    ${context},
    ${objective},
    ${outcome}`;
    } else if (
      status.startsWith("act") ||
      status.startsWith("due") ||
      status.startsWith("new") ||
      status.startsWith("over")
    ) {
      data_field = `${title_link} AS Project,
    ${file_status},
    ${date_span},
    ${context},
    ${objective}`;
    }
  } else if (type.startsWith("par")) {
    if (status.startsWith("don") || status.startsWith("rev")) {
      // Data fields for DONE, or REVIEW statuses
      data_field = `${title_link} AS "Parent Task",
    ${date_span},
    ${objective},
    ${outcome},
    ${project}`;
    } else if (
      status.startsWith("act") ||
      status.startsWith("due") ||
      status.startsWith("new") ||
      status.startsWith("over")
    ) {
      data_field = `${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${objective},
    ${project}`;
    }
  } else if (
    type.startsWith("task") ||
    type.startsWith("child") ||
    type == "child"
  ) {
    // if (md != "true") {
    //   task_hierarcy = project;
    // } else if (md == "true") {
    //   task_hierarcy = `${parent_task},
    // ${project}`;
    // }
    if (status.startsWith("don") || status.startsWith("rev")) {
      // Data fields for DONE, or REVIEW statuses
      data_field = `${task_link},
    ${task_type},
    ${time_span},
    ${task_est_accuracy},
    ${outcome},
    ${parent_task},
    ${project}`;
    } else if (
      status.startsWith("pla") ||
      status.startsWith("due") ||
      status.startsWith("act") ||
      status.startsWith("new")
    ) {
      data_field = `${task_link},
    ${task_type},
    ${task_start} AS Start,
    ${task_end} AS End,
    ${parent_task},
    ${project}`;
    } else if (status.startsWith("pre") || status.startsWith("eve")) {
      data_field = `${task_link},
    ${task_type},
    ${due_date} AS Date,
    ${task_start} AS Start,
    ${task_end} AS End,
    ${parent_task},
    ${project}`;
    }
  } else if (
    type.startsWith("habit") ||
    type.startsWith("morn") ||
    type.startsWith("work") ||
    type.startsWith("eve")
  ) {
    if (status.startsWith("don") || status.startsWith("rev")) {
      // Data fields for DONE, or REVIEW statuses
      data_field = `${task_link},
    ${task_status},
    ${task_date},
    ${time_span},
    ${task_est_accuracy},
    ${parent_task}`;
    } else if (status.startsWith("act") || status.startsWith("due")) {
      data_field = `${task_link},
    ${due_date} AS Date,
    ${task_start} AS Start,
    ${task_end} AS End,
    ${task_estimate},
    ${parent_task}`;
    }
  }

  // FILTER
  let filter = `contains(${yaml_class}, "${type}")`;
  let date_filter = "null";
  // FILTERs for PROJECTS and PARENT TASKS
  if (type.startsWith("pro") || type.startsWith("par")) {
    if (status.startsWith("don")) {
      // FILTER for COMPLETED projects and parent tasks
      date_filter = `date(regexreplace(${yaml_task_end}, "[^\\d-]", "")) >= date(${date_start})
    AND date(regexreplace(${yaml_task_end}, "[^\\d-]", "")) <= date(${date_end})`;
      if (date_end == "") {
        date_filter = `date(regexreplace(${yaml_task_end}, "[^\\d-]", "")) = date(${date_start})`;
      }
      filter = `${filter}
    AND contains(${yaml_status}, "done")
    AND ${date_filter}`;
    } else if (status.startsWith("over")) {
      // FILTER for COMPLETED and DISCARDED projects and parent tasks
      date_filter = `date(regexreplace(${yaml_task_end}, "[^\\d-]", "")) <= date(${date_start})
    AND regextest("\\d", ${yaml_task_end})`;
      filter = `${filter}
    AND !(contains(${yaml_status}, "done")
    OR contains(${yaml_status}, "discarded")
    OR contains(${yaml_status}, "rejected"))
    AND ${date_filter}`;
    } else if (status.startsWith("new")) {
      // FILTER for NEW projects and parent tasks
      date_filter = `date(${date_created}) >= date(${date_start})
    AND date(${date_created}) <= date(${date_end})`;
      if (date_end == "") {
        date_filter = `date(${date_created}) = date(${date_start})`;
      }
      filter = `${filter}
    AND ${date_filter}`;
    } else if (status.startsWith("act")) {
      // FILTER for ACTIVE projects and parent tasks
      filter = `${filter}
    AND (contains(${yaml_status}, "to_do")
    OR contains(${yaml_status}, "in_progress"))`;
    } else if (status.startsWith("sche") || status.startsWith("on")) {
      // FILTER for projects and parent tasks to SCHEDULE or ON HOLD
      filter = `${filter}
    AND (contains(${yaml_status}, "schedule")
    OR contains(${yaml_status}, "on_hold"))`;
    } else if (status.startsWith("und") || status.startsWith("det")) {
      // FILTER for UNDETERMINED projects and parent tasks
      filter = `${filter}
    AND contains(${yaml_status}, "undetermined")`;
    } else {
      filter = `${filter}
    AND contains(${yaml_status}, "${status}")`;
    }
  }
  // FILTERs for CHILD TASKS
  if (type.startsWith("child")) {
    if (status.startsWith("don") || status.startsWith("rev")) {
      filter = `${task_checkbox_filter}
    AND date(${task_due}) = date(${date_start})`;
      if (date_end == "week") {
        filter = `${filter}
    AND ${task_discard_filter}
    AND T.completed`;
      } else {
        filter = `${filter}
    AND ${task_not_due_filter}`;
      }
    } else if (status.startsWith("pla")) {
      filter = `${task_checkbox_filter}
    AND date(${task_due}) = date(${date_start})`;
      if (date_end != "week") {
        filter = `${filter}
    AND !(T.status = "-"
      OR T.status = "<")`;
      }
    } else if (status.startsWith("due")) {
      filter = `${task_checkbox_filter}
    AND date(${task_due}) = date(${date_start})`;
      if (date_end != "week") {
        filter = `${filter}
    AND ${task_due_filter}`;
      }
    } else if (status.startsWith("pre")) {
      filter = `${task_checkbox_filter}
    AND ${task_discard_filter}
    AND ${task_cancel_filter}
    AND (date(${task_due}) = date(${date_start})
    OR (${task_due_filter}
    AND date(${task_due}) < date(${date_start})))`;
    } else if (status.startsWith("eve")) {
      filter = `${task_checkbox_filter}
    AND ${task_discard_filter}
    AND (date(${task_due}) = date(${date_start})
    OR (${task_due_filter}
    AND date(${task_due}) < date(${date_start})
    AND !contains(${yaml_type}, "eve")))`;
    } else if (status.startsWith("new")) {
      filter = `${task_checkbox_filter}
    AND ${task_discard_filter}
    AND date(${task_creation}) = date(${date_start})`;
    }
  }
  if (
    type.startsWith("habit") ||
    type.startsWith("morn") ||
    type.startsWith("work") ||
    type.startsWith("eve")
  ) {
    if (status.startsWith("don") || status.startsWith("rev")) {
      filter = `contains(${yaml_type}, "${type}")
    AND ${task_checkbox_filter}
    AND ${task_not_due_filter}
    AND date(${task_due}) >= date(${date_start})
    AND date(${task_due}) <= date(${date_end})`;
    } else if (status.startsWith("due") && date_end != "") {
      filter = `contains(${yaml_type}, "${type}")
    AND ${task_checkbox_filter}
    AND date(${task_due}) >= date(${date_start})
    AND date(${task_due}) <= date(${date_end})`;
    }
  }

  // SORT fields by status
  let sort_field;
  if (
    type.startsWith("child") ||
    type.startsWith("habit") ||
    type.startsWith("morn") ||
    type.startsWith("work") ||
    type.startsWith("eve")
  ) {
    if (status.startsWith("don") || status.startsWith("rev")) {
      sort_field = `${task_due},
    ${task_status_sort},
    ${task_start}`;
    } else {
      sort_field = `${task_due},
    ${task_start}`;
    }
  } else if (status.startsWith("don")) {
    if (type.startsWith("pro")) {
      // PROJECT
      sort_field = `${yaml_task_end},
    ${yaml_title}`;
    } else if (type.startsWith("par")) {
      sort_field = `${yaml_proj},
    ${yaml_task_end},
    ${yaml_title}`;
    }
  } else if (
    status.startsWith("act") ||
    status.startsWith("sche") ||
    status.startsWith("on") ||
    status.startsWith("und") ||
    status.startsWith("det") ||
    status.startsWith("new")
  ) {
    if (type.startsWith("pro")) {
      // PROJECT
      sort_field = `${status_sort},
    ${yaml_task_start},
    ${yaml_title}`;
    } else if (type.startsWith("par")) {
      sort_field = `${status_sort},
    ${yaml_proj},
    ${yaml_task_start},
    ${yaml_title}`;
    }
  }

  let dataview_query;
  // >>>>> TASK TABLES <<<<<
  if (type.startsWith("pro") || type.startsWith("par")) {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${projects_dir}
WHERE
    ${class_filter}
    AND ${filter}
SORT
    ${sort_field} ASC
${three_backtick}`;
  } else {
    if (status.startsWith("don") || status.startsWith("rev")) {
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
    ${projects_dir}
FLATTEN
    file.tasks AS T
WHERE
    ${class_filter}
    AND ${filter}
SORT
    ${sort_field} ASC
${three_backtick}`;
    }
  }

  if (md == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    let md_query;

    if (type.startsWith("pro") || type.startsWith("par")) {
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
    // dataview_query = md_query;
  }

  return dataview_query;
}

module.exports = dv_task_type_status_dates;
