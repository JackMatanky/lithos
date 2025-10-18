<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
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
const buttons_callout_task_event = "00_40_buttons_callout_task_event";
const child_task_info_callout = "42_child_task_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// DVJS FORMULA
//-------------------------------------------------------------------
const dvjs_task_files = `dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks`;
const dvjs_yaml_date_discard = `dv.equal(dv.date((dv.current().file.frontmatter.date).replace(/[^\\d-]/g, "")),${space}dv.date(t.due))${space}&&${space}t.status${space}=="-"`;
const dvjs_yaml_date_done = `dv.equal(dv.date((dv.current().file.frontmatter.date).replace(/[^\\d-]/g, "")),${space}dv.date(t.completion))`;
const dvjs_habit_ritual = `(t.text.includes("ritual")${space}||${space}t.text.includes("habit"))`;

heading = "Tasks Due Total";
const dvjs_done_total = `${heading}${colon}${space}${backtick}dvjs:${space}${dvjs_task_files}.filter((t)${space}=>${space}${dvjs_yaml_date_done}).length${backtick}`;
heading = "Tasks Discard Total";
const dvjs_discard_total = `${heading}${colon}${space}${backtick}dvjs:${space}${dvjs_task_files}.filter((t)${space}=>${space}${dvjs_yaml_date_discard}).length${backtick}`;
const dvjs_done_discard_total = `${ul}${dvjs_done_total}${tbl_pipe}${dvjs_discard_total}${new_line}`;

heading = "Tasks and Events";
const dvjs_done_task_event = `${heading}${colon}${space}${backtick}dvjs:${space}${dvjs_task_files}.filter((t)${space}=>${space}${dvjs_yaml_date_done}${space}&&${space}!${dvjs_habit_ritual}).length${backtick}`;
heading = "Discarded Tasks and Events";
const dvjs_discard_task_event = `${heading}${colon}${space}${backtick}dvjs:${space}${dvjs_task_files}.filter((t)${space}=>${space}${dvjs_yaml_date_discard}${space}&&${space}!${dvjs_habit_ritual}).length${backtick}`;
const dvjs_done_discard_task_event = `${ul}${dvjs_done_task_event}${tbl_pipe}${dvjs_discard_task_event}${new_line}`;

heading = "Habits and Rituals";
const dvjs_done_habit_ritual = `${heading}${colon}${space}${backtick}dvjs:${space}${dvjs_task_files}.filter((t)${space}=>${space}${dvjs_yaml_date_done}${space}&&${space}${dvjs_habit_ritual}).length${backtick}`;
heading = "Discarded Habits and Rituals";
const dvjs_discard_habit_ritual = `${heading}${colon}${space}${backtick}dvjs:${space}${dvjs_task_files}.filter((t)${space}=>${space}${dvjs_yaml_date_discard}${space}&&${space}${dvjs_habit_ritual}).length${backtick}`;
const dvjs_done_discard_habit_ritual = `${ul}${dvjs_done_task_event}${tbl_pipe}${dvjs_discard_task_event}`;

const dvjs_done_task_habit = `${dvjs_done_discard_total}${dvjs_done_discard_task_event}${dvjs_done_discard_habit_ritual}${two_new_line}`;

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
const task_tag = "#task";

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
// PILLARS, GOALS, ORGANIZATIONS, AND CONTACTS
//-------------------------------------------------------------------
const pillars_obj_arr = await tp.user.file_by_status({
  dir: pillars_dir,
  status: "active",
});
const goals = await tp.user.md_file_name(goals_dir);
const org_obj_arr = await tp.user.md_file_name_alias(organizations_dir);
const contact_obj_arr = await tp.user.md_file_name_alias(contacts_dir);

