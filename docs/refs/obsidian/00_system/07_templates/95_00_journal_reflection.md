<%*
//-------------------------------------------------------------------
// GLOBAL FOLDER PATH VARIABLES
//-------------------------------------------------------------------
const pillars_dir = `20_pillars/`;
const insights_dir = `80_insight/`;
const reflection_journals_dir = `80_insight/95_reflection/`;
const daily_reflection_dir = `80_insight/95_reflection/01_daily/`;
const weekly_reflection_dir = `80_insight/95_reflection/02_weekly/`;
const monthly_reflection_dir = `80_insight/95_reflection/03_monthly/`;
const quarterly_reflection_dir = `80_insight/95_reflection/04_quarterly/`;
const yearly_reflection_dir = `80_insight/95_reflection/05_yearly/`;
const goals_dir = `30_goals/`;
const projects_dir = `40_projects/`;

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
const pillar_name_alias_preset_mental = "20_03_pillar_name_alias_preset_mental";
const related_project = "40_related_project";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format(`YYYY-MM-DD[T]HH:mm`);
const date_modified = moment().format(`YYYY-MM-DD[T]HH:mm`);

//-------------------------------------------------------------------
// SET JOURNAL TYPE AND FILE CLASS
//-------------------------------------------------------------------
const type_obj_arr = [
  {
    name: "Daily Reflection",
    value: "daily_reflection",
    directory: daily_reflection_dir,
  },
  {
    name: "Weekly Reflection",
    value: "weekly_reflection",
    directory: weekly_reflection_dir,
  },
  {
    name: "Monthly Reflection",
    value: "monthly_reflection",
    directory: monthly_reflection_dir,
  },
  {
    name: "Quarterly Reflection",
    value: "quarterly_reflection",
    directory: quarterly_reflection_dir,
  },
  {
    name: "Yearly Reflection",
    value: "yearly_reflection",
    directory: yearly_reflection_dir,
  },
];

const type_name = type_obj.name;
const type = type_obj.value;
const type_dir = type_obj.directory;
const file_class = `pdev_journal`;

//-------------------------------------------------------------------
// CHOOSE THE JOURNAL WRITING DATE
//-------------------------------------------------------------------
// Choose the date for the journal entry
const date = await tp.user.date(tp);
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const full_date_time = moment(`${date}T00:00`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
// Check if note already has title
const has_title = !tp.file.title.startsWith("Untitled");
let title;
let alias;

// If note does not have title,
// prompt for title and rename file
if (!has_title) {
  title = date + " " + type_name;
  alias = date + " " + type;
} else {
  title = tp.file.title;
  title = title.trim();
  alias = date + " " + type;
}

//-------------------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET MENTAL HEALTH
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias_preset_mental}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar_value = include_arr[0];
const pillar_value_link = include_arr[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${full_type_name}?`
);

//-------------------------------------------------------------------
// SET RELATED PROJECT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_project}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const project_value = include_arr[0];
const project_name = include_arr[1];
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);

//-------------------------------------------------------------------
// SET RELATED PARENT TASK
//-------------------------------------------------------------------
let parent_task_value = "null";
let parent_task_name = "Null";
if (project_value!== "null") {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_value,
    file_class: "task",
    type: "parent_task",
  });
  const parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    `Related Parent Task to the ${full_type_name}?`
);
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
};
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);

//-------------------------------------------------------------------
// NAME FILE AND MOVE TO JOURNAL'S DIRECTORY
//-------------------------------------------------------------------
const file_name = `${date}_${type}`;
const folder_path = tp.file.folder(true) + "/";

if (folder_path!= type_dir) {
   await tp.file.move(type_dir + file_name);
};

tR += hr_line;
%>
title: "<%* tR += title %>"
uuid: <%* tR += await tp.user.uuid() %>
aliases:
  - "<%* tR += title %>"
  - "<%* tR += alias %>"
date: <%* tR += date_value_link %>
pillar: <%* tR += pillar %>
goal: <%* tR += goal %>
project: <%* tR += project %>
parent_task: <%* tR += parent_task %>
type: <%* tR += type %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title %>

> [!<%* tR += type_value %> ] <%* tR += full_type_name %> Details
>
> - **Journal Type**:: <%* tR += type_name %>
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`
> - **Parent Task**: `dv: this.file.frontmatter.parent_task`
> - **Date**: `dv: this.file.frontmatter.date`

---

## Recall

## Review Achievements

1. [achievement:: ]
2. [achievement:: ]
3. [achievement:: ]
