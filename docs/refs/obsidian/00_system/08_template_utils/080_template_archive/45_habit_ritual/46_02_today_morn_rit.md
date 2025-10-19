<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const habit_ritual_proj_dir = "45_habit_ritual/";

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
const buttons_table_task_event = "00_40_buttons_table_task_event";
const task_status = "40_task_status";
const child_task_info_callout = "42_child_task_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

/* ---------------------------------------------------------- */
/*                      SET DATE AND TIME                     */
/* ---------------------------------------------------------- */
const date = moment().format("YYYY-MM-DD");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const time = await tp.user.nl_time(tp, "");
const full_date_time = moment(`${date}T${time}`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

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
// PROJECT FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const year_month_short = moment(full_date_time).format("YYYY-MM");
const year_month_long = moment(full_date_time).format("MMMM [']YY");

const project_name = `${year_month_long} ${context_name}`;
const project_value = `${year_month_short}_${context_value}`;
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);
const project_dir = `${context_dir}${project_value}/`;

//-------------------------------------------------------------------
// PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const habit_ritual_order = "02";
const habit_ritual_name = "Morning Rituals";
const parent_task_name = `${year_month_long} ${habit_ritual_name}`;
const parent_task_value = `${year_month_short}_${habit_ritual_order}_${habit_ritual_name.replaceAll(/\s/g, "_").toLowerCase()}`;
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);
const parent_task_dir = `${project_dir}${parent_task_value}/`;

//-------------------------------------------------------------------
// RITUAL TASK TAG, TYPE NAMES, AND FILE CLASS
//-------------------------------------------------------------------
const task_tag = "#task";
const full_type_name = `Daily ${habit_ritual_name}`;
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name
  .split(" ")
  .splice(1, full_type_name.split(" ").length)
  .join(" ");
const type_lower = type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_value = type_lower.slice(0, -1);
const file_class = "task_child";

//-------------------------------------------------------------------
// RITUAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${short_date} ${full_type_name}`;
const partial_title_name = `${short_date} ${type_name}`;
const short_title_name = full_type_name.toLowerCase();
const full_title_value = `${short_date_value}_${full_type_value}`;
const partial_title_value = `${short_date_value}_${type_lower}`;
const short_title_value = full_type_value;


const file_name = `${short_date_value}_${type_lower}`;

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
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${full_type_name}?`
);

//-------------------------------------------------------------------
// ORGANIZATION
//-------------------------------------------------------------------
const organization_value = "null";
const organization_name = "Null";
const organization_link = `[[${organization_value}|${organization_name}]]`;
const organization_value_link = yaml_li(organization_link);

//-------------------------------------------------------------------
// CONTACT
//-------------------------------------------------------------------
const contact_value = "null";
const contact_name = "Null";
const contact_link = `[[${contact_value}|${contact_name}]]`;
const contact_value_link = yaml_li(contact_link);

//-------------------------------------------------------------------
// DO/DUE DATE
//-------------------------------------------------------------------
const due_do_value = "do";

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, "40_task_status");
const status_value = task_status.split(";")[0];
const status_name = task_status.split(";")[1];
const status_symbol = task_status.split(";")[2];

const checkbox_task_tag = `-${space}[${status_symbol}]${space}${task_tag}${space}`;

//-------------------------------------------------------------------
// SELF AFFIRMATION START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const affirm_duration = 3;
const affirm_start = moment(full_date_time).format("HH:mm");
const affirm_end = moment(full_date_time)
  .add(affirm_duration, "minutes")
  .format("HH:mm");
const affirm_reminder = moment(full_date_time)
  .subtract(5, "minutes")
  .format("YYYY-MM-DD HH:mm");

const affirm_title = "Morning Affirmations";

// Morning self Affirmation task checkbox
const affirm_task_checkbox = `${checkbox_task_tag}${affirm_title}_${type_value} [time_start:: ${affirm_start}]  [time_end:: ${affirm_end}]  [duration_est:: ${affirm_duration}] ⏰ ${affirm_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// DAILY PREVIEW START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const preview_duration = 10;
const preview_start = moment(`${date}T${affirm_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const preview_end = moment(`${date}T${preview_start}`)
  .add(preview_duration, "minutes")
  .format("HH:mm");



const preview_title = "Preview Today's Schedule";

// Daily preview task checkbox
const preview_task_checkbox = `${checkbox_task_tag}${preview_title}_${type_value} [time_start:: ${preview_start}]  [time_end:: ${preview_end}]  [duration_est:: ${preview_duration}] ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

// DAILY SCHEDULE PREVIEW AND REVIEW DATAVIEW TABLE
// STATUS_ACTION OPTIONS: "due", "done", "new", "preview", "review"
const preview_task_table = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "preview",
  start_date: date,
  end_date: "",
  md: "false",
});

