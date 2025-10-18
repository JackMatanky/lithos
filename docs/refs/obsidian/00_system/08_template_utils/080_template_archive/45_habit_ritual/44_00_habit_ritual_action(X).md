<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const projects_dir = "40_projects/";
const habit_ritual_proj_dir = "45_habit_ritual/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";

/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const hr_line = hyphen.repeat(3);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const colon = String.fromCodePoint(0x3a);
const two_percent = String.fromCodePoint(0x25).repeat(2);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

//Text Formatting
const head_lvl = (int) => `${hash.repeat(int)}${space}`;
const yaml_li = (value) => `${new_line}${ul_yaml}"${value}"`;
const link_alias = (file, alias) => ["[[" + file, alias + "]]"].join("|");
const link_tbl_alias = (file, alias) => ["[[" + file, alias + "]]"].join("\\|");
const cmnt_ob_start = `${two_percent}${space}`;
const cmnt_ob_end = `${space}${two_percent}`;
const cmnt_html_start = `${less_than}${excl}${two_hyphen}${space}`;
const cmnt_html_end = `${space}${two_hyphen}${great_than}`;
const tbl_start = `${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end = `${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${hyphen}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${great_than}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_start}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;
const dv_colon = `${colon.repeat(2)}${space}`;

//-------------------------------------------------------------------
// FORMATTING FUNCTIONS
//-------------------------------------------------------------------
const snake_case_fmt = (name) =>
  name.replaceAll(/(\-\s\-)|(\s)|(\-)]/g, "_").toLowerCase();

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// SET THE FILE'S TITLE
//-------------------------------------------------------------------
// Check if note already has title
const has_title = !tp.file.title.startsWith("Untitled");
let title;
let alias;

// If note does not have title,
// prompt for title and rename file
if (!has_title) {
  title = await tp.system.prompt("Title", "", true, false);
  title = title.trim();
  alias = title.toLowerCase();
  await tp.file.rename(title);
} else {
  title = tp.file.title;
  title = title.trim();
  alias = title.toLowerCase();
}

//-------------------------------------------------------------------
// CHOOSE THE TASK'S DATE AND TIME
//-------------------------------------------------------------------
// Choose the date for the action item
const nl_date = await tp.user.nl_date(tp);

// Choose the time for the action item
const nl_time = await tp.user.nl_time(tp, "");

// Parse full date with Natural Language Dates
const full_date_time = moment(`${nl_date}T${nl_time}`);

//-------------------------------------------------------------------
// SET THE TASK'S START AND REMINDER TIME
//-------------------------------------------------------------------
const date = moment(full_date_time).format("YYYY-MM-DD");

const start_time = moment(full_date_time).format("HH:mm");

const reminder_date = moment(full_date_time)
  .subtract(10, "minutes")
  .format("YYYY-MM-DD HH:mm");

//-------------------------------------------------------------------
// SET TASK DURATION AND END TIME
//-------------------------------------------------------------------
const duration_min = await tp.user.durationMin(tp);

const full_end_date = moment(full_date_time).add(
  Number(duration_min),
  "minutes"
);

const end_time = moment(full_end_date).format("HH:mm");

const duration_est = moment
  .duration(full_end_date.diff(full_date_time))
  .as("minutes");

//-------------------------------------------------------------------
// SET TASK CONTEXT, TASK TYPE, FILE CLASS, AND TASK TITLE
//-------------------------------------------------------------------
const type_obj_arr = [
  { name: "Habit", value: "habit", file_class: "task_habit" },
  {
    name: "Morning Ritual",
    value: "morning_ritual",
    file_class: "task_ritual_morning",
  },
  {
    name: "Workday Startup Ritual",
    value: "workday_startup_ritual",
    file_class: "task_ritual_work_startup",
  },
  {
    name: "Workday Shutdown Ritual",
    value: "workday_shutdown_ritual",
    file_class: "task_ritual_work_shutdown",
  },
  {
    name: "Evening Ritual",
    value: "evening_ritual",
    file_class: "task_ritual_evening",
  },
];
const type_obj = await tp.system.suggester(
  (item) => item.name,
  type_obj_arr,
  false,
  "Type?"
);

const context = "habit_ritual";
const type = type_obj.value;
const file_class = "task_child";
const task_title = title.trim() + "_" + type;

//-------------------------------------------------------------------
// SET PILLAR
//-------------------------------------------------------------------
// Retrieve all files in the Pillars directory
const pillars = await tp.user.file_by_status({
  dir: pillars_dir,
  status: "active",
});

const pillar = await tp.system.suggester(
  pillars,
  pillars,
  false,
  "Pillar?"
);

const pillar_arr = pillar.split("_");
let pillar_name = "";

