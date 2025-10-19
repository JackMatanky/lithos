<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const insights_dir = "80_insight/";
const daily_reflection_dir = "80_insight/95_reflection/01_daily/";
const goals_dir = "30_goals/";

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
const pdev_journal_info_callout = "90_pdev_journal_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// JOURNAL WRITING DATE AND PREVIOUS DATE
//-------------------------------------------------------------------
const date = moment().format("YYYY-MM-DD");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const prev_date = moment().subtract(1, "days").format("YYYY-MM-DD");
const full_date_time = moment(`${date}T00:00`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

//-------------------------------------------------------------------
// JOURNAL TYPE AND FILE CLASS
//-------------------------------------------------------------------
const full_type_name = "Daily Reflection Journal";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const long_type_name = `${full_type_name.split(" ")[0]} ${full_type_name.split(" ")[1]}`;
const long_type_value = long_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[1];
const type_value = full_type_value.split("_")[1];
const subtype_name = full_type_name.split(" ")[0];
const subtype_value = full_type_value.split("_")[0];
const file_class = `pdev_${full_type_value.split("_")[2]}`;

//-------------------------------------------------------------------
// JOURNAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${date} ${full_type_name}`;
const partial_title_name = `${date} ${long_type_name}`;
const short_title_name = `${date} ${type_name}`;
const full_title_value = `${short_date_value}_${full_type_value}`;
const partial_title_value = `${short_date_value}_${long_type_value}`;
const short_title_value = `${short_date_value}_${type_value}`;

const alias_arr = [long_type_name, full_type_name, full_title_name, partial_title_name, short_title_name, full_title_value, partial_title_value, short_title_value];
let file_alias = "";
for (let i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
};

const file_name = partial_title_value;

//-------------------------------------------------------------------
// PILLAR FILE AND FULL NAME
//-------------------------------------------------------------------
const pillar_name = "Mental Health";
const pillar_value = pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const pillar_link = `[[${pillar_value}|${pillar_name}]]`;
const pillar_value_link = yaml_li(pillar_link);

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goal = "null";

//-------------------------------------------------------------------
// RELATED PROJECT FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const year_month_short = moment().format("YYYY-MM");
const year_month_long = moment().format("MMMM [']YY");

const context_name = "Habits and Rituals";
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();

const project_name = `${year_month_long} ${context_name}`;
const project_value = `${year_month_short}_${context_value}`;
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);

//-------------------------------------------------------------------
// RELATED PARENT TASK FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const habit_ritual_order = "02";
const habit_ritual_name = "Morning Rituals";
const parent_task_name = `${year_month_long} ${habit_ritual_name}`;
const parent_task_value = `${year_month_short}_${habit_ritual_order}_${habit_ritual_name.replaceAll(/\s/g, "_").toLowerCase()}`;
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);

//-------------------------------------------------------------------
// PDEV JOURNAL INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pdev_journal_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const journal_info = include_arr;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${full_type_name}${space}Details${new_line}${call_start}${new_line}`;

const info = `${info_title}${journal_info}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = daily_reflection_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
   await tp.file.move(`${directory}${file_name}`);
};

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
pillar: <%* tR += pillar_value_link %>
goal: <%* tR += goal %>
project: <%* tR += project_value_link %>
parent_task: <%* tR += parent_task_value_link %>
subtype: <%* tR += subtype_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_type_name %>

<%* tR += info %>

---

## What Happened [[<%* tR += prev_date %>|Yesterday]]?

- **Recount**:: <% tp.file.cursor(1) %>
- **Recount**::
- **Recount**::
- **Recount**::
- **Recount**::

## What Was Yesterday's Best Experience?

- **Best Experience**::

## What Unplanned Occurrences Happened?

- **Blindspot**::
- **Blindspot**::
- **Blindspot**::

## What Did I Achieve?

### 1st Achievement and Appreciation

- **Achievement**::
- **Appreciation**::

### 2nd Achievement and Appreciation

- **Achievement**::
- **Appreciation**::

### 3rd Achievement and Appreciation

- **Achievement**::
- **Appreciation**::

### 4th Achievement and Appreciation

- **Achievement**::
- **Appreciation**::

### 5th Achievement and Appreciation

- **Achievement**::
- **Appreciation**::

## What Did I Learn?

1. **Lesson**::
2. **Lesson**::
3. **Lesson**::
