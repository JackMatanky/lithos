---
title: 32_09_cal_week_task_event
aliases:
  - Weekly Tasks and Events Dataview Tables
  - weekly tasks and events dataview tables
  - Weekly Tasks and Events
  - weekly tasks and events
  - cal week task event
plugin: templater
language:
  - javascript
module:
  - momentjs
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T17:27
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs, obsidian/tp/file/include
---
# Weekly Tasks and Events Dataview Tables

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a week's calendar day files.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------  
// WEEKLY TASKS AND EVENTS DATAVIEW TABLES
//---------------------------------------------------------
// TASKS: "project", "parent_task", "child_task", "task"
// COMPLETED STATUSES: "completed", "done"
// ACTIVE STATUSES: "active", "to_do", "in_progress"
// SCHEDULE STATUSES: "schedule", "on_hold"
// DETERMINE STATUSES: "undetermined", "determine"
// CREATED STATUSES: "created", "new"
const week_active_proj = await tp.user.dv_task_type_status_dates({
  type: "project",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
const week_active_parent_task = await tp.user.dv_task_type_status_dates({
  type: "parent_task",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
const tasks_due_sunday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: sunday,
  end_date: "week",
  md: "false",
});

const tasks_due_monday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: monday,
  end_date: "week",
  md: "false",
});

const tasks_due_tuesday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: tuesday,
  end_date: "week",
  md: "false",
});

const tasks_due_wednesday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: wednesday,
  end_date: "week",
  md: "false",
});

const tasks_due_thursday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: thursday,
  end_date: "week",
  md: "false",
});

const tasks_due_friday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: friday,
  end_date: "week",
  md: "false",
});

const tasks_due_saturday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: saturday,
  end_date: "week",
  md: "false",
});
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// FOLDER PATH VARIABLES
//---------------------------------------------------------
const sys_temp_include_dir = "00_system/06_template_include/";
const cal_dir = "10_calendar";
const cal_day_dir = "10_calendar/11_days";
const cal_week_dir = "10_calendar/12_weeks";
const cal_month_dir = "10_calendar/13_months";
const cal_quarter_dir = "10_calendar/14_quarters";
const cal_year_dir = "10_calendar/15_years";

//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const hr_line = hyphen.repeat(3);
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const cmnt_ob_start = `${String.fromCodePoint(37).repeat(2)}${space}`;
const cmnt_ob_end = `${space}${String.fromCodePoint(37).repeat(2)}`;
const colon = String.fromCodePoint(0x3a);
const two_colon = colon.repeat(2);
const tbl_start = `${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end = `${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${String.fromCodePoint(0x2d)}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${String.fromCodePoint(0x3e)}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_ul}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;

const tbl_div = String.fromCodePoint(0x2d).repeat(8);

const call_tbl_three = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const call_tbl_four = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

const head_one = `${hash}${space}`;
const head_two = `${hash.repeat(2)}${space}`;
const head_three = `${hash.repeat(3)}${space}`;
const head_four = `${hash.repeat(4)}${space}`;

//---------------------------------------------------------
// GENERAL VARIABLES
//---------------------------------------------------------
let toc_title = "";
let toc_body = "";
const toc_dv_contents = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;

let heading = "";
let comment = "";
let query = "";

//---------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//---------------------------------------------------------
const buttons_table_pdev_today = "00_90_buttons_table_pdev_today";
const buttons_table_task_event = "00_40_buttons_table_task_event";
const buttons_table_task_habit_today = "00_40_buttons_table_task_habit_today";
const buttons_table_habit_rit_week = "00_42_buttons_table_habit_rit_week";
const buttons_table_note = "00_80_buttons_table_notes";
const weekday_file = "31_00_days_of_week";

//---------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//---------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//---------------------------------------------------------
// BUTTONS TABLES
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${buttons_table_pdev_today}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const pdev_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_table_note}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const note_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_table_habit_rit_week}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const habit_rit_week_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_table_task_event}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const task_buttons_table = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${buttons_table_task_habit_today}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const task_habit_buttons_table = `${include_arr}${two_new_line}`;

//---------------------------------------------------------
// BUTTONS
//---------------------------------------------------------
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;
const button_comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;
let button_name = "";
let button_type = "";
let button_action = "";
let button_replace = "";
let button_color = "";

// DAY MD
button_name = `name 📆Day MD File${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 111_00_dvmd_day_file${new_line}`;
button_replace = `replace [44, 620]${new_line}`;
button_color = `color purple${new_line}`;