for (var i = 0; i < pillar_arr.length; i++) {
  pillar_name += `${pillar_arr[i].charAt(0).toUpperCase()}${pillar_arr[i].substring(1)} `;
};

pillar_name.trim();

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  "Goal?"
);

//-------------------------------------------------------------------
// CHECK FILE LOCATION AND ASSIGN RELEVANT VARIABLES
//-------------------------------------------------------------------
// Get the current folder path
const folder_path = tp.file.folder(true);
const folder_path_split = folder_path.split("/");
const folder_path_length = folder_path_split.length;

//-------------------------------------------------------------------
// SET TASK CONTEXT
//-------------------------------------------------------------------
let context;
let context_dir;

if (projects_dir == folder_path_split[0] + "/" && folder_path_length >= 2) {
  context_dir = projects_dir + folder_path_split[1] + "/";
  context = folder_path_split[1].slice(3);
} else {
  // Choose an object from the array of task context objects
  const context_obj = await tp.user.task_context(tp)

  // Return object's name and directory values
  context = context_obj.value;
  context_dir = context_obj.directory;
}

//-------------------------------------------------------------------
// SET PROJECT
//-------------------------------------------------------------------
let project;
let project_dir;

if (projects_dir == folder_path_split[0] + "/" && folder_path_length >= 3) {
  project = folder_path_split[2];
  project_dir = context_dir + project + "/";
} else {
  // Filter array to only include project folder paths based on task context
  const projects = await tp.user.folder_name({
    dir: context_dir,
    index: 2,
  });



  // Choose a project
  project = await tp.system.suggester(
	projects,
	projects,
	false,
	"Project?"
);



  // Assign the project directory
  project_dir = `${context_dir}${project}/`;
};

//-------------------------------------------------------------------
// SET PARENT TASK
//-------------------------------------------------------------------
let parent_task;

if (projects_dir == folder_path_split[0] + "/" && folder_path_length >= 4) {
  parent_task = folder_path_split[3];
} else {
  // Filter array to only include parent task folder paths matching the chosen project
  const parent_tasks = await tp.user.folder_name({
    dir: project,
    index: 3,
  });



  // Choose a parent task
  parent_task = await tp.system.suggester(
    parent_tasks,
    parent_tasks,
    false,
    "Parent Task?"
);
};

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
template_file = "organization_file_name_alias";
temp_file_path = `${sys_temp_include_dir}${template_file}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const organization = include_arr[0];
const organization_name = include_arr[1];

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
template_file = "contact_file_name_alias";
temp_file_path = `${sys_temp_include_dir}${template_file}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const contact = include_arr[0];
const contact_name = include_arr[1];

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const due_do = "do";

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const status_obj_arr = [
  { key: "To do", value: "to_do", symbol: " " },
  { key: "In Progress", value: "in_progress", symbol: "/" },
  { key: "Done", value: "done", symbol: "x" },
  { key: "Schedule", value: "schedule", symbol: "?" },
];
const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  "Status?"
);
const status_value = status_obj.value;
const status_symbol = status_obj.symbol;

//-------------------------------------------------------------------
// SET TASK CREATION DATE AND CHECKBOX TEXT
//-------------------------------------------------------------------
const date_task_creation = moment().format("YYYY-MM-DD");

let task_checkbox;

if (status_value == "done") {
	task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${date_task_creation} 📅 ${date} ✅ ${date}`
} else {
	task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${date_task_creation} 📅 ${date}`
};

//-------------------------------------------------------------------
// RENAME FILE
//-------------------------------------------------------------------
const full_title = `${date} ${title}`;
await tp.file.rename(file_name);

//-------------------------------------------------------------------
// MOVE FILE TO PROJECT'S DIRECTORY
//-------------------------------------------------------------------
let parent_task_dir;

if (parent_task == "null") {
  if ((folder_path + "/")!= project_dir) {
    await tp.file.move(project_dir + file_name);
  }
} else {
  parent_task_dir = project_dir + parent_task;
  if (folder_path!= parent_task_dir) {
    await tp.file.move(parent_task_dir + "/" + file_name);
  }
};

tR += "---"
%>
title: "<%* tR += title %>"
aliases:
  - "<%* tR += title %>"
  - "<%* tR += file_name %>"
  - "<%* tR += alias %>"
date: <%* tR += date_value_link %>
due_do: <%* tR += due_do %>
pillar: <%* tR += pillar %>
context: <%* tR += context %>
goal: <%* tR += goal %>
project: <%* tR += project %>
parent_task: <%* tR += parent_task %>
organization: <%* tR += organization %>
contact: <%* tR += contact %>
status: <%* tR += status %>
type: <%* tR += type %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title %>

## Tasks

<%* tR += task_checkbox %>

### Related Tasks

## Notes

---

## Resources
