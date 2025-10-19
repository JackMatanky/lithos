<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const cal_dir = "10_calendar";
const cal_day_dir = "10_calendar/11_days";
const cal_week_dir = "10_calendar/12_weeks";
const cal_month_dir = "10_calendar/13_months";
const cal_quarter_dir = "10_calendar/14_quarters";
const cal_year_dir = "10_calendar/15_years";

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

const tbl_div = String.fromCodePoint(0x2d).repeat(8);

const call_tbl_three = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const call_tbl_four = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
let heading = "";
let comment = "";
let query = "";

let toc_title = "";
let toc_body = "";
const dv_toc_title = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;

const dv_current_yaml = "dv.current().file.frontmatter.";
const dv_luxon_iso = "dv.luxon.DateTime.fromISO";
const dv_luxon_format_quarter = `.toFormat("yyyy-'Q'q")`;
const dv_luxon_format_month = `.toFormat("yyyy-MM")`;

//-------------------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const buttons_callout_task_event = "00_40_buttons_callout_task_event";
const buttons_table_task_habit_today = "00_40_buttons_table_task_habit_today";
const buttons_table_habit_rit_week = "00_42_buttons_table_habit_rit_week";
const buttons_callout_notes = "00_80_buttons_callout_notes";
const buttons_callout_pdev_today = "00_90_buttons_callout_pdev_today";

//-------------------------------------------------------------------
// BUTTONS TABLES
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${buttons_callout_task_event}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const task_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_table_task_habit_today}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const task_habit_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_table_habit_rit_week}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const habit_rit_week_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_callout_notes}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const note_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_callout_pdev_today}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const pdev_buttons_table = `${include_arr}${two_new_line}`;

//-------------------------------------------------------------------
// BUTTONS
//-------------------------------------------------------------------
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;
const button_comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;
let button_name = "";
let button_type = "";
let button_action = "";
let button_replace = "";
let button_color = "";

// PERSONAL DEVELOPMENT JOURNALS
button_name = `name 🕯️Quarterly Journals and Attributes${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 114_80_dvmd_quarter_pdev${new_line}`;
button_replace = `replace [63, 500]${new_line}`;
button_color = `color purple${new_line}`;

const pdev_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

// LIBRARY
button_name = `name 🏫Quarterly Library Content${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 114_60_dvmd_quarter_lib${new_line}`;
button_replace = `replace [57, 600]${new_line}`;
button_color = `color green${new_line}`;

const lib_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

// PKM
button_name = `name 🗃️Quarterly PKM Files${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 114_70_dvmd_quarter_pkm${new_line}`;
button_replace = `replace [64, 600]${new_line}`;
button_color = `color green${new_line}`;

const pkm_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

// HABITS AND RITUALS
button_name = `name 🦾Planned Habits and Rituals${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 114_45_dvmd_quarter_habit_rit_due${new_line}`;
button_replace = `replace [65, 500]${new_line}`;
button_color = `color blue${new_line}`;

const habit_rit_due_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

button_name = `name 🦿Completed Habits and Rituals${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 114_45_dvmd_quarter_habit_rit_done${new_line}`;
button_color = `color blue${new_line}`;

const habit_rit_done_button = `${button_start}${button_name}${button_type}${button_action}${button_color}${button_end}${button_comment}`;

// TASKS AND EVENTS
button_name = `name ✅Active Tasks and Events${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 114_40_dvmd_quarter_tasks_due${new_line}`;
button_replace = `replace [70, 500]${new_line}`;
button_color = `color blue${new_line}`;

const tasks_events_due_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

button_name = `name ✅Completed Tasks and Events${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 114_41_dvmd_quarter_tasks_done${new_line}`;
button_color = `color blue${new_line}`;

const tasks_events_done_button = `${button_start}${button_name}${button_type}${button_action}${button_color}${button_end}${button_comment}`;

//-------------------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//-------------------------------------------------------------------
const type_name = "Quarter";
const type_value = type_name.toLowerCase();
const moment_var = `${type_value}s`;
const file_class = `cal_${type_value}`;