// TASKS AND EVENTS BUTTONS
temp_file_path = `${sys_temp_include_dir}${buttons_table_task_event}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const task_event_buttons = include_arr;

//-------------------------------------------------------------------
// DAILY REFLECTION START AND END TIMES
//-------------------------------------------------------------------
const reflection_duration = 15;
const reflection_start = moment(`${date}T${preview_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const reflection_end = moment(`${date}T${reflection_start}`)
  .add(reflection_duration, "minutes")
  .format("HH:mm");

const reflection_alias = "Daily Reflection";
const reflection_file_name = `${date}_${reflection_alias.replaceAll(/\s/g, "_").toLowerCase()}`;

const reflection_task_checkbox = `${checkbox_task_tag}${reflection_alias}_${type_value} [time_start:: ${reflection_start}]  [time_end:: ${reflection_end}]  [duration_est:: ${reflection_duration}] ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

const reflection_link = `[[${reflection_file_name}\\|${reflection_alias}]]`;

//-------------------------------------------------------------------
// DAILY GRATITUDE START AND END TIMES
//-------------------------------------------------------------------
const gratitude_duration = 3;
const gratitude_start = moment(`${date}T${reflection_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const gratitude_end = moment(`${date}T${gratitude_start}`)
  .add(gratitude_duration, "minutes")
  .format("HH:mm");

const gratitude_alias = "Daily Gratitude";
const gratitude_file_name = `${date}_${gratitude_alias.replaceAll(/\s/g, "_").toLowerCase()}`;

const gratitude_task_checkbox = `${checkbox_task_tag}${gratitude_alias}_${type_value} [time_start:: ${gratitude_start}]  [time_end:: ${gratitude_end}]  [duration_est:: ${gratitude_duration}] ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

const gratitude_link = `[[${gratitude_file_name}\\|${gratitude_alias}]]`;

//-------------------------------------------------------------------
// FIVE-MIN MEDITATION START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
// Although meditation concludes my morning rituals,
// it comes after reviewing my email and whatsapp,
// which I consider part of my workday.
// As of now, I assume the above will take me twelve minutes.
const workday_startup_duration = 13;

const meditation_duration = 10;
const meditation_start = moment(`${date}T${gratitude_end}`)
  .add(workday_startup_duration, "minutes")
  .format("HH:mm");
const meditation_end = moment(`${date}T${meditation_start}`)
  .add(meditation_duration, "minutes")
  .format("HH:mm");
const meditation_reminder = moment(`${date}T${meditation_start}`)
  .subtract(5, "minutes")
  .format("YYYY-MM-DD HH:mm");

const meditation_title = "Morning Meditation";

// Meditation task
const meditation_task_checkbox = `${checkbox_task_tag}${meditation_title}_${type_value} [time_start:: ${meditation_start}]  [time_end:: ${meditation_end}]  [duration_est:: ${meditation_duration}] ⏰ ${meditation_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// DAILY HABIT AND RITUAL FILE LINKS
//-------------------------------------------------------------------
const habit_order = "01";
const full_habit_name = "Bi-Daily Habits";
const habit_value = full_habit_name.replaceAll(/[\s-]/g, "_").toLowerCase();
const habit_type = context_name.split(" ")[0].replaceAll(/s$/g, "").toLowerCase();
const habit_link = `[[${short_date_value}_${habit_value}\\|${full_habit_name}]]`;

const work_start_order = "03";
const full_work_start_name = "Daily Workday Startup Rituals";
const work_start_name = full_work_start_name
  .split(" ")
  .splice(1, full_work_start_name.split(" ").length)
  .join(" ");
const work_start_value = work_start_name.replaceAll(/\s/g, "_").toLowerCase();
const work_start_type = work_start_value.slice(0, -1);
const work_start_link = `[[${short_date_value}_${work_start_value}\\|${full_work_start_name}]]`;

const work_shut_order = "04";
const full_work_shut_name = "Daily Workday Shutdown Rituals";
const work_shut_name = full_work_shut_name
  .split(" ")
  .splice(1, full_work_shut_name.split(" ").length)
  .join(" ");
const work_shut_value = work_shut_name.replaceAll(/\s/g, "_").toLowerCase();
const work_shut_type = work_shut_value.slice(0, -1);
const work_shut_link = `[[${short_date_value}_${work_shut_value}\\|${full_work_shut_name}]]`;

const eve_rit_order = "05";
const full_eve_rit_name = "Daily Evening Rituals";
const eve_rit_name = full_eve_rit_name
  .split(" ")
  .splice(1, full_eve_rit_name.split(" ").length)
  .join(" ");
const eve_rit_value = eve_rit_name.replaceAll(/\s/g, "_").toLowerCase();
const eve_rit_type = eve_rit_value.slice(0, -1);
const eve_rit_link = `[[${short_date_value}_${eve_rit_value}\\|${full_eve_rit_name}]]`;

//-------------------------------------------------------------------
// CHILD TASK INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${child_task_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const child_task_info = include_arr;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${context_value}]${space}${type_name}${space}Details${new_line}${call_start}${new_line}`;

const dv_date = `${backtick}dv:${space}this.file.frontmatter.date${backtick}`;
const info_date = `${call_start}Date::${space}${dv_date}${two_space}`;

const info_body = `${child_task_info}${new_line}${info_date}`;

const info = `${info_title}${info_body}${two_new_line}${hr_line}${new_line}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = parent_task_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
due_do: <%* tR += due_do_value %>
pillar: <%* tR += pillar_value_link %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
project: <%* tR += project_value_link %>
parent_task: <%* tR += parent_task_value_link %>
organization: <%* tR += organization_value_link %>
contact: <%* tR += contact_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_type_name %>

<%* tR += info %>

## <%* tR += type_name %>

> [!<%* tR += context_value %> ] Today's <%* tR += context_name %>
>
> | <%* tR += context_name %> | Link |
> |:-------------------------- |:----------------------- |
> | `BUTTON[button-habit-today]` | <%* tR += habit_link %> |
> | `BUTTON[button-work-start-today]` | <%* tR += work_start_link %> |
> | `BUTTON[button-work-shut-today]`  | <%* tR += work_shut_link %>  |
> | `BUTTON[button-eve-rit-today]`        | <%* tR += eve_rit_link %> |

### Morning Self Affirmations

<%* tR += affirm_task_checkbox %>

### Preview Today’s Schedule

<%* tR += preview_task_checkbox %>

<%* tR += task_event_buttons %>

```button
name Daily Task Schedule
type append template
action 111_40_dvmd_day_tasks_due
replace [64, 104]
color yellow
```

#### Daily Schedule

<%* tR += preview_task_table %>

### Recount Yesterday

<%* tR += reflection_task_checkbox %>

> [!reflection] Daily Reflection Journal
>
> | `BUTTON[button-reflection-daily-preset]` | <%* tR += reflection_link %> |
> | -------------------------------- | --------------------------------- |

### Give Thanks

<%* tR += gratitude_task_checkbox %>

> [!gratitude] Gratitude Journal
>
> | `BUTTON[button-gratitude-daily-preset]` | <%* tR += gratitude_link %> |
> | ------------------------------- | -------------------------------- |

---

> [!<%* tR += work_start_type %> ] Today's Workday Startup Rituals
>
> | `BUTTON[button-work-start-today]` | <%* tR += work_start_link %> |
> | -------- | ------- |

---

### Morning Meditation

<%* tR += meditation_task_checkbox %>

#### Mindfulness Bell Meditation

![[1_Five Minute Mindfulness Bell Meditation.mp3]]

#### Positive Mind Meditation

![[morn_1_Positive Mind in 5 Minutes Meditation_Jason Stephenson.mp3]]

---

## Related

### Tasks and Events

### Notes

---

## Resources
