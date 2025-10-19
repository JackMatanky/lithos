<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const projects_dir = "40_projects/";
const insights_dir = "80_insight/";
const vision_purpose_dir = "80_insight/91_vision_and_purpose/";
const principles_values_dir = "80_insight/92_principles_and_values/";
const mindset_dir = "80_insight/93_mindset/";
const limiting_beliefs_dir = "80_insight/94_limiting_beliefs/";
const reflection_journals_dir = "80_insight/95_reflection/";
const daily_reflection_dir =
  "80_insight/95_reflection/01_daily/";
const weekly_reflection_dir =
  "80_insight/95_reflection/02_weekly/";
const monthly_reflection_dir =
  "80_insight/95_reflection/03_monthly/";
const quarterly_reflection_dir =
  "80_insight/95_reflection/04_quarterly/";
const yearly_reflection_dir =
  "80_insight/95_reflection/05_yearly/";
const gratitude_journals_dir = "80_insight/96_gratitude/";
const detachment_journals_dir = "80_insight/97_detachment/";
const prompt_journals_dir = "80_insight/98_prompt_journals/";

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
const pillar_name_alias = `20_00_pillar_name_alias`;

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format(`YYYY-MM-DD[T]HH:mm`);
const date_modified = moment().format(`YYYY-MM-DD[T]HH:mm`);

//-------------------------------------------------------------------
// CHOOSE THE JOURNAL WRITING DATE
//-------------------------------------------------------------------
// Choose the date for the journal entry
const nl_date = await tp.user.nl_date(tp);

//-------------------------------------------------------------------
// SET JOURNAL TYPE AND FILE CLASS
//-------------------------------------------------------------------
const type_obj_arr = [
  {
    name: "Gratitude",
    value: "gratitude",
    directory: gratitude_journals_dir,
  },
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
  {
    name: "Prompt",
    value: "prompt",
    directory: prompt_journals_dir,
  },
];

const type_obj = await tp.system.suggester(
  (item) => item.name,
  type_obj_arr,
  false,
  "Journal Type?"
);

const type_name = type_obj.name;
const type = type_obj.value;
const type_dir = type_obj.directory;
const file_class = `pdev_journal`;

//-------------------------------------------------------------------
// SET THE JOURNAL TITLE
//-------------------------------------------------------------------
// Check if note already has title
const journal_title = !type.startsWith("prompt");
let title;
let alias;

// If note does not have title,
// prompt for title and rename file
if (!journal_title) {
  title = await tp.system.prompt(
    "What is the journal entry's prompt?",
    "",
    true,
    false
  );
  title = title.trim();
  alias = title.replaceAll(/\s/g, "_").toLowerCase();
} else {
  title = type_name;
  alias = type;
}

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
const goal = await tp.system.suggester(goals, goals, false, `Goal?`);

//-------------------------------------------------------------------
// SET RELATED PROJECT
//-------------------------------------------------------------------
// Filter array to only include project folder paths based on task context
const projects = await tp.user.folder_name({
  dir: projects_dir,
  index: 2,
});

// Choose a project
const project = await tp.system.suggester(
  projects,
  projects,
  false,
  `Is this journal entry related to a project?`
);

//-------------------------------------------------------------------
// SET RELATED PARENT TASK
//-------------------------------------------------------------------
let parent_task;
if (project !== "null") {
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
    `Is this journal entry related to the project's parent tasks?`
  );
} else {
  parent_task = `null`;
}

//-------------------------------------------------------------------
// NAME FILE AND MOVE TO JOURNAL'S DIRECTORY
//-------------------------------------------------------------------
const file_name = `${nl_date}_${type}`;
const folder_path = tp.file.folder(true) + "/";

if (folder_path != type_dir) {
  await tp.file.move(type_dir + file_name);
}

tR += "---";
%>
title: "<%* tR += title %>"
aliases:
  - "<%* tR += title %>"
  - "<%* tR += alias %>"
date: <%* tR += nl_date %>
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

> [!journal] <%* tR += full_type_name %> Details
>
> - **Journal Type**:: <%* tR += type_name %>
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`
> - **Parent Task**: `dv: this.file.frontmatter.parent_task`
> - **Date**: `dv: this.file.frontmatter.date`

---