//-------------------------------------------------------------------
// SET THE DATE
//-------------------------------------------------------------------
const date_obj_arr = [
  { key: `Current ${type_name}`, value: `current_${type_value}` },
  { key: `Last ${type_name}`, value: `last_${type_value}` },
  { key: `Next ${type_name}`, value: `next_${type_value}` },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  `Which ${type_name}?`
);

const date_value = date_obj.value;

let full_date = ``;

if (date_value.startsWith(`current`)) {
  full_date = moment();
} else if (date_value.startsWith(`next`)) {
  full_date = moment().add(1, moment_var);
} else {
  full_date = moment().subtract(1, moment_var);
};

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const year_long = moment(full_date).format("YYYY");
const year_short = moment(full_date).format("YY");
const quarter_long_date = moment(full_date).format("Qo [ Quarter of ] YYYY");
const quarter_med_date = moment(full_date).format("[Q]Q YYYY");
const quarter_short_date = moment(full_date).format("[Q]Q [']YY");
const quarter_date_value = moment(full_date).format("YYYY-[Q]Q");
const month_long_date = moment(full_date).format(`MMMM YYYY`);
const month_med_date = moment(full_date).format(`MMM [']YY`);
const month_short_date = moment(full_date).format(`YYYY-MM`);
const quarter = moment(full_date).format("Q");
const month_full_name = moment(full_date).format(`MMMM`);
const month_number = moment(full_date).format(`MM`);
const quarter_date_start = moment(full_date)
  .startOf(type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const quarter_date_end = moment(full_date)
  .endOf(type_value)
  .format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// INLINE DATAVIEW DATE FILE LINKS
//-------------------------------------------------------------------
const quarter_date_prev = `${backtick}dvjs: dv.fileLink(${dv_luxon_iso}(${dv_current_yaml}date_start).minus({days: 1})${dv_luxon_format_quarter})${backtick}`;
const quarter_date_next = `${backtick}dvjs: dv.fileLink(${dv_luxon_iso}(${dv_current_yaml}date_start).plus({months: 3, days: 1})${dv_luxon_format_quarter})${backtick}`;
const date_prev_next = `<<${space}${quarter_date_prev}${tbl_pipe}${quarter_date_next}${space}>>`;

const sub_date_prev = `${backtick}dvjs: dv.fileLink((${dv_luxon_iso}(${dv_current_yaml}date_start).minus({days: 1})${dv_luxon_format_quarter} + "_" + ${dv_current_yaml}type), false, ${dv_luxon_iso}(${dv_current_yaml}date_start).minus({days: 1})${dv_luxon_format_quarter} + " " + (${dv_current_yaml}type == "lib"? "Library": (${dv_current_yaml}type == "habit_ritual"? "Habits and Rituals": (${dv_current_yaml}type == "task_event"? "Tasks and Events": ${dv_current_yaml}type.toUpperCase()))))${backtick}`;
const sub_date_next = `${backtick}dvjs: dv.fileLink((${dv_luxon_iso}(${dv_current_yaml}date_start).plus({months: 3, days: 1})${dv_luxon_format_quarter} + "_" + ${dv_current_yaml}type), false, ${dv_luxon_iso}(${dv_current_yaml}date_start).plus({months: 3, days: 1})${dv_luxon_format_quarter} + " " + (${dv_current_yaml}type == "lib"? "Library": (${dv_current_yaml}type == "habit_ritual"? "Habits and Rituals": (${dv_current_yaml}type == "task_event"? "Tasks and Events": ${dv_current_yaml}type.toUpperCase()))))${backtick}`;
const sub_date_prev_next = `<<${space}${sub_date_prev}${tbl_pipe}${sub_date_next}${space}>>${new_line}`;

//-------------------------------------------------------------------
// CALENDAR FILE LINKS AND ALIASES
//-------------------------------------------------------------------
const year_file = year_long;
const quarter_file = `[[${year_long}-Q${week_quarter_num}]]`;

//-------------------------------------------------------------------
// WEEK CONTEXT CALLOUT
//-------------------------------------------------------------------
const context_title = `${call_start}[!${type_value}]${space}${type_name}${space}Context${new_line}${call_start}${new_line}`;

const quarter_context_date = `${call_start}Year${dv_colon}${year_file}${new_line}${call_start}${new_line}`;

const quarter_sub_context_date = `${call_start}Year${dv_colon}${year_file}${new_line}${call_start}Quarter${dv_colon}${quarter_file}${new_line}${call_start}${new_line}`;

const subfiles_title = `${call_start}**SUBFILES**${new_line}${call_start}${new_line}`;
const subfiles_high = `${call_tbl_start}${pkm_table_link}${tbl_pipe}${lib_table_link}${tbl_pipe}${pdev_table_link}${tbl_pipe}${habit_rit_table_link}${tbl_pipe}${task_event_table_link}${call_tbl_end}${new_line}`;
const subfiles_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const subfiles = `${subfiles_title}${subfiles_high}${subfiles_div}${new_line}${call_start}${new_line}`;

const month_title = `${call_start}**MONTHS**${new_line}${call_start}${new_line}`;
const month_high = `${call_tbl_start}${sunday_table_link}${tbl_pipe}${monday_table_link}${tbl_pipe}${tuesday_table_link}${tbl_pipe}${wednesday_table_link}${tbl_pipe}${thursday_table_link}${tbl_pipe}${friday_table_link}${tbl_pipe}${saturday_table_link}${call_tbl_end}${new_line}`;
const month_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const quarter_month = `${month_title}${month_high}${month_div}`;

const quarter_context = `${context_title}${quarter_context_date}${subfiles}${quarter_month}${new_line}`;
const quarter_sub_context = `${context_title}${quarter_sub_context_date}${subfiles}${quarter_month}${new_line}`;

//-------------------------------------------------------------------
// QUARTERLY CALENDAR TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = quarter_long_date;
const short_title_name = quarter_med_date;
const short_title_value = quarter_short_date;
const file_name = moment(full_date).format("YYYY-[Q]Q");
const file_section = `${file_name}${hash}`;

const alias_arr = [
  full_title_name,
  short_title_name,
  short_title_value,
  file_name
];
let file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
}