const day_md_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}`;

// PERSONAL DEVELOPMENT JOURNALS
button_name = `name 🕯️Weekly Journals and Attributes${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 112_90_dvmd_week_pdev${new_line}`;
button_replace = `replace [58, 400]${new_line}`;
button_color = `color purple${new_line}`;

const pdev_week_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

// LIBRARY
button_name = `name 🏫Weekly Library Content${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 112_60_dvmd_week_lib${new_line}`;
button_replace = `replace [59, 600]${new_line}`;
button_color = `color green${new_line}`;

const lib_week_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

// PKM
button_name = `name 🗃️Weekly PKM Files${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 112_70_dvmd_week_pkm${new_line}`;
button_replace = `replace [58, 500]${new_line}`;
button_color = `color green${new_line}`;

const pkm_week_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

// HABITS AND RITUALS
button_name = `name 🦾Planned Habits and Rituals${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 112_45_dvmd_week_habit_rit_due${new_line}`;
button_replace = `replace [59, 500]${new_line}`;
button_color = `color blue${new_line}`;

const habit_rit_due_week_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

button_name = `name 🦿Completed Habits and Rituals${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 112_45_dvmd_week_habit_rit_done${new_line}`;
button_color = `color blue${new_line}`;

const habit_rit_done_week_button = `${button_start}${button_name}${button_type}${button_action}${button_color}${button_end}${button_comment}`;

// TASKS AND EVENTS
button_name = `name ✅Active Tasks and Events${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 112_40_dvmd_week_tasks_due${new_line}`;
button_replace = `replace [57, 500]${new_line}`;
button_color = `color blue${new_line}`;

const tasks_events_due_week_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

button_name = `name ✅Completed Tasks and Events${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 112_41_dvmd_week_tasks_done${new_line}`;
button_color = `color blue${new_line}`;

const tasks_events_done_week_button = `${button_start}${button_name}${button_type}${button_action}${button_color}${button_end}${button_comment}`;

//---------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//---------------------------------------------------------
const week_type_name = "Week";
const week_type_value = week_type_name.toLowerCase();
const week_moment_var = `${week_type_value}s`;
const week_file_class = `cal_${week_type_value}`;

//---------------------------------------------------------
// SET THE DATE
//---------------------------------------------------------
const date_obj_arr = [
  { key: `Current ${week_type_name}`, value: `current_${week_type_value}` },
  { key: `Last ${week_type_name}`, value: `last_${week_type_value}` },
  { key: `Next ${week_type_name}`, value: `next_${week_type_value}` },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  `Which ${week_type_name}?`
);

const date_value = date_obj.value;

let full_date = "";

if (date_value.startsWith("current")) {
  full_date = moment();
} else if (date_value.startsWith("next")) {
  full_date = moment().add(1, week_moment_var);
} else {
  full_date = moment().subtract(1, week_moment_var);
}

//---------------------------------------------------------
// GENERAL WEEK DATE VARIABLES
//---------------------------------------------------------
const week_long_date = moment(full_date).format("[Week ]ww[,] YYYY");
const week_short_date = moment(full_date).format("YYYY-[W]ww");
const week_year_long = moment(full_date).format("YYYY");
const week_year_short = moment(full_date).format("YY");
const week_quarter_num = moment(full_date).format("Q");
const quarter_ord = moment(full_date).format("Qo");
const week_month_name_full = moment(full_date).format("MMMM");
const week_month_name_short = moment(full_date).format("MMM");
const week_month_num_long = moment(full_date).format("MM");
const week_month_num_short = moment(full_date).format("M");

//---------------------------------------------------------
// WEEK DATE VARIABLES
//---------------------------------------------------------
const week_number = moment(full_date).format("ww");
const week_date_start = moment(full_date)
  .startOf(week_type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const week_date_end = moment(full_date)
  .endOf(week_type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const week_date_prev = `${backtick}dvjs: dv.fileLink(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date_start).minus({days: 1}).toFormat("yyyy-'W'WW"))${backtick}`;
const week_date_next = `${backtick}dvjs: dv.fileLink(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date_start).plus({weeks: 1, days: 1}).toFormat("yyyy-'W'WW"))${backtick}`;
const week_date_prev_next = `<<${space}${week_date_prev}${tbl_pipe}${week_date_next}${space}>>`;

const week_sub_date_prev = `${backtick}dvjs: dv.fileLink((dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date_start).minus({days: 1}).toFormat("yyyy-'W'WW") + "_" + dv.current().file.frontmatter.subtype), false, dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date_start).minus({days: 1}).toFormat("yyyy-'W'WW") + " " + (dv.current().file.frontmatter.subtype == "lib"? "Library": (dv.current().file.frontmatter.subtype == "habit_ritual"? "Habits and Rituals": (dv.current().file.frontmatter.subtype == "task_event"? "Tasks and Events": dv.current().file.frontmatter.subtype.toUpperCase()))))${backtick}`;
const week_sub_date_next = `${backtick}dvjs: dv.fileLink((dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date_start).plus({weeks: 1, days: 1}).toFormat("yyyy-'W'WW") + "_" + dv.current().file.frontmatter.subtype), false, dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date_start).plus({weeks: 1, days: 1}).toFormat("yyyy-'W'WW") + " " + (dv.current().file.frontmatter.subtype == "lib"? "Library": (dv.current().file.frontmatter.subtype == "habit_ritual"? "Habits and Rituals": (dv.current().file.frontmatter.subtype == "task_event"? "Tasks and Events": dv.current().file.frontmatter.subtype.toUpperCase()))))${backtick}`;
const week_sub_date_prev_next = `<<${space}${week_sub_date_prev}${tbl_pipe}${week_sub_date_next}${space}>>${new_line}`;

//---------------------------------------------------------
// WEEKDAY DATE VARIABLES
//---------------------------------------------------------
const sunday = moment(full_date).day(0).format("YYYY-MM-DD");
const monday = moment(full_date).day(1).format("YYYY-MM-DD");
const tuesday = moment(full_date).day(2).format("YYYY-MM-DD");
const wednesday = moment(full_date).day(3).format("YYYY-MM-DD");
const thursday = moment(full_date).day(4).format("YYYY-MM-DD");
const friday = moment(full_date).day(5).format("YYYY-MM-DD");
const saturday = moment(full_date).day(6).format("YYYY-MM-DD");

//---------------------------------------------------------
// WEEKDAY FILE LINKS
//---------------------------------------------------------
const sunday_link = `[[${sunday}|Sunday]]`;
const monday_link = `[[${monday}|Monday]]`;
const tuesday_link = `[[${tuesday}|Tuesday]]`;
const wednesday_link = `[[${wednesday}|Wednesday]]`;
const thursday_link = `[[${thursday}|Thursday]]`;
const friday_link = `[[${friday}|Friday]]`;
const saturday_link = `[[${saturday}|Saturday]]`;

//---------------------------------------------------------
// WEEKDAY FILE LINK EMBEDS FOR WEEKDAYS TABLE
//---------------------------------------------------------
const sunday_table_link = `**[[${sunday}\|Sunday]]**`;
const monday_table_link = `**[[${monday}\|Monday]]**`;
const tuesday_table_link = `**[[${tuesday}\|Tuesday]]**`;
const wednesday_table_link = `**[[${wednesday}\|Wednesday]]**`;
const thursday_table_link = `**[[${thursday}\|Thursday]]**`;
const friday_table_link = `**[[${friday}\|Friday]]**`;
const saturday_table_link = `**[[${saturday}\|Saturday]]**`;

//---------------------------------------------------------
// CALENDAR FILE LINKS AND ALIASES
//---------------------------------------------------------
let year_file = `[[${week_year_long}]]`;
let quarter_file = `[[${week_year_long}-Q${week_quarter_num}]]`;
let month_file = `[[${week_year_long}-${week_month_num_long}\|${week_month_name_short} '${week_year_short}]]`;
let week_file = `[[${week_year_long}-W${week_number}]]`;

//---------------------------------------------------------
// WEEKLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const week_full_title_name = `${week_long_date}`;
const week_short_title_value = `${week_short_date}`;

const week_file_alias = `${new_line}${ul_yaml}${week_full_title_name}${new_line}${ul_yaml}${week_short_title_value}`;

const week_file_name = `${week_short_date}`;
const week_file_section = `${week_file_name}${hash}`;

const week_file_dir = `${cal_week_dir}/${week_file_name}`;

//---------------------------------------------------------
// YAML FRONTMATTER FOR INDIVIDUAL FILES
//---------------------------------------------------------
let fmatter_date_start = `date_start:${space}${week_date_start}${new_line}`;
let fmatter_date_end = `date_end:${space}${week_date_end}${new_line}`;
let fmatter_year = `year:${space}${week_year_long}${new_line}`;
let fmatter_quarter = `quarter:${space}${week_quarter_num}${new_line}`;
let fmatter_month_name = `month_name:${space}${week_month_name_full}${new_line}`;
let fmatter_month_number = `month_number:${space}${week_month_num_long}${new_line}`;
let fmatter_week_number = `week_number:${space}${week_number}${new_line}`;
let fmatter_type = `type:${space}${week_type_value}${new_line}`;
let fmatter_file_class = `file_class:${space}${week_file_class}${new_line}`;
let fmatter_cssclasses = `cssclasses:${space}null${new_line}`;
let fmatter_date_created = `date_created:${space}${date_created}${new_line}`;
let fmatter_date_modified = `date_modified:${space}${date_modified}${new_line}`;

const frontmatter_top = `${fmatter_date_start}${fmatter_date_end}${fmatter_year}${fmatter_quarter}${fmatter_month_name}${fmatter_month_number}${fmatter_week_number}`;

const frontmatter_bottom = `${fmatter_type}${fmatter_file_class}${fmatter_cssclasses}${fmatter_date_created}${fmatter_date_modified}${hr_line}${new_line}`;

//---------------------------------------------------------
// WEEK PDEV SUBFILE DETAILS
//---------------------------------------------------------
// PDEV WEEK FILE
const pdev_full_name = "Personal Development Journals";
const pdev_name = "PDEV";
const pdev_value = "pdev";
const pdev_full_title_name = `${pdev_full_name} for ${week_long_date}`;
const pdev_short_title_name = `${week_short_date} ${pdev_name}`;
const pdev_short_title_value = `${week_short_date}_${pdev_value}`;

alias_arr = [
  pdev_full_title_name,
  pdev_short_title_name,
  pdev_short_title_value,
];
let pdev_file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = `${new_line}${ul_yaml}"${alias_arr[i]}"`;
  pdev_file_alias += alias;
}

const pdev_file_name = pdev_short_title_value;
const pdev_section = `${pdev_file_name}${hash}`;
const pdev_section_embed = `![[${pdev_section}${pdev_full_name}]]`;
const pdev_file_link = `[[${pdev_file_name}|${pdev_name}]]`;
const pdev_table_link = `[[${pdev_file_name}\|${pdev_name}]]`;

const frontmatter_pdev = `${hr_line}${new_line}title:${space}${pdev_file_name}${new_line}aliases:${pdev_file_alias}${new_line}${frontmatter_top}subtype:${space}${pdev_value}${new_line}${frontmatter_bottom}`;

//---------------------------------------------------------
// WEEKLY REFLECTION FILE LINK AND CALLOUT
//---------------------------------------------------------
const reflection_alias = "Weekly Reflection";
const reflection_file_name = `${week_file_name}_${reflection_alias
  .replaceAll(/\s/g, "_")
  .toLowerCase()}`;
const reflection_link = `[[${reflection_file_name}\|${reflection_alias}]]`;
const reflection_button = `${backtick}button-reflection-weekly${backtick}`;

const reflection_call_title = `${call_start}[!reflection]${space}${reflection_alias}${new_line}${call_start}${new_line}`;
const reflection_call_table = `${call_tbl_start}${reflection_button}${tbl_pipe}${reflection_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

const reflection_callout = `${reflection_call_title}${reflection_call_table}${two_new_line}`;

//---------------------------------------------------------
// WEEKLY PDEV HEADINGS
//---------------------------------------------------------
// MD: "true", "false"
// JOURNAL: "file",
// ATTR: "recount", "best-experience", "blindspot", "achievement",
// ATTR: "gratitude", "detachment", "limiting_belief", "lesson"

// JOURNAL RECOUNTING LIST
heading = "Recount";
const head_recount = `${head_three}${heading}${two_new_line}`;
const toc_recount = `[[${pdev_section}${heading}\|Recount]]`;

// JOURNAL BEST EXPERIENCE LIST
heading = "Best Experiences";
const head_experience = `${head_three}${heading}${two_new_line}`;
const toc_best_experience = `[[${pdev_section}${heading}\|Experiences]]`;

// JOURNAL ACHIEVEMENTS LIST
heading = "Achievements";
const head_achievement = `${head_three}${heading}${two_new_line}`;
const toc_achievement = `[[${pdev_section}${heading}\|Achievements]]`;

// JOURNAL GRATITUDE LIST
heading = "Gratitude and Self Gratitude";
const head_gratitude = `${head_three}${heading}${two_new_line}`;
const toc_gratitude = `[[${pdev_section}${heading}\|Gratitude]]`;

// JOURNAL BLIND SPOTS LIST
heading = "Blind Spots";
const head_blindspot = `${head_three}${heading}${two_new_line}`;
const toc_blindspot = `[[${pdev_section}${heading}\|Blindspots]]`;

// JOURNAL DETACHMENT LIST
heading = "Detachment";
const head_detach = `${head_three}${heading}${two_new_line}`;
const toc_detach = `[[${pdev_section}${heading}\|Detachment]]`;

// JOURNAL LIMITING BELIEF LIST
heading = "Limiting Beliefs";
const head_limiting_belief = `${head_three}${heading}${two_new_line}`;
const toc_limiting_belief = `[[${pdev_section}${heading}\|Limiting Beliefs]]`;

// JOURNAL LESSONS LIST
heading = "Lessons Learned";
const head_lesson = `${head_three}${heading}${two_new_line}`;
const toc_lesson = `[[${pdev_section}${heading}\|Lessons]]`;

//---------------------------------------------------------
// WEEK PDEV SUBFILE CONTENTS CALLOUT
//---------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Week${space}${pdev_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

toc_body_head = `${call_tbl_start}${toc_recount}${tbl_pipe}${toc_best_experience}${tbl_pipe}${toc_achievement}${tbl_pipe}${toc_gratitude}${call_tbl_end}${new_line}`;
toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
toc_body_low = `${call_tbl_start}${toc_blindspot}${tbl_pipe}${toc_detach}${tbl_pipe}${toc_limiting_belief}${tbl_pipe}${toc_lesson}${call_tbl_end}`;
toc_body = `${toc_body_head}${toc_body_div}${toc_body_low}`;

let toc_pdev = `${toc_title}${toc_body}${two_new_line}`;

//---------------------------------------------------------
// WEEKLY PDEV DATAVIEW TABLE
//---------------------------------------------------------
// MD: "true", "false"
// JOURNAL: "file",
// ATTR: "recount", "best-experience", "blindspot", "achievement",
// ATTR: "gratitude", "detachment", "limiting_belief", "lesson"

// WEEK PDEV FILES
query = await tp.user.dv_pdev_attr_dates({
  attribute: "file",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const files = `${toc_pdev}${query}${two_new_line}`;

// JOURNAL RECOUNTING LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "recount",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const recount = `${head_recount}${toc_pdev}${query}${two_new_line}`;

// JOURNAL BEST EXPERIENCE LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "best-experience",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const experience = `${head_experience}${toc_pdev}${query}${two_new_line}`;

// JOURNAL ACHIEVEMENTS LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "achievement",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const achievement = `${head_achievement}${toc_pdev}${query}${two_new_line}`;
// JOURNAL GRATITUDE LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "gratitude",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const gratitude = `${head_gratitude}${toc_pdev}${query}${two_new_line}`;

// JOURNAL BLIND SPOTS LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "blindspot",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const blindspot = `${head_blindspot}${toc_pdev}${query}${two_new_line}`;

// JOURNAL DETACHMENT LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "detachment",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const detach = `${head_detach}${toc_pdev}${query}${two_new_line}`;

// JOURNAL LIMITING BELIEF LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "limiting_belief",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const limiting_belief = `${head_limiting_belief}${toc_pdev}${query}${two_new_line}`;

// JOURNAL LESSONS LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "lesson",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const lesson = `${head_lesson}${toc_pdev}${query}${two_new_line}`;

const week_pdev = `${head_two}${pdev_full_name}${two_new_line}${toc_pdev}${reflection_callout}${pdev_week_button}${files}${recount}${experience}${achievement}${gratitude}${blindspot}${detach}${limiting_belief}${lesson}`;

//---------------------------------------------------------
// WEEK LIBRARY SUBFILE DETAILS
//---------------------------------------------------------
// LIBRARY WEEK FILE
const lib_name = "Library";
const lib_value = "lib";
const lib_full_title_name = `${lib_name} for ${week_long_date}`;
const lib_short_title_name = `${week_short_date} ${lib_name}`;
const lib_short_title_value = `${week_short_date}_${lib_value}`;

alias_arr = [lib_full_title_name, lib_short_title_name, lib_short_title_value];
let lib_file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = `${new_line}${ul_yaml}"${alias_arr[i]}"`;
  lib_file_alias += alias;
}

const lib_file_name = lib_short_title_value;

const lib_section = `${lib_file_name}${hash}`;
const lib_section_embed = `![[${lib_section}${lib_name}]]`;
const lib_file_link = `[[${lib_file_name}|${lib_name}]]`;
const lib_table_link = `[[${lib_file_name}\|${lib_name}]]`;

const frontmatter_lib = `${hr_line}${new_line}title:${space}${lib_file_name}${new_line}aliases:${lib_file_alias}${new_line}${frontmatter_top}subtype:${space}${lib_value}${new_line}${frontmatter_bottom}`;

//---------------------------------------------------------
// WEEKLY LIBRARY HEADINGS
//---------------------------------------------------------
const lib_comment = `${cmnt_html_start}Limit 25${cmnt_html_end}${two_new_line}`;

// Completed
heading = "Completed This Week";
let head_lib_done = `${head_three}${heading}${two_new_line}`;
let toc_lib_done = `[[${lib_section}${heading}\|Completed]]`;

// Active Content
heading = "Active Content";
const head_lib_active = `${head_three}${heading}${two_new_line}`;
const toc_lib_active = `[[${lib_section}${heading}\|Active]]`;

// New Content
heading = "Created This Week";
let head_lib_new = `${head_three}${heading}${two_new_line}`;
let toc_lib_new = `[[${lib_section}${heading}\|New]]`;

// Content to Schedule
heading = "Content to Schedule";
const head_lib_sched = `${head_three}${heading}${two_new_line}`;
const toc_lib_sched = `[[${lib_section}${heading}\|Schedule]]`;

// Undetermined Content
heading = "Undetermined Content";
const head_lib_undetermined = `${head_three}${heading}${two_new_line}`;
const toc_lib_undetermined = `[[${lib_section}${heading}\|Undetermined]]`;

//---------------------------------------------------------
// WEEK LIBRARY SUBFILE CONTENTS CALLOUT
//---------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Week${space}${lib_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

toc_body_head = `${call_tbl_start}${toc_lib_done}${tbl_pipe}${toc_lib_active}${tbl_pipe}${toc_lib_new}${tbl_pipe}${toc_lib_sched}${tbl_pipe}${toc_lib_undetermined}${call_tbl_end}${new_line}`;
toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
toc_body = `${toc_body_head}${toc_body_div}`;

let toc_lib = `${toc_title}${toc_body}${two_new_line}`;

//---------------------------------------------------------
// WEEKLY LIBRARY DATAVIEW TABLE
//---------------------------------------------------------
// Completed
query = await tp.user.dv_lib_status_dates({
  status: "done",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let lib_done = `${head_lib_done}${toc_lib}${lib_comment}${query}${two_new_line}`;

// Active Content
query = await tp.user.dv_lib_status_dates({
  status: "active",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let lib_active = `${head_lib_active}${toc_lib}${lib_comment}${query}${two_new_line}`;

// New Content
query = await tp.user.dv_lib_status_dates({
  status: "new",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let lib_new = `${head_lib_new}${toc_lib}${lib_comment}${query}${two_new_line}`;

// Content to Schedule
query = await tp.user.dv_lib_status_dates({
  status: "schedule",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let lib_sched = `${head_lib_sched}${toc_lib}${lib_comment}${query}${two_new_line}`;

// Undetermined Content
query = await tp.user.dv_lib_status_dates({
  status: "determine",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let lib_undetermined = `${head_lib_undetermined}${toc_lib}${lib_comment}${query}${two_new_line}`;

const week_lib = `${head_two}${lib_name}${two_new_line}${toc_lib}${lib_week_button}${lib_done}${lib_active}${lib_new}${lib_sched}${lib_undetermined}`;

//---------------------------------------------------------
// WEEK PKM SUBFILE DETAILS
//---------------------------------------------------------
// PKM WEEK FILE
const pkm_full_name = "Personal Knowledge Management";
const pkm_name = "PKM";
const pkm_value = "pkm";
const pkm_full_title_name = `${pkm_full_name} for ${week_long_date}`;
const pkm_short_title_name = `${week_short_date} ${pkm_name}`;
const pkm_short_title_value = `${week_short_date}_${pkm_value}`;

alias_arr = [pkm_full_title_name, pkm_short_title_name, pkm_short_title_value];
let pkm_file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = `${new_line}${ul_yaml}"${alias_arr[i]}"`;
  pkm_file_alias += alias;
}

const pkm_file_name = pkm_short_title_value;

const pkm_section = `${pkm_file_name}${hash}`;
const pkm_section_embed = `![[${pkm_section}${pkm_full_name}]]`;
const pkm_file_link = `[[${pkm_file_name}|${pkm_name}]]`;
const pkm_table_link = `[[${pkm_file_name}\|${pkm_name}]]`;

const frontmatter_pkm = `${hr_line}${new_line}title:${space}${pkm_file_name}${new_line}aliases:${pkm_file_alias}${new_line}${frontmatter_top}subtype:${space}${pkm_value}${new_line}${frontmatter_bottom}`;

//---------------------------------------------------------
// WEEKLY PKM HEADERS
//---------------------------------------------------------
heading = "Notes Taken";
const head_notes_taken = `${head_three}${heading}${two_new_line}`;
const toc_notes_taken = `[[${pkm_section}${heading}\|Notes Taken]]`;

// Knowledge Tree
heading = "Knowledge Tree";
const head_tree = `${head_four}${heading}${two_new_line}`;
const toc_tree = `[[${pkm_section}${heading}\|Tree]]`;

// Permanent
heading = "Permanent";
const head_permanent = `${head_four}${heading}${two_new_line}`;
const toc_permanent = `[[${pkm_section}${heading}\|Permanent]]`;

// Literature
heading = "Literature";
const head_literature = `${head_four}${heading}${two_new_line}`;
const toc_literature = `[[${pkm_section}${heading}\|Literature]]`;

// Fleeting
heading = "Fleeting";
const head_fleeting = `${head_four}${heading}${two_new_line}`;
const toc_fleeting = `[[${pkm_section}${heading}\|Fleeting]]`;

// Info
heading = "General";
const head_info = `${head_four}${heading}${two_new_line}`;
const toc_info = `[[${pkm_section}${heading}\|General]]`;

heading = "Note Making";
const head_note_making = `${head_three}${heading}${two_new_line}`;
const toc_note_making = `[[${pkm_section}${heading}\|Note Making]]`;

// Review
heading = "Review";
const head_review = `${head_four}${heading}${two_new_line}`;
const toc_review = `[[${pkm_section}${heading}\|Review]]`;

// Clarify
heading = "Clarify";
const head_clarify = `${head_four}${heading}${two_new_line}`;
const toc_clarify = `[[${pkm_section}${heading}\|Clarify]]`;

// Develop
heading = "Develop";
const head_develop = `${head_four}${heading}${two_new_line}`;
const toc_develop = `[[${pkm_section}${heading}\|Develop]]`;

//---------------------------------------------------------
// WEEK PKM SUBFILE CONTENTS CALLOUT
//---------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Week${space}${pkm_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

toc_body_high = `${call_tbl_start}${toc_tree}${tbl_pipe}${toc_permanent}${tbl_pipe}${toc_literature}${tbl_pipe}${toc_fleeting}${tbl_pipe}${toc_info}${call_tbl_end}${new_line}`;
toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
toc_body_low = `${call_tbl_start}${toc_note_making}${tbl_pipe}${toc_review}${tbl_pipe}${toc_clarify}${tbl_pipe}${toc_develop}${tbl_pipe}${two_space}${call_tbl_end}`;
toc_body = `${toc_body_high}${toc_body_div}${toc_body_low}`;

let toc_pkm = `${toc_title}${toc_body}${two_new_line}`;

//---------------------------------------------------------
// WEEKLY PKM FILES DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
// Knowledge Tree
query = await tp.user.dv_pkm_type_status_dates({
  type: "tree",
  status: "",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let tree = `${head_tree}${toc_pkm}${query}${two_new_line}`;

// Permanent
query = await tp.user.dv_pkm_type_status_dates({
  type: "permanent",
  status: "",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let permanent = `${head_permanent}${toc_pkm}${query}${two_new_line}`;

// Literature
query = await tp.user.dv_pkm_type_status_dates({
  type: "literature",
  status: "",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let literature = `${head_literature}${toc_pkm}${query}${two_new_line}`;

// Fleeting
query = await tp.user.dv_pkm_type_status_dates({
  type: "fleeting",
  status: "",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let fleeting = `${head_fleeting}${toc_pkm}${query}${two_new_line}`;

// Info
query = await tp.user.dv_pkm_type_status_dates({
  type: "info",
  status: "",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
let info = `${head_info}${toc_pkm}${query}${two_new_line}`;

// Review
query = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "review",
  start_date: "",
  end_date: "",
  md: "false",
});
let review = `${head_review}${toc_pkm}${query}${two_new_line}`;

// Clarify
query = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "clarify",
  start_date: "",
  end_date: "",
  md: "false",
});
let clarify = `${head_clarify}${toc_pkm}${query}${two_new_line}`;

// Develop
query = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "develop",
  start_date: "",
  end_date: "",
  md: "false",
});
let develop = `${head_develop}${toc_pkm}${query}${two_new_line}`;

const week_pkm = `${head_two}${pkm_full_name}${two_new_line}${toc_pkm}${note_buttons_table}${pkm_week_button}${head_notes_taken}${toc_pkm}${tree}${permanent}${literature}${fleeting}${info}${head_note_making}${toc_pkm}${review}${clarify}${develop}`;

//---------------------------------------------------------
// WEEK HABITS AND RITUALS SUBFILE DETAILS
//---------------------------------------------------------
// HABITS AND RITUALS WEEK FILE
const habit_rit_name = "Habits and Rituals";
const habit_rit_value = "habit_ritual";
const habit_rit_full_title_name = `${habit_rit_name} for ${week_long_date}`;
const habit_rit_short_title_name = `${week_short_date} ${habit_rit_name}`;
const habit_rit_short_title_value = `${week_short_date}_${habit_rit_value}`;

alias_arr = [
  habit_rit_full_title_name,
  habit_rit_short_title_name,
  habit_rit_short_title_value,
];
let habit_rit_file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = `${new_line}${ul_yaml}"${alias_arr[i]}"`;
  habit_rit_file_alias += alias;
}

const habit_rit_file_name = habit_rit_short_title_value;

const habit_rit_section = `${habit_rit_file_name}${hash}`;
const habit_rit_section_embed = `![[${habit_rit_section}${habit_rit_name}]]`;
const habit_rit_file_link = `[[${habit_rit_file_name}|${habit_rit_name}]]`;
const habit_rit_table_link = `[[${habit_rit_file_name}\|${habit_rit_name}]]`;

const frontmatter_habit_rit = `${hr_line}${new_line}title:${space}${habit_rit_file_name}${new_line}aliases:${habit_rit_file_alias}${new_line}${frontmatter_top}subtype:${space}${habit_rit_value}${new_line}${frontmatter_bottom}`;

//---------------------------------------------------------
// WEEKLY HABITS AND RITUALS HEADERS
//---------------------------------------------------------
heading = "Due This Week";
const head_habit_rit_due = `${head_three}${heading}${two_new_line}`;
const due_suffix = `${space}${heading}`;

heading = "Completed This Week";
const head_habit_rit_done = `${head_three}${heading}${two_new_line}`;
const done_suffix = `${space}${heading}`;

// HABITS
heading = "Habits";
const head_habit_due = `${head_four}${heading}${due_suffix}${two_new_line}`;
const toc_habit_due = `[[${habit_rit_section}${heading}${due_suffix}\|Due]]`;
const toc_habit_done = `[[${habit_rit_section}${heading}${done_suffix}\|Done]]`;

// MORNING RITUALS
heading = "Morning Rituals";
const head_morn_rit_due = `${head_four}${heading}${due_suffix}${two_new_line}`;
const toc_morn_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\|Due]]`;
const toc_morn_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\|Done]]`;

// WORKDAY STARTUP RITUALS
heading = "Workday Startup Rituals";
const head_work_start_rit_due = `${head_four}${heading}${due_suffix}${two_new_line}`;
const toc_work_start_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\|Due]]`;
const toc_work_start_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\|Done]]`;

// WORKDAY SHUTDOWN RITUALS
heading = "Workday Shutdown Rituals";
const head_work_shut_rit_due = `${head_four}${heading}${due_suffix}${two_new_line}`;
const toc_work_shut_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\|Due]]`;
const toc_work_shut_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\|Done]]`;

// EVENING RITUALS
heading = "Evening Rituals";
const head_eve_rit_due = `${head_four}${heading}${due_suffix}${two_new_line}`;
const toc_eve_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\|Due]]`;
const toc_eve_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\|Done]]`;

//---------------------------------------------------------
// WEEK HABITS AND RITUALS SUBFILE CONTENTS CALLOUT
//---------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Week${space}${habit_rit_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

toc_body_title = `${call_tbl_start}Habits${tbl_pipe}Morning${tbl_pipe}Workday${space}Startup${tbl_pipe}Workday${space}Shutdown${tbl_pipe}Evening${call_tbl_end}${new_line}`;
toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
toc_body_due = `${call_tbl_start}${toc_habit_due}${tbl_pipe}${toc_morn_rit_due}${tbl_pipe}${toc_work_start_rit_due}${tbl_pipe}${toc_work_shut_rit_due}${tbl_pipe}${toc_eve_rit_due}${call_tbl_end}${new_line}`;
toc_body_done = `${call_tbl_start}${toc_habit_done}${tbl_pipe}${toc_morn_rit_done}${tbl_pipe}${toc_work_start_rit_done}${tbl_pipe}${toc_work_shut_rit_done}${tbl_pipe}${toc_eve_rit_done}${call_tbl_end}`;
toc_body = `${toc_body_title}${toc_body_div}${toc_body_due}${toc_body_done}`;

const toc_habit_rit = `${toc_title}${toc_body}${two_new_line}`;

//---------------------------------------------------------
// WEEKLY HABITS AND RITUALS DATAVIEW TABLES
//---------------------------------------------------------
// TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// STATUS OPTIONS: "due", "done"
// HABITS
query = await tp.user.dv_task_type_status_dates({
  type: "habit",
  status: "due",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const habit_due = `${head_habit_due}${toc_habit_rit}${query}${two_new_line}`;

// MORNING RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "morning_ritual",
  status: "due",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const morn_rit_due = `${head_morn_rit_due}${toc_habit_rit}${query}${two_new_line}`;

// WORKDAY STARTUP RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "workday_startup_ritual",
  status: "due",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const work_start_rit_due = `${head_work_start_rit_due}${toc_habit_rit}${query}${two_new_line}`;

// WORKDAY SHUTDOWN RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "workday_shutdown_ritual",
  status: "due",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const work_shut_rit_due = `${head_work_shut_rit_due}${toc_habit_rit}${query}${two_new_line}`;

// EVENING RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "evening_ritual",
  status: "due",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const eve_rit_due = `${head_eve_rit_due}${toc_habit_rit}${query}${two_new_line}`;

const week_habit_rit = `${head_two}${habit_rit_name}${two_new_line}${habit_rit_week_buttons_table}${head_habit_rit_due}${toc_habit_rit}${habit_rit_due_week_button}${habit_due}${morn_rit_due}${work_start_rit_due}${work_shut_rit_due}${eve_rit_due}${head_habit_rit_done}${toc_habit_rit}${habit_rit_done_week_button}`;

//---------------------------------------------------------
// WEEK TASKS AND EVENTS SUBFILE DETAILS
//---------------------------------------------------------
// TASKS AND EVENTS WEEK FILE
const task_event_name = "Tasks and Events";
const task_event_value = "task_event";
const task_event_full_title_name = `${task_event_name} for ${week_long_date}`;
const task_event_short_title_name = `${week_short_date} ${task_event_name}`;
const task_event_short_title_value = `${week_short_date}_${task_event_value}`;

alias_arr = [
  task_event_full_title_name,
  task_event_short_title_name,
  task_event_short_title_value,
];
let task_event_file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = `${new_line}${ul_yaml}"${alias_arr[i]}"`;
  task_event_file_alias += alias;
}

const task_event_file_name = task_event_short_title_value;

const task_event_section = `${task_event_file_name}${hash}`;
const task_event_section_embed = `![[${task_event_section}${task_event_name}]]`;
const task_event_file_link = `[[${task_event_file_name}|${task_event_name}]]`;
const task_event_table_link = `[[${task_event_file_name}\|${task_event_name}]]`;

const frontmatter_task_event = `${hr_line}${new_line}title:${space}${task_event_file_name}${new_line}aliases:${task_event_file_alias}${new_line}${frontmatter_top}subtype:${space}${task_event_value}${new_line}${frontmatter_bottom}`;

//---------------------------------------------------------
// WEEKLY TASKS AND EVENTS DATAVIEW TABLES
//---------------------------------------------------------
// ACTIVE PROJECTS
heading = "Active Projects";
const head_proj_active = `${head_three}${heading}${two_new_line}`;
const toc_proj_active = `[[${task_event_section}${heading}\|Projects]]`;

// ACTIVE PARENT TASKS
heading = "Active Parent Tasks";
const head_parent_active = `${head_three}${heading}${two_new_line}`;
const toc_parent_active = `[[${task_event_section}${heading}\|Parent Tasks]]`;

// ACTIVE PARENT TASKS
heading = "Completed Parent Tasks";
const head_parent_done = `${head_four}${heading}${two_new_line}`;
const toc_parent_done = `[[${task_event_section}${heading}\|Parent Tasks]]`;

// PLANNED TASKS
heading = "Due This Week";
const head_task_week_due = `${head_three}${heading}${two_new_line}`;
const toc_task_week_due = `[[${task_event_section}${heading}\|Tasks Due]]`;
const due_prefix = `Due on${space}`;

// COMPLETED TASKS
heading = "Completed This Week";
const head_task_week_done = `${head_three}${heading}${two_new_line}`;
const toc_task_week_done = `[[${task_event_section}${heading}\|Tasks Done]]`;
const done_prefix = `Completed on${space}`;

// SUNDAY TASKS
heading = "Sunday";
const head_sunday_due = `${head_four}${due_prefix}${heading}${two_new_line}`;
const toc_sunday_due = `[[${task_event_section}${due_prefix}${heading}\|Due]]`;
const toc_sunday_done = `[[${task_event_section}${done_prefix}${heading}\|Done]]`;

// MONDAY TASKS
heading = "Monday";
const head_monday_due = `${head_four}${due_prefix}${heading}${two_new_line}`;
const toc_monday_due = `[[${task_event_section}${due_prefix}${heading}\|Due]]`;
const toc_monday_done = `[[${task_event_section}${done_prefix}${heading}\|Done]]`;

// TUESDAY TASKS
heading = "Tuesday";
const head_tuesday_due = `${head_four}${due_prefix}${heading}${two_new_line}`;
const toc_tuesday_due = `[[${task_event_section}${due_prefix}${heading}\|Due]]`;
const toc_tuesday_done = `[[${task_event_section}${done_prefix}${heading}\|Done]]`;

// WEDNESDAY TASKS
heading = "Wednesday";
const head_wednesday_due = `${head_four}${due_prefix}${heading}${two_new_line}`;
const toc_wednesday_due = `[[${task_event_section}${due_prefix}${heading}\|Due]]`;
const toc_wednesday_done = `[[${task_event_section}${done_prefix}${heading}\|Done]]`;

// THURSDAY TASKS
heading = "Thursday";
const head_thursday_due = `${head_four}${due_prefix}${heading}${two_new_line}`;
const toc_thursday_due = `[[${task_event_section}${due_prefix}${heading}\|Due]]`;
const toc_thursday_done = `[[${task_event_section}${done_prefix}${heading}\|Done]]`;

// FRIDAY TASKS
heading = "Friday";
const head_friday_due = `${head_four}${due_prefix}${heading}${two_new_line}`;
const toc_friday_due = `[[${task_event_section}${due_prefix}${heading}\|Due]]`;
const toc_friday_done = `[[${task_event_section}${done_prefix}${heading}\|Done]]`;

// SATURDAY TASKS
heading = "Saturday";
const head_saturday_due = `${head_four}${due_prefix}${heading}${two_new_line}`;
const toc_saturday_due = `[[${task_event_section}${due_prefix}${heading}\|Due]]`;
const toc_saturday_done = `[[${task_event_section}${done_prefix}${heading}\|Done]]`;

//---------------------------------------------------------
// WEEK TASKS AND EVENTS SUBFILE CONTENTS CALLOUT
//---------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Week${space}${task_event_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

const high_level = "**High-Level Tasks and Events**";
toc_body_high_head = `${call_tbl_start}${toc_proj_active}${tbl_pipe}${toc_parent_active}${tbl_pipe}${toc_parent_done}${tbl_pipe}${toc_task_week_due}${tbl_pipe}${toc_task_week_done}${call_tbl_end}${new_line}`;
toc_body_high_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
toc_body_high = `${toc_body_high_head}${toc_body_high_div}`;

const low_level = "**Daily Tasks and Events**";
toc_body_low_head = `${call_tbl_start}Sunday${tbl_pipe}Monday${tbl_pipe}Tuesday${tbl_pipe}Wednesday${tbl_pipe}Thursday${tbl_pipe}Friday${tbl_pipe}Saturday${call_tbl_end}${new_line}`;
toc_body_low_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
toc_body_low_due = `${call_tbl_start}${toc_sunday_due}${tbl_pipe}${toc_monday_due}${tbl_pipe}${toc_tuesday_due}${tbl_pipe}${toc_wednesday_due}${tbl_pipe}${toc_thursday_due}${tbl_pipe}${toc_friday_due}${tbl_pipe}${toc_saturday_due}${call_tbl_end}${new_line}`;
toc_body_low_done = `${call_tbl_start}${toc_sunday_done}${tbl_pipe}${toc_monday_done}${tbl_pipe}${toc_tuesday_done}${tbl_pipe}${toc_wednesday_done}${tbl_pipe}${toc_thursday_done}${tbl_pipe}${toc_friday_done}${tbl_pipe}${toc_saturday_done}${call_tbl_end}`;
toc_body_low = `${toc_body_low_head}${toc_body_low_div}${toc_body_low_due}${toc_body_low_done}`;

toc_body = `${call_start}${high_level}${new_line}${call_start}${new_line}${toc_body_high}${new_line}${call_start}${new_line}${call_start}${low_level}${new_line}${call_start}${new_line}${toc_body_low}`;

const toc_tasks_events = `${toc_title}${toc_body}${two_new_line}`;
const toc_tasks_events_low = `${toc_title}${toc_body_low}${two_new_line}`;
const toc_tasks_events_high = `${toc_title}${toc_body_high}${two_new_line}`;

//---------------------------------------------------------
// WEEKLY TASKS AND EVENTS DATAVIEW TABLES
//---------------------------------------------------------
// ACTIVE PROJECTS
query = await tp.user.dv_task_type_status_dates({
  type: "project",
  status: "active",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const proj_active = `${head_proj_active}${toc_tasks_events_high}${query}${two_new_line}`;

// ACTIVE PARENT TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "parent_task",
  status: "active",
  start_date: week_date_start,
  end_date: week_date_end,
  md: "false",
});
const parent_active = `${head_parent_active}${toc_tasks_events_high}${query}${two_new_line}`;

// SUNDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: sunday,
  end_date: "week",
  md: "false",
});
const sunday_due = `${head_sunday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// MONDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: monday,
  end_date: "week",
  md: "false",
});
const monday_due = `${head_monday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// TUESDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: tuesday,
  end_date: "week",
  md: "false",
});
const tuesday_due = `${head_tuesday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// WEDNESDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: wednesday,
  end_date: "week",
  md: "false",
});
const wednesday_due = `${head_wednesday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// THURSDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: thursday,
  end_date: "week",
  md: "false",
});
const thursday_due = `${head_thursday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// FRIDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: friday,
  end_date: "week",
  md: "false",
});
const friday_due = `${head_friday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// SATURDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: saturday,
  end_date: "week",
  md: "false",
});
const saturday_due = `${head_saturday_due}${toc_tasks_events_low}${query}${two_new_line}`;

const week_tasks_events = `${head_two}${task_event_name}${two_new_line}${toc_tasks_events}${task_buttons_table}${tasks_events_due_week_button}${proj_active}${parent_active}${head_task_week_due}${sunday_due}${monday_due}${tuesday_due}${wednesday_due}${thursday_due}${friday_due}${saturday_due}${head_task_week_done}${toc_tasks_events}${tasks_events_due_week_button}`;

//---------------------------------------------------------
// WEEK CONTEXT CALLOUT
//---------------------------------------------------------
const week_title = `${call_start}[!${week_type_value}]${space}${week_type_name}${space}Context${new_line}${call_start}${new_line}`;

const week_dates_head = `${call_tbl_start}Year${tbl_pipe}Quarter${tbl_pipe}Month${call_tbl_end}${new_line}`;
const week_dates_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
const week_dates_low = `${call_tbl_start}${year_file}${tbl_pipe}${quarter_file}${tbl_pipe}${month_file}${call_tbl_end}`;
const week_dates = `${week_dates_head}${week_dates_div}${week_dates_low}${new_line}${call_start}${new_line}`;

const week_sub_dates_head = `${call_tbl_start}Year${tbl_pipe}Quarter${tbl_pipe}Month${tbl_pipe}Week${call_tbl_end}${new_line}`;
const week_sub_dates_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
const week_sub_dates_low = `${call_tbl_start}${year_file}${tbl_pipe}${quarter_file}${tbl_pipe}${month_file}${tbl_pipe}${week_file}${call_tbl_end}`;
const week_sub_dates = `${week_sub_dates_head}${week_sub_dates_div}${week_sub_dates_low}${new_line}${call_start}${new_line}`;

const week_files_title = `${call_start}**SUBFILES**${new_line}${call_start}${new_line}`;
const week_files_head = `${call_tbl_start}${pkm_table_link}${tbl_pipe}${lib_table_link}${tbl_pipe}${pdev_table_link}${tbl_pipe}${habit_rit_table_link}${tbl_pipe}${task_event_table_link}${call_tbl_end}${new_line}`;
const week_files_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const week_files = `${week_files_title}${week_files_head}${week_files_div}${new_line}${call_start}${new_line}`;

const week_days_title = `${call_start}**DAYS**${new_line}${call_start}${new_line}`;
const week_days_head = `${call_tbl_start}${sunday_table_link}${tbl_pipe}${monday_table_link}${tbl_pipe}${tuesday_table_link}${tbl_pipe}${wednesday_table_link}${tbl_pipe}${thursday_table_link}${tbl_pipe}${friday_table_link}${tbl_pipe}${saturday_table_link}${call_tbl_end}${new_line}`;
const week_days_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const week_days = `${week_days_title}${week_days_head}${week_days_div}`;

const week_context = `${week_title}${week_dates}${week_files}${week_days}${new_line}`;
const week_sub_context = `${week_title}${week_sub_dates}${week_files}${week_days}${new_line}`;

//---------------------------------------------------------
// WEEK FILE CONTENTS CALLOUT
//---------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Week${space}[[${week_file_section}${week_full_title_name}|Contents]]${two_space}${new_line}${call_start}${new_line}`;

