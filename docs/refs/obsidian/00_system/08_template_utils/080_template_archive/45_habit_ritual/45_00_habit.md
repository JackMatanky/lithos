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
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const pillar_name_alias = "20_00_pillar_name_alias";
const do_due_date = "40_task_do_due_date";
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Habits and Rituals";
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();
const context_dir = habit_ritual_proj_dir;

//-------------------------------------------------------------------
// HABIT TASK TAG, TYPE, AND FILE CLASS
//-------------------------------------------------------------------
const task_tag = "#task";
const type_name = context_name.split(" ")[0].replaceAll(/s$/g, "");
const type_value = type_name.toLowerCase();
const file_class = "task_child";

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt("Title", null, true, false);
} else {
  title = tp.file.title;
}
title = title.trim();
title = await tp.user.title_case(title);

/* ---------------------------------------------------------- */
/*                      SET DATE AND TIME                     */
/* ---------------------------------------------------------- */
const date = await tp.user.nl_date(tp, "start");
const time = await tp.user.nl_time(tp, "");
const full_date_time = moment(`${date}T${time}`);

//-------------------------------------------------------------------
// SET TASK START AND REMINDER TIME
//-------------------------------------------------------------------
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

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar = include_arr[0];
const pillar_name = include_arr[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, "Goal?");

/* ------------------- FILE PATH VARIABLES ------------------ */
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split("/");
const folder_path_length = folder_path_split.length;

//-------------------------------------------------------------------
// SET PROJECT BY FILE PATH OR SUGGESTER
//-------------------------------------------------------------------
let project;
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 3) {
  project = folder_path_split[2];
} else {
  const projects = await tp.user.folder_name({
    dir: context_dir,
    index: 2,
  });
  project = await tp.system.suggester(projects, projects, false, "Project?");
}
const project_dir = `${context_dir}${project}/`;

/* ---------------------------------------------------------- */
/*          SET PARENT TASK BY FILE PATH OR SUGGESTER         */
/* ---------------------------------------------------------- */
let parent_task;
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 4) {
  parent_task = folder_path_split[3];
} else {
  const parent_tasks = await tp.user.folder_name({
    dir: project,
    index: 3,
  });
  parent_task = await tp.system.suggester(
    parent_tasks,
    parent_tasks,
    false,
    "Parent Task?"
  );
}

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
temp_file_path = `${sys_temp_include_dir}${org_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const organization_value = include_arr[0];
const organization_name = include_arr[1].replace(/\n/, "");
const organization_link = `[[${organization_value}|${organization_name}]]`;
const organization_value_link = yaml_li(organization_link);

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
temp_file_path = `${sys_temp_include_dir}${contact_name_alias}.md`;
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
// TASK CHECKBOX TEXT
//-------------------------------------------------------------------
let task_checkbox;
if (status_value == "done") {
  task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type_value} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${moment().format(
    "YYYY-MM-DD"
)} 📅 ${date} ✅ ${date}`;
} else {
  task_checkbox = `- [${status_symbol}] ${task_tag} ${title}_${type_value} [time_start:: ${start_time}]  [time_end:: ${end_time}]  [duration_est:: ${duration_est}] ⏰ ${reminder_date} ➕ ${moment().format(
    "YYYY-MM-DD"
)} 📅 ${date}`;
}

/* ---------------------------------------------------------- */
/*         FRONTMATTER TITLES, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
const full_title_name = `${date} ${title}`;
const short_title_name = `${title.toLowerCase()}`;
const full_title_value = `${date}_${title
  .replaceAll(/\s/g, "_")
  .toLowerCase()}`;
const short_title_value = `${title.replaceAll(/\s/g, "_").toLowerCase()}`;

const alias_arr = yaml_li(title}"${ul_yaml}"${full_title_name}"${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}"${short_title_value}"${new_line}${ul_yaml}"${full_title_value);

const file_name = `${date} ${short_title_name}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
let directory;
if (parent_task == "null") {
  directory = `${project_dir}`;
} else {
  directory = `${project_dir}${parent_task}/`;
}

if (folder_path!= directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += "---"
%>
title: <%* tR += file_name %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
due_do: <%* tR += due_do %>
pillar: <%* tR += pillar %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
project: <%* tR += project %>
parent_task: <%* tR += parent_task %>
organization: <%* tR += organization %>
contact: <%* tR += contact %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
---

# <%* tR += title %>

> [!<%* tR += type_value %> ] <%* tR += type_name %> Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) => !contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

## <%* tR += type_name %>

<%* tR += task_checkbox %>

---

## Related

### Tasks and Events

### Notes

---

## Resources