//-------------------------------------------------------------------
// FILE CONTENTS CALLOUT
//-------------------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Quarter${space}[[${file_section}${full_title_name}|Contents]]${two_space}${new_line}${call_start}${new_line}`;

const toc_journals = `[[${file_section}Personal Development Journals\\|PDEV]]`;
const toc_library = `[[${file_section}Library\\|Library]]`;
const toc_notes = `[[${file_section}Related Notes\]]`;
const toc_habit_ritual = `[[${file_section}Habits and Rituals\\|Habits and Rituals]]`;
const toc_task_event = `[[${file_section}Tasks and Events\\|Tasks and Events]]`;

toc_high = `${call_tbl_start}${toc_journals}${tbl_pipe}${toc_library}${tbl_pipe}${toc_notes}${tbl_pipe}${toc_habit_ritual}${tbl_pipe}${toc_task_event}${call_tbl_end}${new_line}`;
toc_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
toc_body = `${toc_high}${toc_div}`;

const toc_quarter = `${toc_title}${toc_body}${new_line}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = cal_quarter_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
   await tp.file.move(`${directory}${file_name}`);
};


tR += hr_line;
%>
title: <%* tR += file_name %>
aliases: <%* tR += file_alias %>
date_start: <%* tR += date_start %>
date_end: <%* tR += date_end %>
year: <%* tR += year_long %>
quarter: <%* tR += quarter %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>

# <%* tR += full_title_name %>

<%* tR += quarter_context %>
<%* tR += date_prev_next %>

---

## Personal Development Journals

<%* tR += toc_quarter %>
<%* tR += pdev_section_embed %>

review the following journals:

1. monthly reflections
2. monthly accountability reviews
3. vision

Revise vision

---

## Library

<%* tR += toc_quarter %>
<%* tR += lib_section_embed %>

only completed content

---

## Personal Knowledge Management

<%* tR += toc_week %>
<%* tR += pkm_section_embed %>

---

## Habits and Rituals

<%* tR += toc_quarter %>
<%* tR += habit_rit_section_embed %>

daily and weekly completion average per parent task and project

---

## Tasks and Events

<%* tR += toc_quarter %>
<%* tR += task_event_section_embed %>

goals sorted by type, duration
projects and parent tasks
execution plan (tactic and due date)