const toc_journals = `[[${week_file_section}Personal Development Journals\|PDEV]]`;
const toc_library = `[[${week_file_section}Library\|Library]]`;
const toc_notes = `[[${week_file_section}Personal Knowledge Management\|PKM]]`;
const toc_habit_ritual = `[[${week_file_section}Habits and Rituals\|Habits and Rituals]]`;
const toc_task_event = `[[${week_file_section}Tasks and Events\|Tasks and Events]]`;

toc_body = `${call_tbl_start}${toc_journals}${tbl_pipe}${toc_library}${tbl_pipe}${toc_notes}${tbl_pipe}${toc_habit_ritual}${tbl_pipe}${toc_task_event}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

const toc_week = `${toc_title}${toc_body}${new_line}`;

//---------------------------------------------------------
// MOVE FILE TO DIRECTORY
//---------------------------------------------------------
const directory = `${week_file_dir}/`;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${week_file_name}`);
}

//---------------------------------------------------------
// PDEV WEEK FILE
//---------------------------------------------------------
const pdev_file_content = `${frontmatter_pdev}
${head_one}${pdev_full_title_name}${new_line}
${week_sub_context}
${week_sub_date_prev_next}
${hr_line}${new_line}
${week_pdev}`;

await tp.file.create_new(
  pdev_file_content,
  pdev_file_name,
  false,
  app.vault.getAbstractFileByPath(week_file_dir)
);

