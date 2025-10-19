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
const child_task_info_callout = "42_child_task_info_callout";
const movement_early_afternoon_callout = "45_01_movement_early_afternoon_callout";
const movement_late_afternoon_callout = "45_02_movement_late_afternoon_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// CONTEXT NAME, VALUE, DIRECTORY, AND FILE CLASS
//-------------------------------------------------------------------
const context_name = "Habits and Rituals";
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();
const context_dir = habit_ritual_proj_dir;
const file_class = "task_child";

//-------------------------------------------------------------------
// DAILY HABIT AND RITUAL FILES
//-------------------------------------------------------------------
const habit_order = "01";
const habit_full_name = "Bi-Daily Habits";
const habit_full_value = habit_full_name
  .replaceAll(/[\s-]/g, "_")
  .toLowerCase();
const habit_name = habit_full_name.split(" ")[1];
const habit_type = context_name
  .split(" ")[0]
  .replaceAll(/s$/g, "")
  .toLowerCase();

const morn_rit_order = "02";
const morn_rit_full_name = "Daily Morning Rituals";
const morn_rit_full_value = morn_rit_full_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const morn_rit_name = morn_rit_full_name
  .split(" ")
  .splice(1, morn_rit_full_name.split(" ").length)
  .join(" ");
const morn_rit_value = morn_rit_name.replaceAll(/\s/g, "_").toLowerCase();
const morn_rit_type = morn_rit_value.slice(0, -1);

const work_start_order = "03";
const work_start_full_name = "Daily Workday Startup Rituals";
const work_start_full_value = work_start_full_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const work_start_name = work_start_full_name
  .split(" ")
  .splice(1, work_start_full_name.split(" ").length)
  .join(" ");
const work_start_value = work_start_name.replaceAll(/\s/g, "_").toLowerCase();
const work_start_type = work_start_value.slice(0, -1);

const work_shut_order = "04";
const work_shut_full_name = "Daily Workday Shutdown Rituals";
const work_shut_full_value = work_shut_full_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const work_shut_name = work_shut_full_name
  .split(" ")
  .splice(1, work_shut_full_name.split(" ").length)
  .join(" ");
const work_shut_value = work_shut_name.replaceAll(/\s/g, "_").toLowerCase();
const work_shut_type = work_shut_value.slice(0, -1);

const eve_rit_order = "05";
const eve_rit_full_name = "Daily Evening Rituals";
const eve_rit_full_value = eve_rit_full_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const eve_rit_name = eve_rit_full_name
  .split(" ")
  .splice(1, eve_rit_full_name.split(" ").length)
  .join(" ");
const eve_rit_value = eve_rit_name.replaceAll(/\s/g, "_").toLowerCase();
const eve_rit_type = eve_rit_value.slice(0, -1);

//-------------------------------------------------------------------
// SET THE WEEK
//-------------------------------------------------------------------
const date_obj_arr = [
  { key: "Current Week", value: "current" },
  { key: "Last Week", value: "last" },
  { key: "Next Week", value: "next" },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  "Which Week?"
);

const date_value = date_obj.value;

let full_date = "";

if (date_value.startsWith("current")) {
  full_date = moment();
} else if (date_value.startsWith("next")) {
  full_date = moment().add(1, "week");
} else {
  full_date = moment().subtract(1, "week");
}

//-------------------------------------------------------------------
// WEEKDAY CALENDAR VARIABLE
//-------------------------------------------------------------------
const sunday = moment(full_date).day(0).format("YYYY-MM-DD");
const monday = moment(full_date).day(1).format("YYYY-MM-DD");
const tuesday = moment(full_date).day(2).format("YYYY-MM-DD");
const wednesday = moment(full_date).day(3).format("YYYY-MM-DD");
const thursday = moment(full_date).day(4).format("YYYY-MM-DD");

// WEEKDAY DATES ARRAY
const weekday_arr = [sunday, monday, tuesday, wednesday, thursday];

//-------------------------------------------------------------------
// PILLAR FILES AND FULL NAMES
//-------------------------------------------------------------------
const mental_pillar_name = "Mental Health";
const mental_pillar_value = mental_pillar_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const mental_pillar_link = `[[${mental_pillar_value}|${mental_pillar_name}]]`;
const mental_pillar_value_link = yaml_li(mental_pillar_link);

