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
const task_status = "40_task_status";
const child_task_info_callout = "42_child_task_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// SET DATE
//-------------------------------------------------------------------
const date = moment().add(1, "days").format("YYYY-MM-DD");
const short_date = moment(`${date}T00:00`).format("YY-MM-DD")
const short_date_value = moment(`${date}T00:00`).format("YY_MM_DD")

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
const year_month_short = moment(date).format("YYYY-MM");
const year_month_long = moment(date).format("MMMM [']YY");

const project_value = `${year_month_short}_${context_value}`;
const project_name = `${year_month_long} ${context_name}`;
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);
const project_dir = `${context_dir}${project_value}/`;

//-------------------------------------------------------------------
// PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const habit_ritual_order = "01";
const habit_ritual_name = "Habits";
const parent_task_value = `${year_month_short}_${habit_ritual_order}_${habit_ritual_name.toLowerCase()}`;
const parent_task_name = `${year_month_long} ${habit_ritual_name}`;
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);
const parent_task_dir = `${project_dir}${parent_task_value}/`;

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
const title = "Bi-Daily Habits";

//-------------------------------------------------------------------
// PILLAR FILES AND FULL NAMES
//-------------------------------------------------------------------
const mental_pillar_name = "Mental Health";
const mental_pillar_value = mental_pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const mental_pillar_link = `[[${mental_pillar_value}|${mental_pillar_name}]]`;

const physical_pillar_name = "Physical Health";
const physical_pillar_value = physical_pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const physical_pillar_link = `[[${physical_pillar_value}|${physical_pillar_name}]]`;

const pillar_value = `[${mental_pillar_value}, ${physical_pillar_value}]`;
const pillar_link = `${mental_pillar_link}, ${physical_pillar_link}`;
const pillar_value_link = yaml_li(pillar_link);

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, `Goal for ${title}?`);

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

//-------------------------------------------------------------------
// TASK STATUS AND SYMBOL
//-------------------------------------------------------------------
const status_name = "To do";
const status_value = status_name.replaceAll(/\s/g, "_").toLowerCase();
const status_symbol = " ";

const checkbox_task_tag = `-${space}[${status_symbol}]${space}${task_tag}${space}`;

//-------------------------------------------------------------------
// DETACHMENT FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const detachment_alias = "Daily Detachment";
const detachment_file_name = `${date}_${detachment_alias
  .replaceAll(/\s/g, "_")
  .toLowerCase()}`;
const detachment_link = `[[${detachment_file_name}\\|${detachment_alias}]]`;

//-------------------------------------------------------------------
// DETACHMENT BUTTON AND CALLOUT
//-------------------------------------------------------------------
const detach_button = `${backtick}button-detachment-daily-preset${backtick}`;
const detach_callout_title = `${call_start}[!detachment]${space}${detachment_alias}${space}Journal${new_line}${call_start}${new_line}`;
const detach_callout_table = `${call_tbl_start}${detach_button}${tbl_pipe}${detachment_link}${call_tbl_end}${call_tbl_start}----------${tbl_pipe}----------${call_tbl_end}`;
detach_callout = `${detach_callout_title}${detach_callout_table}${new_line}`;

//-------------------------------------------------------------------
// GRATITUDE FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const gratitude_alias = "Daily Self Gratitude";
const gratitude_file_name = `${date}_${gratitude_alias.split(" ")[0].toLowerCase()}_${gratitude_alias.split(" ")[2].toLowerCase()}`;
const gratitude_link = `[[${gratitude_file_name}\\|${gratitude_alias}]]`;

//-------------------------------------------------------------------
// GRATITUDE BUTTON AND CALLOUT
//-------------------------------------------------------------------
const gratitude_callout_title = `${call_start}[!gratitude]${space}Self Gratitude Journal${new_line}${call_start}${new_line}`;
const gratitude_callout_table = `${call_tbl_start}Daily Gratitude Journal${tbl_pipe}${gratitude_link}${call_tbl_end}${call_tbl_start}----------${tbl_pipe}----------${call_tbl_end}`;
const gratitude_callout = `${gratitude_callout_title}${gratitude_callout_table}${new_line}`;

//-------------------------------------------------------------------
// EARLY AFTERNOON DETACHMENT START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const early_noon_full_date = moment(`${date}T13:00`);
const early_noon_detach_duration = 5;
const early_noon_detach_start = moment(early_noon_full_date).format("HH:mm");
const early_noon_detach_end = moment(early_noon_full_date)
  .add(early_noon_detach_duration, "minutes")
  .format("HH:mm");