//---------------------------------------------------------
// LIBRARY WEEK FILE
//---------------------------------------------------------
const lib_file_content = `${frontmatter_lib}
${head_one}${lib_full_title_name}${new_line}
${week_sub_context}
${week_sub_date_prev_next}
${hr_line}${new_line} 
${week_lib}`;

await tp.file.create_new(
  lib_file_content,
  lib_file_name,
  false,
  app.vault.getAbstractFileByPath(week_file_dir)
);

//---------------------------------------------------------
// PKM WEEK FILE
//---------------------------------------------------------
const pkm_file_content = `${frontmatter_pkm}
${head_one}${pkm_full_title_name}${new_line}
${week_sub_context}
${week_sub_date_prev_next}
${hr_line}${new_line}
${week_pkm}`;

await tp.file.create_new(
  pkm_file_content,
  pkm_file_name,
  false,
  app.vault.getAbstractFileByPath(week_file_dir)
);

//---------------------------------------------------------
// HABITS AND RITUALS WEEK FILE
//---------------------------------------------------------
const habit_rit_file_content = `${frontmatter_habit_rit}
${head_one}${habit_rit_full_title_name}${new_line}
${week_sub_context}
${week_sub_date_prev_next}
${hr_line}${new_line}
${week_habit_rit}`;

