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

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// SET THE DATE
//-------------------------------------------------------------------
const full_date = moment();

//-------------------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//-------------------------------------------------------------------
const type_name = "Day";
const type_value = type_name.toLowerCase();
const moment_var = `${type_value}s`;
const file_class = `cal_${type_value}`;

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const date = moment(full_date).format(`YYYY-MM-DD`);
const long_date = moment(full_date).format(`MMMM D, YYYY`);
const short_date = moment(full_date).format(`YY-M-D`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const year_day = moment(full_date).format(`DDDD`);
const quarter = moment(full_date).format(`Q`);
const month_full_name = moment(full_date).format(`MMMM`);
const month_short_name = moment(full_date).format(`MMM`);
const month_number = moment(full_date).format(`MM`);
const month_day = moment(full_date).format(`DD`);
const week_number = moment(full_date).format(`ww`);
const weekday_name = moment(full_date).format(`dddd`);
const weekday_number = moment(full_date).format(`[0]e`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`YYYY-MM-DD`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`YYYY-MM-DD`);
const two_weeks = moment(full_date)
  .add(14, moment_var)
  .format(`YYYY-MM-DD`);

//-------------------------------------------------------------------
// DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${weekday_name}, ${long_date}`;
const short_title_name = `${long_date}`;
const full_title_value = `${date}`;
const short_title_value = `${short_date}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}${short_title_value}`;

const file_name = `${date}`;

//-------------------------------------------------------------------
// CALENDAR FILE LINKS AND ALIASES
//-------------------------------------------------------------------
const year_file = `${year_full}`;
const quarter_file = `${year_full}-Q${quarter}`;
const month_file = `${year_full}-${month_number}\\|${month_short_name} '${year_short}`;
const week_file = `${year_full}-W${week_number}`;

//-------------------------------------------------------------------
// JOURNAL DATAVIEW LIST
//-------------------------------------------------------------------
const journal_list = await tp.user.dv_pdev_date(date, "false");

//-------------------------------------------------------------------
// DAILY TASKS AND EVENTS DATAVIEW TABLES
//-------------------------------------------------------------------
// STATUS OPTIONS: `due`, `done`, `new`
const tasks_due_date = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: date,
  end_date: "",
  md: "false",
});
const tasks_done_date = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: date,
  end_date: "",
  md: "false",
});
const tasks_new_date = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "new",
  start_date: date,
  end_date: "",
  md: "false",
});

//-------------------------------------------------------------------
// DAILY PKM FILES DATAVIEW TABLE
//-------------------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
const pkm_tree = await tp.user.dv_pkm_type_status_dates({
  type: "tree",
  status: "",
  start_date: date,
  end_date: "",
  md: "false",
});

const pkm_note_perm = await tp.user.dv_pkm_type_status_dates({
  type: "permanent",
  status: "",
  start_date: date,
  end_date: "",
  md: "false",
});

const pkm_note_lit = await tp.user.dv_pkm_type_status_dates({
  type: "literature",
  status: "",
  start_date: date,
  end_date: "",
  md: "false",
});

const pkm_note_fleet = await tp.user.dv_pkm_type_status_dates({
  type: "type",
  status: "",
  start_date: date,
  end_date: "",
  md: "false",
});

const pkm_note_info = await tp.user.dv_pkm_type_status_dates({
  type: "info",
  status: "",
  start_date: date,
  end_date: "",
  md: "false",
});

//-------------------------------------------------------------------
// DAILY LIBRARY DATAVIEW TABLE
//-------------------------------------------------------------------
const lib_completed = await tp.user.dv_lib_status_dates({
  status: "done",
  start_date: date,
  end_date: "",
  md: "false",
});
const lib_created = await tp.user.dv_lib_status_dates({
  status: "new",
  start_date: date,
  end_date: "",
  md: "false",
});
const lib_modified = await tp.user.dv_lib_status_dates({
  status: "modified",
  start_date: date,
  end_date: "",
  md: "false",
});

tR += "---"
%>
title: <%* tR += file_name %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
year: <%* tR += year_full %>
year_day: <%* tR += year_day %>
quarter: <%* tR += quarter %>
month_name: <%* tR += month_full_name %>
month_number: <%* tR += month_number %>
month_day: <%* tR += month_day %>
week_number: <%* tR += week_number %>
weekday_name: <%* tR += weekday_name %>
weekday_number: <%* tR += weekday_number %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
cssclasses:
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
---

# <%* tR += full_title_name %>

> [!<%* tR += type_value %> ] <%* tR += type_name %> Context
>
> |            Year            |            Quarter            |            Month            |            Week            |
> |:--------------------------:|:-----------------------------:|:---------------------------:|:--------------------------:|
> | [[<%* tR += year_file %>]] | [[<%* tR += quarter_file %>]] | [[<%* tR += month_file %>]] | [[<%* tR += week_file %>]] |

<< [[<%* tR += prev_date %>]] | [[<%* tR += next_date %>]] >>

---

## Buttons

|        Tasks and Events         |         Habits and Rituals         |         Journals          |
|:-------------------------------:|:----------------------------------:|:-------------------------:|
| `BUTTON[button-action-item-task-table]` |    `BUTTON[button-morn-rit-daily-note]`    | `BUTTON[button-reflection-daily]` |
|   `BUTTON[button-meeting-task-table]`   | `BUTTON[button-work-start-daily-note]` | `BUTTON[button-gratitude-daily]`  |
|                                 | `BUTTON[button-work-shut-daily-note]`  | `BUTTON[button-detachment-daily]` |
|                                 |    `BUTTON[button-eve-rit-daily-note]`     |                           |

## Journal Entries

| `BUTTON[button-reflection-daily]` | `BUTTON[button-gratitude-daily]` | `BUTTON[button-detachment-daily]` |
| ------------------------- | ------------------------ | ------------------------- |

<%* tR += journal_list %>

---

## Tasks and Events

| `BUTTON[button-action-item-task-table]` | `BUTTON[button-meeting-task-table]` |
| ------------------------------- | --------------------------- |

### Due Today

<%* tR += tasks_due_date %>

### Completed Today

<%* tR += tasks_done_date %>

### Created Today

<%* tR += tasks_new_date %>

---

## Knowledge

### Zettelkasten

#### Fleeting

<%* tR += note_fleet_table %>

#### Literature

<%* tR += note_lit_table %>

### Notes

#### Concept

<%* tR += note_concept_table %>

#### Definition

<%* tR += note_def_table %>

#### General

<%* tR += note_gen_table %>

---

## Library

### Created Today

<!-- Limit 50  -->

<%* tR += library_created_table %>

### Modified Today

<!-- Limit 50  -->

<%* tR += library_modified_table %>