const early_noon_detach_reminder = moment(early_noon_full_date)
  .subtract(5, "minutes")
  .format("YYYY-MM-DD HH:mm");

const early_noon_detach_title = "Early Afternoon Detachment";

// Early afternoon detachment task checkbox
const early_noon_detach_task_checkbox = `${checkbox_task_tag}${early_noon_detach_title}_${type_value} [time_start:: ${early_noon_detach_start}]  [time_end:: ${early_noon_detach_end}]  [duration_est:: ${early_noon_detach_duration}] ⏰ ${early_noon_detach_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// EARLY AFTERNOON SELF GRATITUDE START AND END TIMES
//-------------------------------------------------------------------
const early_noon_grat_duration = 3;
const early_noon_grat_start = moment(`${date}T${early_noon_detach_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const early_noon_grat_end = moment(`${date}T${early_noon_grat_start}`)
  .add(early_noon_grat_duration, "minutes")
  .format("HH:mm");



const early_noon_grat_title = "Early Afternoon Self Gratitude";

// Early afternoon self gratitude task checkbox
const early_noon_grat_task_checkbox = `${checkbox_task_tag}${early_noon_grat_title}_${type_value} [time_start:: ${early_noon_grat_start}]  [time_end:: ${early_noon_grat_end}]  [duration_est:: ${early_noon_grat_duration}] ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// EARLY AFTERNOON MOVEMENT START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const early_noon_move_duration = 10;
const early_noon_move_start = moment(`${date}T${early_noon_grat_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const early_noon_move_end = moment(`${date}T${early_noon_move_start}`)
  .add(early_noon_move_duration, "minutes")
  .format("HH:mm");
const early_noon_move_reminder = moment(`${date}T${early_noon_move_start}`)
  .subtract(1, "minutes")
  .format("YYYY-MM-DD HH:mm");

const early_noon_move_title = "Early Afternoon Movement";

// Early afternoon movement task checkbox
const early_noon_move_task_checkbox = `${checkbox_task_tag}${early_noon_move_title}_${type_value} [time_start:: ${early_noon_move_start}]  [time_end:: ${early_noon_move_end}]  [duration_est:: ${early_noon_move_duration}] ⏰ ${early_noon_move_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

// Early afternoon movement routine callout
const early_noon_move_callout_title = `${call_start}[!exercise]${space}${early_noon_move_title}${space}Routine${new_line}${call_start}${new_line}`;
const core_strength_routine_callout = `${call_start}1.${space}Core Strength Routine${space}(One minute per exercise)${two_space}${new_line}${call_check_indent}Abdominal:${space}Figure Eights${new_line}${call_check_indent}Lower Back:${space}Left-Leg Glute Bridge${new_line}${call_check_indent}Abdominal:${space}Windshield Wipers${new_line}${call_check_indent}Lower Back:${space}Right-Leg Glute Bridge${new_line}${call_check_indent}Core:${space}Front Plank${new_line}${call_check_indent}Core:${space}Side Plank${new_line}`;
const core_stretch_routine_callout = `${call_start}2.${space}Core Strength Routine${space}(One minute per exercise)${two_space}${new_line}${call_check_indent}Ten Cat-Cows${new_line}${call_check_indent}Ten Bird-Dogs${new_line}${call_check_indent}Five Child pose to Cobra${new_line}`;
const early_noon_move_callout = `${early_noon_move_callout_title}${core_strength_routine_callout}${core_stretch_routine_callout}`;

//-------------------------------------------------------------------
// LATE AFTERNOON DETACHMENT START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const late_noon_full_date = moment(`${date}T16:00`);
const late_noon_detach_duration = 5;
const late_noon_detach_start = moment(late_noon_full_date).format("HH:mm");
const late_noon_detach_end = moment(late_noon_full_date)
  .add(late_noon_detach_duration, "minutes")
  .format("HH:mm");
const late_noon_detach_reminder = moment(late_noon_full_date)
  .subtract(5, "minutes")
  .format("YYYY-MM-DD HH:mm");

const late_noon_detach_title = "Early Afternoon Detachment";

// Late afternoon detachment task checkbox
const late_noon_detach_task_checkbox = `${checkbox_task_tag}${late_noon_detach_title}_${type_value} [time_start:: ${late_noon_detach_start}]  [time_end:: ${late_noon_detach_end}]  [duration_est:: ${late_noon_detach_duration}] ⏰ ${late_noon_detach_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// LATE AFTERNOON SELF GRATITUDE START AND END TIMES
//-------------------------------------------------------------------
const late_noon_grat_duration = 3;
const late_noon_grat_start = moment(`${date}T${late_noon_detach_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const late_noon_grat_end = moment(`${date}T${late_noon_grat_start}`)
  .add(late_noon_grat_duration, "minutes")
  .format("HH:mm");



const late_noon_grat_title = "Early Afternoon Self Gratitude";

// Late afternoon self gratitude task checkbox
const late_noon_grat_task_checkbox = `${checkbox_task_tag}${late_noon_grat_title}_${type_value} [time_start:: ${late_noon_grat_start}]  [time_end:: ${late_noon_grat_end}]  [duration_est:: ${late_noon_grat_duration}] ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// LATE AFTERNOON MOVEMENT START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const late_noon_move_duration = 10;
const late_noon_move_start = moment(`${date}T${late_noon_grat_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const late_noon_move_end = moment(`${date}T${late_noon_move_start}`)
  .add(late_noon_move_duration, "minutes")
  .format("HH:mm");
const late_noon_move_reminder = moment(`${date}T${late_noon_move_start}`)
  .subtract(1, "minutes")
  .format("YYYY-MM-DD HH:mm");

const late_noon_move_title = "Early Afternoon Movement";

// Late afternoon movement task checkbox
const late_noon_move_task_checkbox = `${checkbox_task_tag}${late_noon_move_title}_${type_value} [time_start:: ${late_noon_move_start}]  [time_end:: ${late_noon_move_end}]  [duration_est:: ${late_noon_move_duration}] ⏰ ${late_noon_move_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

// Late afternoon movement routine callout
const late_noon_move_callout_title = `${call_start}[!exercise]${space}${late_noon_move_title}${space}Routine${new_line}${call_start}${new_line}`;
const dynamic_stretch_callout = `${call_start}1.${space}Dynamic Stretching Routine${space}${two_space}${new_line}${call_check_indent}Five Deep Squat Adductor Push Outs${space}${new_line}${call_check_indent}Five Deep Squat Head-to-Floors${space}${new_line}${call_check_indent}Five Deep Squat Thoracic Rotating Arm Reaches${space}${new_line}${call_check_indent}Five Deep Squat Rotation to Lower Back Stretches${space}${new_line}${call_check_indent}Five Squat to Toe-Touches${space}${new_line}${call_check_indent}Five Downward-Dog to Lunge and Thoracic Rotating Arm Reaches${space}${new_line}${call_check_indent}Five Chest Stretches on All Fours${space}${new_line}${call_check_indent}Five Arm Reaches on All Fours for Adductors${space}${new_line}${call_check_indent}Five Back Bridge to Butterflies and Bicep Stretch${space}${new_line}${call_check_indent}Five Cossack Squats${space}${new_line}`;
const basic_strength_callout = `${call_start}2.${space}Basic Strength Routine${space}(One minute per exercise)${two_space}${new_line}${call_check_indent}Twenty Romanian Deadlifts${new_line}${call_check_indent}Seven Alternating Lunges per Side${new_line}${call_check_indent}Fifteen Pushups${new_line}${call_check_indent}Fifteen One-Arm Rows${new_line}`;
const late_noon_move_callout = `${late_noon_move_callout_title}${dynamic_stretch_callout}${basic_strength_callout}`;

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
/*         FRONTMATTER TITLES, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
const full_title_name = `${short_date} ${title}`;
const short_title_name = title.toLowerCase();
const short_title_value = title
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/'/g, "")
  .toLowerCase();
const full_title_value = `${short_date_value}_${short_title_value}`;

const alias_arr = yaml_li(title}"${ul_yaml}"${full_title_name}"${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}"${short_title_value}"${new_line}${ul_yaml}"${full_title_value);

const file_name = full_title_value;

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
# <%* tR += title %>

<%* tR += info %>

## <%* tR += type_name %>

### Detachment Practice

<%* tR += early_noon_detach_task_checkbox %>
<%* tR += late_noon_detach_task_checkbox %>

<%* tR += detach_callout %>

### Thank Yourself

<%* tR += early_noon_grat_task_checkbox %>
<%* tR += late_noon_grat_task_checkbox %>

<%* tR += gratitude_callout %>

### Movement

<%* tR += early_noon_move_task_checkbox %>

<%* tR += early_noon_move_callout %>

<%* tR += late_noon_move_task_checkbox %>

<%* tR += late_noon_move_callout %>

---

## Related

### Tasks and Events

### Notes

---

## Resources