//-------------------------------------------------------------------
// TASK STATUS AND SYMBOL
//-------------------------------------------------------------------
const status_name = "To do";
const status_value = status_name.replaceAll(/\s/g, "_").toLowerCase();
const status_symbol = " ";
const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}${space}`;

//-------------------------------------------------------------------
// TASKS AND EVENTS BUTTONS
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${buttons_callout_task_event}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const task_event_buttons = `${include_arr}${two_new_line}`;

//-------------------------------------------------------------------
// TODAY'S AND TOMORROW'S HABITS AND RITUALS BUTTONS
//-------------------------------------------------------------------
const today_habit_button = `${backtick}button-habit-today${backtick}`;
const today_morn_rit_button = `${backtick}button-morn-rit-today${backtick}`;
const today_work_start_button = `${backtick}button-work-start-today${backtick}`;
const today_work_shut_button = `${backtick}button-work-shut-today${backtick}`;
const today_eve_rit_button = `${backtick}button-eve-rit-today${backtick}`;

const tomorrow_habit_button = `${backtick}button-habit-tomorrow${backtick}`;
const tomorrow_morn_rit_button = `${backtick}button-morn-rit-tomorrow${backtick}`;
const tomorrow_work_start_button = `${backtick}button-work-start-tomorrow${backtick}`;
const tomorrow_work_shut_button = `${backtick}button-work-shut-tomorrow${backtick}`;
const tomorrow_eve_rit_button = `${backtick}button-eve-rit-tomorrow${backtick}`;

//-------------------------------------------------------------------
// DAILY SCHEDULE BUTTON
//-------------------------------------------------------------------
comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;

const button_name = `name ✅Daily Schedule Review${new_line}`;
const button_type = `type append template${new_line}`;
const button_action = `action 111_40_dvmd_day_tasks_due${new_line}`;
const button_replace = `replace [139, 207]${new_line}`;
const button_color = `color yellow${new_line}`;

const daily_review_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}`;

//-------------------------------------------------------------------
// MENTAL HEALTH PILLAR FILE AND FULL NAME
//-------------------------------------------------------------------
const mental_health_pillar_name = "Mental Health";
const mental_health_pillar_value = mental_health_pillar_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const mental_health_pillar_link = `[[${mental_health_pillar_value}|${mental_health_pillar_name}]]`;
const mental_health_pillar_value_link = yaml_li(mental_health_pillar_link);

//-------------------------------------------------------------------
// NULL VALUE, NAME, LINK, AND YAML LINK
//-------------------------------------------------------------------
const null_value = "null";
const null_name = "Null";
const null_link = `[[${null_value}|${null_name}]]`;
const null_yaml_li = yaml_li(null_link);

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
const work_shut_info_title = `${call_start}[!${context_value}]${space}${work_shut_name}${space}Details${two_space}${new_line}${call_start}${new_line}`;
const eve_rit_info_title = `${call_start}[!${context_value}]${space}${eve_rit_name}${space}Details${two_space}${new_line}${call_start}${new_line}`;

const dv_date = `${backtick}dv:${space}this.file.frontmatter.date${backtick}`;
const info_date = `${call_start}Date${dv_colon}${dv_date}${two_space}`;

const info_body = `${child_task_info}${new_line}${info_date}`;

const work_shut_info = `${work_shut_info_title}${info_body}${two_new_line}${hr_line}${new_line}`;
const eve_rit_info = `${eve_rit_info_title}${info_body}${two_new_line}${hr_line}${new_line}`;