await tp.file.create_new(
  habit_rit_file_content,
  habit_rit_file_name,
  false,
  app.vault.getAbstractFileByPath(week_file_dir)
);

//---------------------------------------------------------
// TASKS AND EVENTS WEEK FILE
//---------------------------------------------------------
const task_event_file_content = `${frontmatter_task_event}
${head_one}${task_event_full_title_name}${new_line}
${week_sub_context}
${week_sub_date_prev_next}
${hr_line}${new_line}
${week_tasks_events}`;

await tp.file.create_new(
  task_event_file_content,
  task_event_file_name,
  false,
  app.vault.getAbstractFileByPath(week_file_dir)
);

//---------------------------------------------------------
// WEEKDAY CALENDAR FILES
//---------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
const day_type_name = "Day";
const day_type_value = day_type_name.toLowerCase();
const day_moment_var = `${day_type_value}s`;
const day_file_class = `cal_${day_type_value}`;

const day_date_prev = `${backtick}dvjs: dv.fileLink(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date).minus({days: 1}).toFormat("yyyy-MM-dd"))${backtick}`;
const day_date_next = `${backtick}dvjs: dv.fileLink(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date).plus({days: 1}).toFormat("yyyy-MM-dd"))${backtick}`;
const day_date_prev_next = `<<${space}${day_date_prev}${tbl_pipe}${day_date_next}${space}>>`;