const physical_pillar_name = "Physical Health";
const physical_pillar_value = physical_pillar_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const physical_pillar_link = `[[${physical_pillar_value}|${physical_pillar_name}]]`;
const physical_pillar_value_link = yaml_li(physical_pillar_link);

const pillar_value = `[${mental_pillar_value}, ${physical_pillar_value}]`;
const pillar_link = `${mental_pillar_link}, ${physical_pillar_link}`;
const pillar_value_link = `${mental_pillar_value_link}${physical_pillar_value_link}`;

//-------------------------------------------------------------------
// NULL VALUE, NAME, LINK, AND YAML LINK
//-------------------------------------------------------------------
const null_value = "null";
const null_name = "Null";
const null_link = `[[${null_value}|${null_name}]]`;
const null_yaml_li = yaml_li(null_link);

//-------------------------------------------------------------------
// DO/DUE DATE
//-------------------------------------------------------------------
const due_do_value = "do";

//-------------------------------------------------------------------
// TASK STATUS AND SYMBOL
//-------------------------------------------------------------------
const task_tag = "#task";
const status_symbol = " ";
const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}${space}`;

//-------------------------------------------------------------------
// EARLY AFTERNOON MOVEMENT ROUTINE CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${movement_early_afternoon_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const movement_early_afternoon_call = `${include_arr}${two_new_line}`;

//-------------------------------------------------------------------
// LATE AFTERNOON MOVEMENT ROUTINE CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${movement_late_afternoon_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const movement_late_afternoon_call = `${include_arr}${two_new_line}`;

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
const info_title = `${call_start}[!${context_value}]${space}${habit_name}${space}Details${two_space}${new_line}${call_start}${new_line}`;

const dv_date = `${backtick}dv:${space}this.file.frontmatter.date${backtick}`;
const info_date = `${call_start}Date${dv_colon}${dv_date}${two_space}`;

const info_body = `${child_task_info}${new_line}${info_date}`;

const info = `${info_title}${info_body}${two_new_line}${hr_line}${new_line}`;

//-------------------------------------------------------------------
// YAML FRONTMATTER
//-------------------------------------------------------------------
let yaml_title;
let yaml_alias;
let yaml_date;
let yaml_due_do = `due_do:${space}${due_do_value}${new_line}`;
let yaml_pillar = `pillar:${pillar_value_link}${new_line}`;
let yaml_context = `context:${space}${context_value}${new_line}`;
let yaml_goal = `goal:${space}null${new_line}`;
let yaml_project;
let yaml_parent_task;
let yaml_organization = `organization:${null_yaml_li}${new_line}`;
let yaml_contact = `contact:${null_yaml_li}${new_line}`;
let yaml_library = `library:${null_yaml_li}${new_line}`;
let yaml_type = `type:${space}${habit_type}${new_line}`;
let yaml_file_class = `file_class:${space}${file_class}${new_line}`;
let yaml_date_created = `date_created:${space}${date_created}${new_line}`;
let yaml_date_modified = `date_modified:${space}${date_modified}${new_line}`;
let yaml_tags = `tags:${new_line}`;

//-------------------------------------------------------------------
// TP.CREATE_NEW VARIABLES
//-------------------------------------------------------------------
let file_name;
let file_content;
let directory;

// LOOP THROUGH WEEKDAY DATES ARRAY
for (let i = 0; i < weekday_arr.length; i++) {
  // DATES FOR TITLES, ALIAS, AND FILE NAME
  date = weekday_arr[i];
  date_link = `[[${date}]]`;
  weekday = moment(`${date}T00:00`).format("dddd");
  full_date_time = moment(`${date}T00:00`);
  short_date = moment(full_date_time).format("YY-MM-DD");
  short_date_value = moment(full_date_time).format("YY_MM_DD");
  yaml_date = `date:${space}"${date_link}"${new_line}`;

  inline_date_data = `➕${space}${moment().format("YYYY-MM-DD")}${space}📅${space}${date}`;

  full_title_name = `${short_date} ${habit_full_name}`;
  short_title_name = habit_full_name.toLowerCase();
  short_title_value = habit_full_name
    .replaceAll(/[\s-]/g, "_")
    .replaceAll(/'/g, "")
    .toLowerCase();
  full_title_value = `${short_date_value}_${short_title_value}`;

  alias_arr = [
    habit_full_name,
    full_title_name,
    short_title_name,
    short_title_value,
    full_title_value,
  ];
  file_alias = "";
  for (let j = 0; j < alias_arr.length; j++) {
    alias = yaml_li(alias_arr[j]);
    file_alias += alias;
  }
  file_name = full_title_value;
  yaml_title = `title:${space}${file_name}${new_line}`;
  yaml_alias = `aliases:${file_alias}${new_line}`;

  // PROJECT FILE NAME, ALIAS, LINK, AND DIRECTORY
  year_month_short = moment(full_date_time).format("YYYY-MM");
  year_month_long = moment(full_date_time).format("MMMM [']YY");
  project_value = `${year_month_short}_${context_value}`;
  project_name = `${year_month_long} ${context_name}`;
  project_link = `[[${project_value}|${project_name}]]`;
  project_value_link = yaml_li(project_link);
  project_dir = `${context_dir}${project_value}/`;
  yaml_project = `project:${project_value_link}${new_line}`;

  // PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
  habit_ritual_order = "01";
  habit_ritual_name = "Habits";
  parent_task_value = `${year_month_short}_${habit_ritual_order}_${habit_ritual_name.toLowerCase()}`;
  parent_task_name = `${year_month_long} ${habit_ritual_name}`;
  parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
  parent_task_value_link = yaml_li(parent_task_link);
  parent_task_dir = `${project_dir}${parent_task_value}/`;
  yaml_parent_task = `parent_task:${parent_task_value_link}${new_line}`;

  // DETACHMENT FILE NAME, ALIAS, AND LINK
  detachment_alias = "Daily Detachment";
  detachment_alias_value = detachment_alias
    .replaceAll(/\s/g, "_")
    .toLowerCase();
  detachment_file_name = `${short_date_value}_${detachment_alias_value}`;
  detachment_link = `[[${detachment_file_name}\\|${detachment_alias}]]`;

  // DETACHMENT BUTTON AND CALLOUT
  detach_button = `${backtick}button-detachment-daily-preset${backtick}`;
  detach_callout_title = `${call_start}[!detachment]${space}${detachment_alias}${space}Journal${new_line}${call_start}${new_line}`;
  detach_callout_table = `${call_tbl_start}${detach_button}${tbl_pipe}${detachment_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
  detach_callout = `${detach_callout_title}${detach_callout_table}${new_line}`;

  // GRATITUDE FILE NAME, ALIAS, AND LINK
  gratitude_alias = "Daily Self Gratitude";
  gratitude_alias_value = gratitude_alias
    .replace(" Self ", "_")
    .toLowerCase();
  gratitude_file_name = `${short_date_value}_${gratitude_alias_value}`;
  gratitude_link = `[[${gratitude_file_name}\\|${gratitude_alias}]]`;

  // GRATITUDE BUTTON AND CALLOUT
  gratitude_callout_title = `${call_start}[!gratitude]${space}Self Gratitude Journal${new_line}${call_start}${new_line}`;
  gratitude_callout_table = `${call_tbl_start}Daily Gratitude Journal${tbl_pipe}${gratitude_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
  gratitude_callout = `${gratitude_callout_title}${gratitude_callout_table}${new_line}`;

  // EARLY AFTERNOON DETACHMENT START, END, AND REMINDER TIMES
  title = "Early Afternoon Detachment";
  noon_full_date = moment(`${date}T13:00`);
  duration = 5;
  time_start = moment(noon_full_date).format("HH:mm");
  time_end = moment(noon_full_date).add(duration, "minutes").format("HH:mm");
  reminder = moment(noon_full_date)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Early afternoon detachment task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  early_noon_detach = task_checkbox;

  // EARLY AFTERNOON SELF GRATITUDE START AND END TIMES
  title = "Early Afternoon Self Gratitude";
  duration = 3;
  time_start = moment(`${date}T${time_end}`).add(1, "minutes").format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");

  // Early afternoon self gratitude task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${inline_date_data}`;

  early_noon_gratitude = task_checkbox;

  // EARLY AFTERNOON MOVEMENT START, END, AND REMINDER TIMES
  title = "Early Afternoon Movement";
  duration = 10;
  time_start = moment(`${date}T${time_end}`).add(1, "minutes").format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");
  reminder = moment(`${date}T${time_start}`)
    .subtract(1, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Early afternoon movement task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  early_noon_movement = `${task_checkbox}${two_new_line}${movement_early_afternoon_call}`;

  // LATE AFTERNOON DETACHMENT START, END, AND REMINDER TIMES
  title = "Late Afternoon Detachment";
  noon_full_date = moment(`${date}T16:00`);
  duration = 5;
  time_start = moment(noon_full_date).format("HH:mm");
  time_end = moment(noon_full_date).add(duration, "minutes").format("HH:mm");
  reminder = moment(noon_full_date)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Late afternoon detachment task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  late_noon_detach = task_checkbox;

  // LATE AFTERNOON SELF GRATITUDE START AND END TIMES
  title = "Late Afternoon Self Gratitude";
  duration = 3;
  time_start = moment(`${date}T${time_end}`).add(1, "minutes").format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");

  // Late afternoon self gratitude task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${inline_date_data}`;

  late_noon_gratitude = task_checkbox;

  // LATE AFTERNOON MOVEMENT START, END, AND REMINDER TIMES
  title = "Late Afternoon Movement";
  duration = 10;
  time_start = moment(`${date}T${time_end}`).add(1, "minutes").format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");
  reminder = moment(`${date}T${time_start}`)
    .subtract(1, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Late afternoon movement task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  late_noon_movement = `${task_checkbox}${two_new_line}${movement_late_afternoon_call}`;

  // FIRST SRS REVIEW START, END, AND REMINDER TIMES
  title = "First SRS Review";
  day_srs_habit = `${weekday} ${title} Start Time?`;
  nl_time = await tp.user.nl_time(tp, day_srs_habit);

  duration = 10;
  time_start = moment(`${date}T${nl_time}`).format("HH:mm");
  time_end = moment(`${date}T${nl_time}`)
    .add(duration, "minutes")
    .format("HH:mm");
  reminder = moment(`${date}T${nl_time}`)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Early afternoon movement task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  srs_first = task_checkbox;

  // SECOND SRS REVIEW START, END, AND REMINDER TIMES
  title = "Second SRS Review";
  day_srs_habit = `${weekday} ${title}  Start Time`;
  nl_time = await tp.user.nl_time(tp, day_srs_habit);

  duration = 10;
  time_start = moment(`${date}T${nl_time}`).format("HH:mm");
  time_end = moment(`${date}T${nl_time}`)
    .add(duration, "minutes")
    .format("HH:mm");
  reminder = moment(`${date}T${nl_time}`)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Early afternoon movement task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${habit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  srs_second = task_checkbox;

  heading = "Spaced Repetition";
  srs = `${head_lvl(3)}${heading}${two_new_line}${srs_first}${new_line}${srs_second}${new_line}`;

  heading = "Detachment Practice";
  detachment = `${head_lvl(3)}${heading}${two_new_line}${early_noon_detach}${new_line}${late_noon_detach}${two_new_line}${detach_callout}`;

  heading = "Thank Yourself";
  gratitude = `${head_lvl(3)}${heading}${two_new_line}${early_noon_gratitude}${new_line}${late_noon_gratitude}${two_new_line}${gratitude_callout}`;

  heading = `${head_lvl(3)}Movement${two_new_line}`;
  sub_head_early = `${head_lvl(4)}Core Strength${two_new_line}`;
  sub_head_late = `${head_lvl(4)}Dynamic Flexibility${two_new_line}`;
  movement = `${heading}${sub_head_early}${early_noon_movement}${sub_head_late}${late_noon_movement}`;

  frontmatter = `${hr_line}${new_line}${yaml_title}${yaml_alias}${yaml_date}${yaml_due_do}${yaml_pillar}${yaml_context}${yaml_goal}${yaml_project}${yaml_parent_task}${yaml_organization}${yaml_contact}${yaml_library}${yaml_type}${yaml_file_class}${yaml_date_created}${yaml_date_modified}${yaml_tags}${hr_line}`;

  file_content = `${frontmatter}
${head_lvl(1)}${habit_full_name}${new_line}
${info}
${head_lvl(2)}${habit_name}${new_line}
${srs}
${detachment}
${gratitude}
${movement}
${hr_line}${new_line}
${head_lvl(2)}Related${new_line}
${head_lvl(3)}Tasks and Events${new_line}
${head_lvl(3)}Notes${new_line}
${hr_line}${new_line}
${head_lvl(2)}Resources
${new_line}`;

  directory = `${project_dir}${parent_task_value}`;
  // Create subdirectory file
  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(directory)
  );
}
%>