//-------------------------------------------------------------------
// YAML FRONTMATTER
//-------------------------------------------------------------------
let yaml_work_shut_title;
let yaml_eve_rit_title;
let yaml_work_shut_alias;
let yaml_eve_rit_alias;
let yaml_date;
let yaml_due_do = `due_do:${space}do${new_line}`;
let yaml_work_shut_pillar;
let yaml_eve_rit_pillar = `pillar:${mental_health_pillar_value_link}${new_line}`;
let yaml_context = `context:${space}${context_value}${new_line}`;
let yaml_work_shut_goal;
let yaml_eve_rit_goal = `goal:${space}null${new_line}`;
let yaml_project;
let yaml_work_shut_parent_task;
let yaml_eve_rit_parent_task;
let yaml_work_shut_organization;
let yaml_eve_rit_organization = `organization:${null_yaml_li}${new_line}`;
let yaml_work_shut_contact;
let yaml_eve_rit_contact = `contact:${null_yaml_li}${new_line}`;
let yaml_library = `library:${null_yaml_li}${new_line}`;
let yaml_work_shut_type = `type:${space}${work_shut_type}${new_line}`;
let yaml_eve_rit_type = `type:${space}${eve_rit_type}${new_line}`;
let yaml_file_class = `file_class:${space}${file_class}${new_line}`;
let yaml_date_created = `date_created:${space}${date_created}${new_line}`;
let yaml_date_modified = `date_modified:${space}${date_modified}${new_line}`;
let yaml_tags = `tags:${new_line}`;

//-------------------------------------------------------------------
// TP.CREATE_NEW VARIABLES
//-------------------------------------------------------------------
let work_shut_file_name;
let work_shut_file_content;
let work_shut_directory;
let eve_rit_file_name;
let eve_rit_file_content;
let eve_rit_directory;