// WEEKDAY DATES ARRAY
const weekday_arr = [
  sunday,
  monday,
  tuesday,
  wednesday,
  thursday,
  friday,
  saturday,
];

// WEEKDAY FRONTMATTER VARIABLES
let fmatter_title;
let fmatter_alias;
let fmatter_date;
let fmatter_year_day;
let fmatter_month_day;
let fmatter_weekday_name;
let fmatter_weekday_number;
fmatter_type = `type:${space}${day_type_value}${new_line}`;
fmatter_file_class = `file_class:${space}${day_file_class}${new_line}`;
fmatter_cssclasses = `cssclasses:${new_line}${ul_yaml}/read_view_zoom${new_line}${ul_yaml}/read_wide_margin${new_line}${ul_yaml}/inline_title_hide${new_line}`;

// FILE CREATION VARIABLES
let day_file_name;
let day_file_content;
const day_directory = cal_day_dir;

// LOOP THROUGH WEEKDAY DATES ARRAY
for (var i = 0; i < weekday_arr.length; i++) {
  day_file_name = weekday_arr[i];
  day_file_section = `${day_file_name}${hash}`;
  full_date = moment(weekday_arr[i]);

  // DATE VARIABLES
  date = moment(full_date).format("YYYY-MM-DD");
  long_date = moment(full_date).format("MMMM D, YYYY");
  short_date = moment(full_date).format("YY-M-D");
  day_year_long = moment(full_date).format("YYYY");
  day_year_short = moment(full_date).format("YY");
  year_day = moment(full_date).format("DDDD");
  day_quarter_num = moment(full_date).format("Q");
  day_month_name_full = moment(full_date).format("MMMM");
  day_month_name_short = moment(full_date).format("MMM");
  day_month_num_long = moment(full_date).format("MM");
  day_month_num_short = moment(full_date).format("M");
  day_month_day_long = moment(full_date).format("DD");
  day_month_day_short = moment(full_date).format("D");
  weekday_name = moment(full_date).format("dddd");
  weekday_number = moment(full_date).format("[0]e");

  // DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
  day_full_title_name = `${weekday_name}, ${long_date}`;
  day_short_title_name = long_date;
  day_full_title_value = date;
  day_short_title_value = short_date;

  alias_arr = [
    day_full_title_name,
    day_short_title_name,
    day_short_title_value,
    day_full_title_value,
  ];
  day_file_alias = "";
  for (var j = 0; j < alias_arr.length; j++) {
    alias = `${new_line}${ul_yaml}"${alias_arr[j]}"`;
    day_file_alias += alias;
  }

  // CALENDAR FILE LINKS AND ALIASES
  year_file = `[[${day_year_long}]]`;
  quarter_file = `[[${day_year_long}-Q${day_quarter_num}]]`;
  month_file = `[[${day_year_long}-${day_month_num_long}\|${day_month_name_short} '${day_year_short}]]`;
  week_file = `[[${day_year_long}-W${week_number}]]`;

  // DAY CONTEXT CALLOUT
  context_title = `${call_start}[!${day_type_value}]${space}${day_type_name}${space}Context${new_line}${call_start}${new_line}`;
  context_header = `${call_tbl_start}Year${tbl_pipe}Quarter${tbl_pipe}Month${tbl_pipe}Week${call_tbl_end}${new_line}`;
  context_tbl_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
  context_links = `${call_tbl_start}${year_file}${tbl_pipe}${quarter_file}${tbl_pipe}${month_file}${tbl_pipe}${week_file}${call_tbl_end}`;

  context = `${context_title}${context_header}${context_tbl_div}${context_links}${two_new_line}`;

  // PDEV HEADING
  heading = "Journal Entries";
  head_pdev = `${head_two}${heading}${two_new_line}`;
  toc_pdev = `[[${day_file_section}${heading}\|PDEV]]`;

  // PKM HEADINGS
  heading = "Personal Knowledge Management";
  head_pkm = `${head_two}${heading}${two_new_line}`;
  toc_pkm = `[[${day_file_section}${heading}\|PKM]]`;

  heading = "Knowledge Tree";
  head_pkm_tree = `${head_three}${heading}${two_new_line}`;
  toc_pkm_tree = `[[${day_file_section}${heading}\|Tree]]`;

  heading = "Permanent";
  head_pkm_perm = `${head_three}${heading}${two_new_line}`;
  toc_pkm_perm = `[[${day_file_section}${heading}\|Permanent]]`;

  heading = "Literature";
  head_pkm_lit = `${head_three}${heading}${two_new_line}`;
  toc_pkm_lit = `[[${day_file_section}${heading}\|Literature]]`;

  heading = "Fleeting";
  head_pkm_fleet = `${head_three}${heading}${two_new_line}`;
  toc_pkm_fleet = `[[${day_file_section}${heading}\|Fleeting]]`;

  heading = "General Info";
  head_pkm_info = `${head_three}${heading}${two_new_line}`;
  toc_pkm_info = `[[${day_file_section}${heading}\|Info]]`;

  toc_pkm_sect = `${call_tbl_start}${toc_pkm_tree}${tbl_pipe}${toc_pkm_perm}${tbl_pipe}${toc_pkm_lit}${tbl_pipe}${toc_pkm_fleet}${tbl_pipe}${toc_pkm_info}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

  // LIBRARY HEADINGS
  heading = "Library";
  head_lib = `${head_two}${heading}${two_new_line}`;
  toc_lib = `[[${day_file_section}${heading}\|Library]]`;

  heading = "Completed Today";
  head_lib_done = `${head_three}${heading}${two_new_line}`;
  toc_lib_done = `[[${day_file_section}${heading}\|Done]]`;

  heading = "Modified Today";
  head_lib_mod = `${head_three}${heading}${two_new_line}`;
  toc_lib_mod = `[[${day_file_section}${heading}\|Modified]]`;

  heading = "Created Today";
  head_lib_new = `${head_three}${heading}${two_new_line}`;
  toc_lib_new = `[[${day_file_section}${heading}\|New]]`;

  toc_lib_sect = `${call_tbl_start}${toc_lib_done}${tbl_pipe}${toc_lib_mod}${tbl_pipe}${toc_lib_new}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

  // TASKS AND EVENTS HEADINGS
  heading = "Tasks and Events";
  head_task = `${head_two}${heading}${two_new_line}`;
  toc_task = `[[${day_file_section}${heading}\|Tasks and Events]]`;

  heading = "Due Today";
  head_task_due = `${head_three}${heading}${two_new_line}`;
  toc_task_due = `[[${day_file_section}${heading}\|Due]]`;

  heading = "Completed Today";
  head_task_done = `${head_three}${heading}${two_new_line}`;
  toc_task_done = `[[${day_file_section}${heading}\|Done]]`;

  heading = "Created Today";
  head_task_new = `${head_three}${heading}${two_new_line}`;
  toc_task_new = `[[${day_file_section}${heading}\|New]]`;

  toc_task_sect = `${call_tbl_start}${toc_task_due}${tbl_pipe}${toc_task_done}${tbl_pipe}${toc_task_new}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

  // TABLE OF CONTENTS
  toc_title = `${call_start}[!toc]${space}${day_type_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

  toc_section = `${call_tbl_start}${toc_pdev}${tbl_pipe}${toc_pkm}${tbl_pipe}${toc_lib}${tbl_pipe}${toc_task}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

  toc = `${toc_title}${toc_section}${two_new_line}`;

  // PDEV DATAVIEW LIST
  query = await tp.user.dv_pdev_date(date, "false");
  pdev = `${head_pdev}${toc}${pdev_buttons_table}${query}${two_new_line}${hr_line}${new_line}`;

  // DAILY PKM FILES DATAVIEW TABLE
  // TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
  // STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
  query = await tp.user.dv_pkm_type_status_dates({
    type: "tree",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  tree = `${head_pkm_tree}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "permanent",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  permanent = `${head_pkm_perm}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "literature",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  literature = `${head_pkm_lit}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "fleeting",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  fleeting = `${head_pkm_fleet}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "info",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  info = `${head_pkm_info}${query}${two_new_line}`;
  pkm = `${head_pkm}${toc}${note_buttons_table}${tree}${permanent}${literature}${fleeting}${info}${hr_line}${new_line}`;

  // LIBRARY DATAVIEW TABLE
  // STATUS OPTIONS: 'created', 'modified'
  comment = `${cmnt_html_start}Limit 50${cmnt_html_end}${two_new_line}`;
  query = await tp.user.dv_lib_status_dates({
    status: "done",
    start_date: date,
    end_date: "",
    md: "false",
  });
  lib_done = `${head_lib_done}${comment}${query}${two_new_line}`;

  query = await tp.user.dv_lib_status_dates({
    status: "modified",
    start_date: date,
    end_date: "",
    md: "false",
  });
  lib_mod = `${head_lib_mod}${comment}${query}${two_new_line}`;

  query = await tp.user.dv_lib_status_dates({
    status: "new",
    start_date: date,
    end_date: "",
    md: "false",
  });
  lib_new = `${head_lib_new}${comment}${query}${two_new_line}`;
  lib = `${head_lib}${toc}${lib_done}${lib_mod}${lib_new}${hr_line}${new_line}`;

  // TASKS AND EVENTS DATAVIEW TABLES
  // STATUS OPTIONS: 'due', 'done', 'new'
  query = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "due",
    start_date: date,
    end_date: "",
    md: "false",
  });
  task_due = `${head_task_due}${query}${two_new_line}`;

  query = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "done",
    start_date: date,
    end_date: "",
    md: "false",
  });
  task_done = `${head_task_done}${query}${two_new_line}`;

  query = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "new",
    start_date: date,
    end_date: "",
    md: "false",
  });
  task_new = `${head_task_new}${query}${two_new_line}`;
  task = `${head_task}${toc}${task_habit_buttons_table}${task_due}${task_done}${task_new}${hr_line}${new_line}`;

  fmatter_title = `title:${space}${day_file_name}${new_line}`;
  fmatter_alias = `aliases:${day_file_alias}${new_line}`;
  fmatter_date = `date:${space}${date}${new_line}`;
  fmatter_year = `year:${space}${day_year_long}${new_line}`;
  fmatter_year_day = `year_day:${space}${year_day}${new_line}`;
  fmatter_quarter = `quarter:${space}${day_quarter_num}${new_line}`;
  fmatter_month_name = `month_name:${space}${day_month_name_full}${new_line}`;
  fmatter_month_number = `month_number:${space}${day_month_num_long}${new_line}`;
  fmatter_month_day = `month_day:${space}${day_month_day_long}${new_line}`;
  fmatter_weekday_name = `weekday_name:${space}${weekday_name}${new_line}`;
  fmatter_weekday_number = `weekday_number:${space}${weekday_number}${new_line}`;

  frontmatter = `${hr_line}${new_line}${fmatter_title}${fmatter_alias}${fmatter_date}${fmatter_year}${fmatter_year_day}${fmatter_quarter}${fmatter_month_name}${fmatter_month_number}${fmatter_month_day}${fmatter_week_number}${fmatter_weekday_name}${fmatter_weekday_number}${fmatter_type}${fmatter_file_class}${fmatter_cssclasses}${fmatter_date_created}${fmatter_date_modified}${hr_line}`;

  day_file_content = `${frontmatter}
${head_one}${day_full_title_name}${new_line}
${context}${day_date_prev_next}${new_line}
${day_md_button}${new_line}
${hr_line}${new_line}
${pdev}${new_line}${pkm}${new_line}${lib}${new_line}${task}`;

  await tp.file.create_new(
    day_file_content,
    day_file_name,
    false,
    app.vault.getAbstractFileByPath(day_directory)
  );
}

tR += hr_line;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_00_week|Weekly Calendar Template]]
2. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[31_00_days_of_week]]
2. [[30_01_cal_date_suggester|Calendar Date Suggester]]
3. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
4. [[31_02_cal_day_titles_alias_and_file_name|Daily Calendar Titles, Alias, and File Name]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[32_05_cal_week_journal|Weekly Journal Dataview Tables]]
2. [[32_06_cal_week_library|Weekly Library Dataview Tables]]
3. [[32_07_cal_week_pkm|Weekly PKM Dataview Tables]]
4. [[32_08_cal_week_habit_ritual|Weekly Habits and Rituals Dataview Tables]]

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
	file.frontmatter.definition AS Definition
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