// LOOP THROUGH WEEKDAY DATES ARRAY
for (let i = 0; i < weekday_arr.length; i++) {
  // DATES AND TIMES
  date = weekday_arr[i];
  date_link = `[[${date}]]`;
  weekday = moment(`${date}T00:00`).format("dddd");
  day_habit_rit = `${weekday} Workday Shutdown and Evening Rituals Start Time?`;
  time = await tp.user.nl_time(tp, day_habit_rit);
  full_date_time = moment(`${date}T${time}`);
  short_date = moment(full_date_time).format("YY-MM-DD");
  short_date_value = moment(full_date_time).format("YY_MM_DD");

  next_date = moment(full_date_time).add(1, "days").format("YYYY-MM-DD");
  next_date_short_value = moment(full_date_time)
    .add(1, "days")
    .format("YY_MM_DD");

  yaml_date = `date:${space}"${date_link}"${new_line}`;
  
  inline_date_data = `➕${space}${moment().format("YYYY-MM-DD")}${space}📅${space}${date}`;

  // DAILY HABIT AND RITUAL FILE LINKS
  habit_link = `[[${short_date_value}_${habit_full_value}\\|${habit_name}]]`;
  morn_rit_link = `[[${short_date_value}_${morn_rit_value}\\|${morn_rit_name}]]`;
  work_start_link = `[[${short_date_value}_${work_start_value}\\|${work_start_name}]]`;
  work_shut_link = `[[${short_date_value}_${work_shut_value}\\|${work_shut_name}]]`;
  eve_rit_link = `[[${short_date_value}_${eve_rit_value}\\|${eve_rit_name}]]`;

  // TOMORROW HABIT AND RITUAL FILE LINKS
  tomorrow_habit_link = `[[${next_date_short_value}_${habit_full_value}\\|${habit_name}]]`;
  tomorrow_morn_rit_link = `[[${next_date_short_value}_${morn_rit_value}\\|${morn_rit_name}]]`;
  tomorrow_work_start_link = `[[${next_date_short_value}_${work_start_value}\\|${work_start_name}]]`;
  tomorrow_work_shut_link = `[[${next_date_short_value}_${work_shut_value}\\|${work_shut_name}]]`;
  tomorrow_eve_rit_link = `[[${next_date_short_value}_${eve_rit_value}\\|${eve_rit_name}]]`;

  // WORKDAY SHUTDOWN RITUAL TITLES, ALIAS, AND FILE NAME
  work_shut_full_title_name = `${short_date} ${work_shut_full_name}`;
  work_shut_partial_title_name = `${short_date} ${work_shut_name}`;
  work_shut_short_title_name = work_shut_full_name.toLowerCase();
  work_shut_full_title_value = `${short_date_value}_${work_shut_full_value}`;
  work_shut_partial_title_value = `${short_date_value}_${work_shut_value}`;
  work_shut_short_title_value = work_shut_full_value;

  work_shut_alias_arr = [
    work_shut_full_name,
    work_shut_name,
    work_shut_full_title_name,
    work_shut_full_title_value,
    work_shut_partial_title_name,
    work_shut_partial_title_value,
    work_shut_short_title_name,
    work_shut_short_title_value,
  ];
  work_shut_alias = "";
  for (let j = 0; j < work_shut_alias_arr.length; j++) {
    alias = yaml_li(work_shut_alias_arr[j]);
    work_shut_alias += alias;
  }

  work_shut_file_name = `${short_date_value}_${work_shut_value}`;
  yaml_work_shut_title = `title:${space}${work_shut_file_name}${new_line}`;
  yaml_work_shut_alias = `aliases:${work_shut_alias}${new_line}`;

  // EVENING RITUAL TITLES, ALIAS, AND FILE NAME
  eve_rit_full_title_name = `${short_date} ${eve_rit_full_name}`;
  eve_rit_partial_title_name = `${short_date} ${eve_rit_name}`;
  eve_rit_short_title_name = eve_rit_full_name.toLowerCase();
  eve_rit_full_title_value = `${short_date_value}_${eve_rit_full_value}`;
  eve_rit_partial_title_value = `${short_date_value}_${eve_rit_value}`;
  eve_rit_short_title_value = eve_rit_full_value;

  eve_rit_alias_arr = [
    eve_rit_full_name,
    eve_rit_name,
    eve_rit_full_title_name,
    eve_rit_full_title_value,
    eve_rit_partial_title_name,
    eve_rit_partial_title_value,
    eve_rit_short_title_name,
    eve_rit_short_title_value,
  ];
  eve_rit_alias = "";
  for (let j = 0; j < eve_rit_alias_arr.length; j++) {
    alias = yaml_li(eve_rit_alias_arr[j]);
    eve_rit_alias += alias;
  }

  eve_rit_file_name = `${short_date_value}_${eve_rit_value}`;
  yaml_eve_rit_title = `title:${space}${eve_rit_file_name}${new_line}`;
  yaml_eve_rit_alias = `aliases:${eve_rit_alias}${new_line}`;

  // PROJECT FILE NAME, ALIAS, LINK, AND DIRECTORY
  year_month_short = moment(full_date_time).format("YYYY-MM");
  year_month_long = moment(full_date_time).format("MMMM [']YY");
  project_value = `${year_month_short}_${context_value}`;
  project_name = `${year_month_long} ${context_name}`;
  project_link = `[[${project_value}|${project_name}]]`;
  project_value_link = yaml_li(project_link);
  project_dir = `${context_dir}${project_value}/`;
  yaml_project = `project:${project_value_link}${new_line}`;

  // WORKDAY SHUTDOWN RITUAL PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
  work_shut_parent_task_value = `${year_month_short}_${work_shut_order}_${work_shut_value}`;
  work_shut_parent_task_name = `${year_month_long} ${work_shut_name}`;
  work_shut_parent_task_link = `[[${work_shut_parent_task_value}|${work_shut_parent_task_name}]]`;
  work_shut_parent_task_value_link = yaml_li(work_shut_parent_task_link);
  work_shut_parent_task_dir = `${project_dir}${work_shut_parent_task_value}/`;
  yaml_work_shut_parent_task = `parent_task:${work_shut_parent_task_value_link}${new_line}`;

  // EVENING RITUAL PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
  eve_rit_parent_task_value = `${year_month_short}_${eve_rit_order}_${eve_rit_value}`;
  eve_rit_parent_task_name = `${year_month_long} ${eve_rit_name}`;
  eve_rit_parent_task_link = `[[${eve_rit_parent_task_value}|${eve_rit_parent_task_name}]]`;
  eve_rit_parent_task_value_link = yaml_li(eve_rit_parent_task_link);
  eve_rit_parent_task_dir = `${project_dir}${eve_rit_parent_task_value}/`;
  yaml_eve_rit_parent_task = `parent_task:${eve_rit_parent_task_value_link}${new_line}`;

  // SET PILLAR
  pillar_obj = await tp.system.suggester(
    (item) => item.key,
    pillars_obj_arr,
    false,
    `Pillar for ${weekday} Workday Shutdown Ritual?`
  );

  pillar_value = pillar_obj.value;
  pillar_name = pillar_obj.key;
  pillar_link = `[[${pillar_value}|${pillar_name}]]`;
  pillar_value_link = yaml_li(pillar_link);
  yaml_work_shut_pillar = `pillar:${pillar_value_link}${new_line}`;

  // SET GOAL
  work_shut_goal = await tp.system.suggester(
    goals,
    goals,
    false,
    `Goal for ${weekday} Workday Shutdown Ritual?`
  );
  yaml_work_shut_goal = `goal:${space}${work_shut_goal}${new_line}`;

  // SET ORGANIZATION
  org_obj = await tp.system.suggester(
    (item) => item.key,
    org_obj_arr,
    false,
    `Organization for ${weekday} Workday Shutdown Ritual?`
  );
  organization_value = org_obj.value;
  organization_name = org_obj.key;

  if (organization_value.includes("_user_input")) {
    organization_name = await tp.system.prompt(
      `Organization for ${weekday} Workday Shutdown Ritual?`,
      "",
      false,
      false
    );
    organization_value = organization_name
      .replaceAll(/[,']/g, "")
      .replaceAll(/\s/g, "_")
      .replaceAll(/\//g, "-")
      .replaceAll(/&/g, "and")
      .toLowerCase();
  }
  organization_link = `[[${organization_value}|${organization_name}]]`;
  organization_value_link = yaml_li(organization_link);
  yaml_work_shut_organization = `organization:${organization_value_link}${new_line}`;

  // SET CONTACT
  contact_obj = await tp.system.suggester(
    (item) => item.key,
    contact_obj_arr,
    false,
    `Contact for ${weekday} Workday Shutdown Ritual?`
  );
  contact_value = contact_obj.value;
  contact_name = contact_obj.key;

  if (contact_value.includes("_user_input")) {
    contact_names = await tp.user.dirContactNames(tp);
    full_name = contact_names.full_name;
    last_first_name = contact_names.last_first_name;
    contact_name = full_name;
    contact_value = last_first_name
      .replaceAll(/,/g, "")
      .replaceAll(/[^\w]/g, "*")
      .toLowerCase();
  }
  contact_link = `[[${contact_value}|${contact_name}]]`;
  contact_value_link = yaml_li(contact_link);
  yaml_work_shut_contact = `contact:${contact_value_link}${new_line}`;

  // WORKDAY SHUTDOWN RITUAL CONTENT
  // DAILY PKM REVIEW START, END, AND REMINDER TIMES
  pkm_duration = 20;
  pkm_start = moment(full_date_time).format("HH:mm");
  pkm_end = moment(full_date_time).add(pkm_duration, "minutes").format("HH:mm");
  pkm_reminder = moment(full_date_time)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  pkm_heading = "### Knowledge Review";
  pkm_title = "Daily PKM Review";

  // Daily PKM review task checkbox
  pkm_task_checkbox = `${checkbox_task_tag}${pkm_title}_${work_shut_type} [time_start${dv_colon}${pkm_start}]  [time_end${dv_colon}${pkm_end}]  [duration_est${dv_colon}${pkm_duration}] ⏰ ${pkm_reminder} ${inline_date_data}`;

  pkm_subheading = "#### Daily PKM Review";
  daily_pkm_table = await tp.user.dv_pkm_type_status_dates({
    type: "",
    status: "eve_review",
    start_date: date,
    end_date: "",
    md: "false",
  });

  pkm = `${pkm_heading}${two_new_line}${pkm_task_checkbox}${two_new_line}${pkm_subheading}${two_new_line}${daily_pkm_table}`;

  // EMAIL PREVIEW START, END, AND REMINDER TIMES
  email_duration = 6;
  email_start = moment(`${date}T${pkm_end}`).add(1, "minutes").format("HH:mm");
  email_end = moment(`${date}T${email_start}`)
    .add(email_duration, "minutes")
    .format("HH:mm");
  email_reminder = moment(`${date}T${email_start}`)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  email_heading = "### Email Review";
  email_title = "Evening Email Review";

  // Daily email review task checkbox
  email_task_checkbox = `${checkbox_task_tag}${email_title}_${work_shut_type} [time_start${dv_colon}${email_start}]  [time_end${dv_colon}${email_end}]  [duration_est${dv_colon}${email_duration}] ⏰ ${email_reminder} ${inline_date_data}`;

  email = `${email_heading}${two_new_line}${email_task_checkbox}`;

  // WHATSAPP PREVIEW START AND END TIMES
  whatsapp_duration = 6;
  whatsapp_start = moment(`${date}T${email_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  whatsapp_end = moment(`${date}T${whatsapp_start}`)
    .add(whatsapp_duration, "minutes")
    .format("HH:mm");

  whatsapp_heading = "### WhatsApp Review";
  whatsapp_title = "Evening WhatsApp Review";

  // Daily email review task checkbox
  whatsapp_task_checkbox = `${checkbox_task_tag}${whatsapp_title}_${work_shut_type} [time_start${dv_colon}${whatsapp_start}]  [time_end${dv_colon}${whatsapp_end}]  [duration_est${dv_colon}${whatsapp_duration}] ${inline_date_data}`;

  whatsapp = `${whatsapp_heading}${two_new_line}${whatsapp_task_checkbox}`;

  // DAILY REVIEW START, END, AND REMINDER TIMES
  review_duration = 10;
  review_start = moment(`${date}T${whatsapp_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  review_end = moment(`${date}T${review_start}`)
    .add(review_duration, "minutes")
    .format("HH:mm");
  review_reminder = moment(`${date}T${whatsapp_end}`)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  review_heading = "### Review the Day";
  review_title = "Review Today's Schedule";

  // Daily preview task checkbox
  review_task_checkbox = `${checkbox_task_tag}${review_title}_${work_shut_type} [time_start${dv_colon}${review_start}]  [time_end${dv_colon}${review_end}]  [duration_est${dv_colon}${review_duration}] ⏰ ${review_reminder} ${inline_date_data}`;

  review_planned_heading = "#### Planned Schedule";
  review_task_count_heading = "#### Completed and Discarded Tasks";
  review_actual_heading = "#### Daily Schedule Review";

  morn_rit_section = "#" + "Daily Schedule";
  morn_rit_section_embed = `![[${short_date_value}_${morn_rit_value}${morn_rit_section}]]`;

  // DAILY SCHEDULE PREVIEW AND REVIEW DATAVIEW TABLE
  // STATUS_ACTION OPTIONS: `due`, `done`, `new`, `preview`, `review`
  review_task_table = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "review",
    start_date: date,
    end_date: "",
    md: "false",
  });

  review = `${review_heading}${two_new_line}${review_task_checkbox}${two_new_line}${review_task_count_heading}${two_new_line}${dvjs_done_task_habit}${daily_review_button}${review_actual_heading}${review_task_table}${two_new_line}${review_planned_heading}${two_new_line}${morn_rit_link}${two_new_line}${morn_rit_section_embed}${two_new_line}`;

  // EVENING RITUAL CALLOUT
  eve_rit_callout = `${call_start}[!${eve_rit_type}]${space}Today's${space}${eve_rit_name}${new_line}${call_start}${new_line}${call_tbl_start}${today_eve_rit_button}${tbl_pipe}${eve_rit_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

  // EVENING RITUAL CONTENT
  // TOMORROW'S HABITS AND RITUALS CALLOUT
  tomorrow_habit_rit_callout_title = `${call_start}[!${context_value}]${space}Tomorrow's${space}${context_name}${new_line}${call_start}${new_line}`;

  tomorrow_habit_rit_callout_buttons = `${call_tbl_start}${tomorrow_habit_button}${tbl_pipe}${tomorrow_morn_rit_button}${tbl_pipe}${tomorrow_work_start_button}${tbl_pipe}${tomorrow_work_shut_button}${tbl_pipe}${tomorrow_eve_rit_button}${call_tbl_end}${new_line}`;
  tomorrow_habit_rit_callout_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
  tomorrow_habit_rit_callout_links = `${call_tbl_start}${tomorrow_habit_link}${tbl_pipe}${tomorrow_morn_rit_link}${tbl_pipe}${tomorrow_work_start_link}${tbl_pipe}${tomorrow_work_shut_link}${tbl_pipe}${tomorrow_eve_rit_link}${call_tbl_end}`;
  tomorrow_habit_rit_callout_body = `${tomorrow_habit_rit_callout_buttons}${tomorrow_habit_rit_callout_div}${tomorrow_habit_rit_callout_links}`;

  tomorrow_habit_rit_callout = `${tomorrow_habit_rit_callout_title}${tomorrow_habit_rit_callout_body}`;

  // DAILY PREVIEW START, END, AND REMINDER TIMES
  preview_duration = 10;
  preview_start = moment(`${date}T${review_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  preview_end = moment(`${date}T${preview_start}`)
    .add(preview_duration, "minutes")
    .format("HH:mm");

  preview_heading = "### Preview Tomorrow's Schedule";
  preview_title = "Preview Tomorrow's Schedule";

  // Daily preview task checkbox
  preview_task_checkbox = `${checkbox_task_tag}${preview_title}_${eve_rit_type} [time_start${dv_colon}${preview_start}]  [time_end${dv_colon}${preview_end}]  [duration_est${dv_colon}${preview_duration}] ${inline_date_data}`;

  preview_subheading = "#### Tomorrow's Schedule";
  comment = `${cmnt_html_start}Schedule the most important and longest tasks in descending order${cmnt_html_end}`;
  // DAILY SCHEDULE PREVIEW AND REVIEW DATAVIEW TABLE
  // STATUS_ACTION OPTIONS: "due", "done", "new", "preview", "review"
  preview_task_table = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "eve_preview",
    start_date: next_date,
    end_date: "",
    md: "false",
  });

  preview = `${preview_heading}${two_new_line}${preview_task_checkbox}${two_new_line}${preview_subheading}${two_new_line}${comment}${two_new_line}${task_event_buttons}${two_new_line}${preview_task_table}`;

  // EVENING MEDITATION START, END, AND REMINDER TIMES
  meditation_duration = 10;
  meditation_start = moment(`${date}T${preview_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  meditation_end = moment(`${date}T${meditation_start}`)
    .add(meditation_duration, "minutes")
    .format("HH:mm");
  meditation_reminder = moment(`${date}T${meditation_start}`)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  meditation_heading = "### Evening Meditation";
  meditation_title = "Evening Meditation";

  // Meditation task
  meditation_task_checkbox = `${checkbox_task_tag}${meditation_title}_${eve_rit_type} [time_start${dv_colon}${meditation_start}]  [time_end${dv_colon}${meditation_end}]  [duration_est${dv_colon}${meditation_duration}] ⏰ ${meditation_reminder} ${inline_date_data}`;

  meditation_bell_heading = "#### Mindfulness Bell Meditation";
  meditation_bell_embed = "![[1_Five Minute Mindfulness Bell Meditation.mp3]]";

  meditation_positive_mind_heading = "#### Positive Mind Meditation";
  meditation_positive_mind_embed =
    "![[morn_1_Positive Mind in 5 Minutes Meditation_Jason Stephenson.mp3]]";

  meditation = `${meditation_heading}${two_new_line}${meditation_task_checkbox}${two_new_line}${meditation_bell_heading}${two_new_line}${meditation_bell_embed}${two_new_line}${meditation_positive_mind_heading}${two_new_line}${meditation_positive_mind_embed}`;

  // WORKDAY SHUTDOWN RITUAL CALLOUT
  work_shut_callout = `${call_start}[!${work_shut_type}]${space}Today's${space}${work_shut_name}${new_line}${call_start}${new_line}${call_tbl_start}${today_work_shut_button}${tbl_pipe}${work_shut_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

  work_shut_frontmatter = `${hr_line}${new_line}${yaml_work_shut_title}${yaml_work_shut_alias}${yaml_date}${yaml_due_do}${yaml_work_shut_pillar}${yaml_context}${yaml_work_shut_goal}${yaml_project}${yaml_work_shut_parent_task}${yaml_work_shut_organization}${yaml_work_shut_contact}${yaml_library}${yaml_work_shut_type}${yaml_file_class}${yaml_date_created}${yaml_date_modified}${yaml_tags}${hr_line}`;

  work_shut_file_content = `${work_shut_frontmatter}
${head_lvl(1)}${work_shut_name}${new_line}
${work_shut_info}
${head_lvl(2)}Rituals${new_line}
${pkm}${new_line}
${email}${new_line}
${whatsapp}${new_line}
${review}${new_line}
${hr_line}${new_line}
${eve_rit_callout}${new_line}
${hr_line}${new_line}
${head_lvl(2)}Related${new_line}
${head_lvl(3)}Tasks and Events${new_line}
${head_lvl(3)}Notes${new_line}
${hr_line}${new_line}
${head_lvl(2)}Resources${new_line}`;

  work_shut_directory = `${project_dir}${work_shut_parent_task_value}`;
  // Create subdirectory file
  await tp.file.create_new(
    work_shut_file_content,
    work_shut_file_name,
    false,
    app.vault.getAbstractFileByPath(work_shut_directory)
  );

  eve_rit_frontmatter = `${hr_line}${new_line}${yaml_eve_rit_title}${yaml_eve_rit_alias}${yaml_date}${yaml_due_do}${yaml_eve_rit_pillar}${yaml_context}${yaml_eve_rit_goal}${yaml_project}${yaml_eve_rit_parent_task}${yaml_eve_rit_organization}${yaml_eve_rit_contact}${yaml_library}${yaml_eve_rit_type}${yaml_file_class}${yaml_date_created}${yaml_date_modified}${yaml_tags}${hr_line}`;

  eve_rit_file_content = `${eve_rit_frontmatter}
${head_lvl(1)}${eve_rit_name}${new_line}
${eve_rit_info}
${head_lvl(2)}Rituals${new_line}
${tomorrow_habit_rit_callout}${new_line}
${preview}${new_line}
${meditation}${new_line}
${hr_line}${new_line}
${work_shut_callout}${new_line}
${hr_line}${new_line}
${head_lvl(2)}Related${new_line}
${head_lvl(3)}Tasks and Events${new_line}
${head_lvl(3)}Notes${new_line}
${hr_line}${new_line}
${head_lvl(2)}Resources${new_line}`;

  eve_rit_directory = `${project_dir}${eve_rit_parent_task_value}`;
  // Create subdirectory file
  await tp.file.create_new(
    eve_rit_file_content,
    eve_rit_file_name,
    false,
    app.vault.getAbstractFileByPath(eve_rit_directory)
  );
}
%>