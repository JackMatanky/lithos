<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
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

const call_tbl_div = (int) =>
  call_tbl_start + Array(int).fill(tbl_cent).join(tbl_pipe) + call_tbl_end;

//-------------------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const child_task_info_callout = "42_child_task_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// NULL VALUE, NAME, LINK, AND YAML LINK
//-------------------------------------------------------------------
const null_value = "null";
const null_name = "Null";
const null_link = `[[${null_value}|${null_name}]]`;
const null_yaml_li = yaml_li(null_link);

//-------------------------------------------------------------------
// TASKS AND EVENTS BUTTONS
//-------------------------------------------------------------------
const task_event_buttons = (await tp.user.include_file("00_40_buttons_callout_task_event")) + two_new_line;

//-------------------------------------------------------------------
// TODAY'S HABITS AND RITUALS BUTTONS
//-------------------------------------------------------------------
const today_habit_button = `${backtick}button-habit-today${backtick}`;
const today_morn_rit_button = `${backtick}button-morn-rit-today${backtick}`;
const today_work_start_button = `${backtick}button-work-start-today${backtick}`;
const today_work_shut_button = `${backtick}button-work-shut-today${backtick}`;
const today_eve_rit_button = `${backtick}button-eve-rit-today${backtick}`;

//-------------------------------------------------------------------
// DAILY JOURNALS BUTTON
//-------------------------------------------------------------------
const daily_journal_button =
  [
    `${three_backtick}button`,
    "name 🕯️Daily Journals",
    "type note(Untitled, split) template",
    "action 90_01_daily_journals_preset",
    "color purple",
    `${three_backtick}`,
  ].join(new_line) + two_new_line;

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
// REFLECTION AND GRATITUDE JOURNAL VARIABLES
//-------------------------------------------------------------------
const reflection_button = `${backtick}button-reflection-daily-preset${backtick}`;
const reflection_alias = "Daily Reflection";
const reflection_alias_value = reflection_alias
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const reflection_callout_title = `${call_start}[!reflection]${space}${reflection_alias}${space}Journal${new_line}${call_start}${new_line}`;

const gratitude_button = `${backtick}button-gratitude-daily-preset${backtick}`;
const gratitude_alias = "Daily Gratitude";
const gratitude_alias_value = gratitude_alias
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const gratitude_callout_title = `${call_start}[!gratitude]${space}${gratitude_alias}${space}Journal${new_line}${call_start}${new_line}`;
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
const task_tag = "#task";
const status_symbol = " ";
const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}${space}`;

//-------------------------------------------------------------------
// MENTAL HEALTH PILLAR FILE AND FULL NAME
//-------------------------------------------------------------------
const mental_health_pillar_name = "Mental Health";
const mental_health_pillar_value = mental_health_pillar_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const mental_health_pillar_link = `[[${mental_health_pillar_value}|${mental_health_pillar_name}]]`;
const mental_health_pillar_value_link = yaml_li(mental_health_pillar_link);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const child_task_info = await tp.user.include_file("42_child_task_info_callout");
const morn_rit_info_title = `${call_start}[!${morn_rit_type}]${space}${morn_rit_name}${space}Details${two_space}${new_line}${call_start}${new_line}`;
const work_start_info_title = `${call_start}[!${work_start_type}]${space}${work_start_name}${space}Details${two_space}${new_line}${call_start}${new_line}`;

const dv_date = `${backtick}dv:${space}this.file.frontmatter.date${backtick}`;
const info_date = `${call_start}Date${dv_colon}${dv_date}${two_space}`;

const info_body = `${child_task_info}${new_line}${info_date}`;

const morn_rit_info = `${morn_rit_info_title}${info_body}${two_new_line}${hr_line}${new_line}`;
const work_start_info = `${work_start_info_title}${info_body}${two_new_line}${hr_line}${new_line}`;

//-------------------------------------------------------------------
// CALLOUT TITLES
//-------------------------------------------------------------------
const today_habit_rit_callout_title = `${call_start}[!${context_value}]${space}Today's${space}${context_name}${new_line}${call_start}${new_line}`;

const morn_rit_callout_title = `${call_start}[!${morn_rit_type}]${space}Today's${space}${morn_rit_name}${new_line}${call_start}${new_line}`;

//-------------------------------------------------------------------
// TASK OBJECT ARRAYS
//-------------------------------------------------------------------
const tasks_obj_arr = [
  {
    hab_rit: "morning",
    head_level: 3,
    head: "Morning Self Affirmations",
    task: "Self Affirmations",
    duration: 3,
    remind: 5,
  },
  {
    hab_rit: "morning",
    head_level: 3,
    head: "Daily Schedule Preview",
    task: "Preview Today's Schedule",
    duration: 10,
    remind: null,
  },
  {
    hab_rit: "morning",
    head_level: 3,
    head: "Morning Journals",
    task: null,
    duration: null,
    remind: null,
  },
  {
    hab_rit: "morning",
    head_level: 4,
    head: "Recount Yesterday",
    task: "Daily Reflection",
    duration: 20,
    remind: null,
  },
  {
    hab_rit: "morning",
    head_level: 4,
    head: "Give Thanks",
    task: "Daily Gratitude",
    duration: 3,
    remind: null,
  },
  {
    hab_rit: "work_start",
    head_level: 3,
    head: "Email Review",
    task: "Morning Email Review",
    duration: 6,
    remind: 5,
  },
  {
    hab_rit: "work_start",
    head_level: 3,
    head: "WhatsApp Review",
    task: "Morning WhatsApp Review",
    duration: 6,
    remind: null,
  },
  {
    hab_rit: "morning",
    head_level: 3,
    head: "Morning Meditation",
    task: "Morning Meditation",
    duration: 10,
    remind: 5,
  },
];

//-------------------------------------------------------------------
// YAML FRONTMATTER
//-------------------------------------------------------------------
let yaml_morn_rit_title;
let yaml_work_start_title;
let yaml_morn_rit_alias;
let yaml_work_start_alias;
let yaml_date;
let yaml_due_do = `due_do:${space}do${new_line}`;
let yaml_morn_rit_pillar = `pillar:${mental_health_pillar_value_link}${new_line}`;
let yaml_work_start_pillar;
let yaml_context = `context:${space}${context_value}${new_line}`;
let yaml_morn_rit_goal;
let yaml_work_start_goal;
let yaml_project;
let yaml_morn_rit_parent_task;
let yaml_work_start_parent_task;
let yaml_morn_rit_organization = `organization:${null_yaml_li}${new_line}`;
let yaml_work_start_organization;
let yaml_morn_rit_contact = `contact:${null_yaml_li}${new_line}`;
let yaml_work_start_contact;
let yaml_library = `library:${null_yaml_li}${new_line}`;
let yaml_morn_rit_type = `type:${space}${morn_rit_type}${new_line}`;
let yaml_work_start_type = `type:${space}${work_start_type}${new_line}`;
let yaml_file_class = `file_class:${space}${file_class}${new_line}`;
let yaml_date_created = `date_created:${space}${date_created}${new_line}`;
let yaml_date_modified = `date_modified:${space}${date_modified}${new_line}`;
let yaml_tags = `tags:${new_line}`;

//-------------------------------------------------------------------
// TP.CREATE_NEW VARIABLES
//-------------------------------------------------------------------
let morn_rit_file_name;
let morn_rit_file_content;
let morn_rit_directory;
let work_start_file_name;
let work_start_file_content;
let work_start_directory;

// LOOP THROUGH WEEKDAY DATES ARRAY
for (let i = 0; i < weekday_arr.length; i++) {
  // DATES AND TIMES
  date = weekday_arr[i];
  date_link = `[[${date}]]`;
  weekday = moment(`${date}T00:00`).format("dddd");
  day_habit_rit = `${weekday} Morning and Workday Startup Ritual Start Time?`;
  time = await tp.user.nl_time(tp, day_habit_rit);
  full_date_time = moment(`${date}T${time}`);
  short_date = moment(full_date_time).format("YY-MM-DD");
  short_date_value = moment(full_date_time).format("YY_MM_DD");
  yaml_date = `date:${space}"${date_link}"${new_line}`;

  inline_date_data = `➕${space}${moment().format("YYYY-MM-DD")}${space}📅${space}${date}`;

  // DAILY HABIT AND RITUAL FILE LINKS
  habit_link = `[[${short_date_value}_${habit_full_value}\\|${habit_name}]]`;
  morn_rit_link = `[[${short_date_value}_${morn_rit_value}\\|${morn_rit_name}]]`;
  work_start_link = `[[${short_date_value}_${work_start_value}\\|${work_start_name}]]`;
  work_shut_link = `[[${short_date_value}_${work_shut_value}\\|${work_shut_name}]]`;
  eve_rit_link = `[[${short_date_value}_${eve_rit_value}\\|${eve_rit_name}]]`;

  // MORNING RITUAL TITLES, ALIAS, AND FILE NAME
  morn_rit_full_title_name = `${short_date} ${morn_rit_full_name}`;
  morn_rit_partial_title_name = `${short_date} ${morn_rit_name}`;
  morn_rit_short_title_name = morn_rit_full_name.toLowerCase();
  morn_rit_full_title_value = `${short_date_value}_${morn_rit_full_value}`;
  morn_rit_partial_title_value = `${short_date_value}_${morn_rit_value}`;
  morn_rit_short_title_value = morn_rit_full_value;

  morn_rit_alias_arr = [
    morn_rit_full_name,
    morn_rit_name,
    morn_rit_full_title_name,
    morn_rit_full_title_value,
    morn_rit_partial_title_name,
    morn_rit_partial_title_value,
    morn_rit_short_title_name,
    morn_rit_short_title_value,
  ];
  morn_rit_alias = "";
  for (let j = 0; j < morn_rit_alias_arr.length; j++) {
    alias = `${new_line}${ul_yaml}${morn_rit_alias_arr[j]}`;
    morn_rit_alias += alias;
  }

  morn_rit_file_name = `${short_date_value}_${morn_rit_value}`;
  yaml_morn_rit_title = `title:${space}${morn_rit_file_name}${new_line}`;
  yaml_morn_rit_alias = `aliases:${morn_rit_alias}${new_line}`;

  // WORKDAY STARTUP RITUAL TITLES, ALIAS, AND FILE NAME
  work_start_full_title_name = `${short_date} ${work_start_full_name}`;
  work_start_partial_title_name = `${short_date} ${work_start_name}`;
  work_start_short_title_name = work_start_full_name.toLowerCase();
  work_start_full_title_value = `${short_date_value}_${work_start_full_value}`;
  work_start_partial_title_value = `${short_date_value}_${work_start_value}`;
  work_start_short_title_value = work_start_full_value;

  work_start_alias_arr = [
    work_start_full_name,
    work_start_name,
    work_start_full_title_name,
    work_start_full_title_value,
    work_start_partial_title_name,
    work_start_partial_title_value,
    work_start_short_title_name,
    work_start_short_title_value,
  ];
  work_start_alias = "";
  for (let k = 0; k < work_start_alias_arr.length; k++) {
    alias = `${new_line}${ul_yaml}${work_start_alias_arr[k]}`;
    work_start_alias += alias;
  }

  work_start_file_name = `${short_date_value}_${work_start_value}`;
  yaml_work_start_title = `title:${space}${work_start_file_name}${new_line}`;
  yaml_work_start_alias = `aliases:${work_start_alias}${new_line}`;

  // PROJECT FILE NAME, ALIAS, LINK, AND DIRECTORY
  year_month_short = moment(full_date_time).format("YYYY-MM");
  year_month_long = moment(full_date_time).format("MMMM [']YY");
  project_value = `${year_month_short}_${context_value}`;
  project_name = `${year_month_long} ${context_name}`;
  project_link = `[[${project_value}|${project_name}]]`;
  project_value_link = yaml_li(project_link);
  project_dir = `${context_dir}${project_value}/`;
  yaml_project = `project:${project_value_link}${new_line}`;

  // MORNING RITUAL PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
  morn_rit_parent_task_value = `${year_month_short}_${morn_rit_order}_${morn_rit_value}`;
  morn_rit_parent_task_name = `${year_month_long} ${morn_rit_name}`;
  morn_rit_parent_task_link = `[[${morn_rit_parent_task_value}|${morn_rit_parent_task_name}]]`;
  morn_rit_parent_task_value_link = yaml_li(morn_rit_parent_task_link);
  morn_rit_parent_task_dir = `${project_dir}${morn_rit_parent_task_value}/`;
  yaml_morn_rit_parent_task = `parent_task:${morn_rit_parent_task_value_link}${new_line}`;

  // WORKDAY STARTUP RITUAL PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
  work_start_parent_task_value = `${year_month_short}_${work_start_order}_${work_start_value}`;
  work_start_parent_task_name = `${year_month_long} ${work_start_name}`;
  work_start_parent_task_link = `[[${work_start_parent_task_value}|${work_start_parent_task_name}]]`;
  work_start_parent_task_value_link = yaml_li(work_start_parent_task_link);
  work_start_parent_task_dir = `${project_dir}${work_start_parent_task_value}/`;
  yaml_work_start_parent_task = `parent_task:${work_start_parent_task_value_link}${new_line}`;

  // SET GOAL
  morn_rit_goal = await tp.system.suggester(
    goals,
    goals,
    false,
    `Goal for ${weekday} Morning Ritual?`
  );
  yaml_morn_rit_goal = `goal:${space}${morn_rit_goal}${new_line}`;

  // SET PILLAR
  pillar_obj = await tp.system.suggester(
    (item) => item.key,
    pillars_obj_arr,
    false,
    `Pillar for ${weekday} Workday Startup Ritual?`
  );

  pillar_value = pillar_obj.value;
  pillar_name = pillar_obj.key;
  pillar_link = `[[${pillar_value}|${pillar_name}]]`;
  pillar_value_link = yaml_li(pillar_link);
  yaml_work_start_pillar = `pillar:${pillar_value_link}${new_line}`;

  // SET GOAL
  work_start_goal = await tp.system.suggester(
    goals,
    goals,
    false,
    `Goal for ${weekday} Workday Startup Ritual?`
  );
  yaml_work_start_goal = `goal:${space}${work_start_goal}${new_line}`;

  // SET ORGANIZATION
  org_obj = await tp.system.suggester(
    (item) => item.key,
    org_obj_arr,
    false,
    `Organization for ${weekday} Workday Startup Ritual?`
  );
  organization_value = org_obj.value;
  organization_name = org_obj.key;

  if (organization_value.includes("_user_input")) {
    organization_name = await tp.system.prompt(
      `Organization for ${weekday} Workday Startup Ritual?`,
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
  yaml_work_start_organization = `organization:${organization_value_link}${new_line}`;

  // SET CONTACT
  contact_obj = await tp.system.suggester(
    (item) => item.key,
    contact_obj_arr,
    false,
    `Contact for ${weekday} Workday Startup Ritual?`
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
  yaml_work_start_contact = `contact:${contact_value_link}${new_line}`;

  // MORNING RITUAL CALLOUT
  morn_rit_callout_table = call_tbl_start + today_morn_rit_button + tbl_pipe + morn_rit_link + call_tbl_end + new_line + call_tbl_div(2);
  morn_rit_callout = `${morn_rit_callout_title}${morn_rit_callout_table}${two_new_line}${hr_line}${new_line}`;

  // TODAY'S HABITS AND RITUALS CALLOUT
  today_habit_rit_callout_buttons = call_tbl_start + [today_habit_button, today_morn_rit_button, today_work_start_button, today_work_shut_button, today_eve_rit_button].join(tbl_pipe) + call_tbl_end;
  today_habit_rit_callout_links = call_tbl_start + [habit_link, morn_rit_link, work_start_link, work_shut_link, eve_rit_link].join(tbl_pipe) + call_tbl_end;
  today_habit_rit_callout_body = [today_habit_rit_callout_buttons, call_tbl_div(5), today_habit_rit_callout_links].join(new_line);

  today_habit_rit_callout = `${today_habit_rit_callout_title}${today_habit_rit_callout_body}`;

  // REFLECTION FILE NAME, LINK, AND CALLOUT
  const reflection_file_name = `${short_date_value}_${reflection_alias_value}`;
  const reflection_link = `[[${reflection_file_name}\\|${reflection_alias}]]`;
  const reflection_callout_table = `${call_tbl_start}${reflection_button}${tbl_pipe}${reflection_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
  const reflection_callout = `${reflection_callout_title}${reflection_callout_table}${new_line}`;

  // GRATITUDE FILE NAME, LINK, AND CALLOUT
  const gratitude_file_name = `${short_date_value}_${gratitude_alias_value}`;
  const gratitude_link = `[[${gratitude_file_name}\\|${gratitude_alias}]]`;
  const gratitude_callout_table = `${call_tbl_start}${gratitude_button}${tbl_pipe}${gratitude_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
  const gratitude_callout = `${gratitude_callout_title}${gratitude_callout_table}${new_line}`;

  // SELF AFFIRMATION START, END, AND REMINDER TIMES
  title = "Self Affirmations";
  duration = 3;
  time_start = moment(full_date_time).format("HH:mm");
  time_end = moment(full_date_time)
    .add(duration, "minutes")
    .format("HH:mm");
  reminder = moment(full_date_time)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Morning self Affirmation task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${morn_rit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  heading = "### Morning Self Affirmations";
  affirm = `${heading}${two_new_line}${task_checkbox}${two_new_line}`;

  // DAILY PREVIEW START, END, AND REMINDER TIMES
  title = "Preview Today's Schedule";
  duration = 10;
  time_start = moment(`${date}T${time_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");

  // Daily preview task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${morn_rit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${inline_date_data}`;

  heading = "### Preview Today's Schedule";
  comment = "Schedule the most important and longest tasks in descending order";
  const day_task_event_link = `[[${date}_task_event#Due Today|Task Schedule]]`;
  const daily_schedule_callout = `${call_start}[!task_preview]${space}Daily Schedule${new_line}${call_start}${new_line}${call_start}${day_task_event_link}${new_line}${call_start}${new_line}${call_start}${comment}`;

  preview = `${heading}${two_new_line}${task_checkbox}${two_new_line}${daily_schedule_callout}${two_new_line}`;

  // DAILY REFLECTION START AND END TIMES
  title = "Daily Reflection";
  duration = 20;
  time_start = moment(`${date}T${time_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");

  task_checkbox_title = `${checkbox_task_tag}${title}_${morn_rit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${inline_date_data}`;

  heading = `${head_lvl(3)}Morning Journals${two_new_line}${daily_journal_button}`;
  subheading = `${head_lvl(4)}Recount Yesterday${two_new_line}`;
  reflection = `${heading}${subheading}${task_checkbox}${two_new_line}${reflection_callout}${two_new_line}`;

  // DAILY GRATITUDE START AND END TIMES
  title = "Daily Gratitude";
  duration = 3;
  time_start = moment(`${date}T${time_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");

  task_checkbox_title = `${checkbox_task_tag}${title}_${morn_rit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${inline_date_data}`;

  heading = `${head_lvl(4)}Give Thanks${two_new_line}`;
  gratitude = `${heading}${task_checkbox}${two_new_line}${gratitude_callout}${two_new_line}`;

  // WORKDAY STARTUP RITUAL CALLOUT
  work_start_callout = `${call_start}[!${work_start_type}]${space}Today's${space}${work_start_name}${new_line}${call_start}${new_line}${call_tbl_start}${today_work_start_button}${tbl_pipe}${work_start_link}${call_tbl_end}${new_line}${call_tbl_start}${tbl_left}${tbl_pipe}${tbl_cent}${call_tbl_end}${two_new_line}${hr_line}${new_line}`;

  // EMAIL PREVIEW START, END, AND REMINDER TIMES
  title = "Morning Email Review";
  duration = 6;
  time_start = moment(`${date}T${time_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");
  reminder = moment(`${date}T${time_start}`)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Daily email review task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${work_start_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  heading = "### Email Review";
  email = `${heading}${two_new_line}${task_checkbox}${two_new_line}`;

  // WHATSAPP PREVIEW START AND END TIMES
  title = "Morning WhatsApp Review";
  duration = 6;
  time_start = moment(`${date}T${time_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");

  // Daily email review task checkbox
  task_checkbox_title = `${checkbox_task_tag}${title}_${work_start_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${inline_date_data}`;

  heading = "### WhatsApp Review";
  whatsapp = `${heading}${two_new_line}${task_checkbox}${two_new_line}`;

  // FIVE-MIN MEDITATION START, END, AND REMINDER TIMES
  title = "Morning Meditation";
  duration = 10;
  time_start = moment(`${date}T${time_end}`)
    .add(1, "minutes")
    .format("HH:mm");
  time_end = moment(`${date}T${time_start}`)
    .add(duration, "minutes")
    .format("HH:mm");
  reminder = moment(`${date}T${time_start}`)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");

  // Meditation task
  task_checkbox_title = `${checkbox_task_tag}${title}_${morn_rit_type}${space}`;
  task_checkbox_time = `[time_start${dv_colon}${time_start}]${two_space}[time_end${dv_colon}${time_end}]${two_space}[duration_est${dv_colon}${duration}]${space}`;
  task_checkbox_reminder = `⏰${space}${reminder}${space}`;
  task_checkbox = `${task_checkbox_title}${task_checkbox_time}${task_checkbox_reminder}${inline_date_data}`;

  heading = "### Morning Meditation";
  meditation_bell_heading = "#### Mindfulness Bell Meditation";
  meditation_bell_embed = "![[1_Five Minute Mindfulness Bell Meditation.mp3]]";

  meditation_positive_mind_heading = "#### Positive Mind Meditation";
  meditation_positive_mind_embed =
    "![[morn_1_Positive Mind in 5 Minutes Meditation_Jason Stephenson.mp3]]";

  meditation = `${heading}${two_new_line}${task_checkbox}${two_new_line}${meditation_bell_heading}${two_new_line}${meditation_bell_embed}${two_new_line}${meditation_positive_mind_heading}${two_new_line}${meditation_positive_mind_embed}${two_new_line}`;

  morn_rit_frontmatter = `${hr_line}${new_line}${yaml_morn_rit_title}${yaml_morn_rit_alias}${yaml_date}${yaml_due_do}${yaml_morn_rit_pillar}${yaml_context}${yaml_morn_rit_goal}${yaml_project}${yaml_morn_rit_parent_task}${yaml_morn_rit_organization}${yaml_morn_rit_contact}${yaml_library}${yaml_morn_rit_type}${yaml_file_class}${yaml_date_created}${yaml_date_modified}${yaml_tags}${hr_line}`;

  morn_rit_file_content = `${morn_rit_frontmatter}
${head_lvl(1)}${morn_rit_name}${new_line}
${morn_rit_info}
${head_lvl(2)}Rituals${new_line}
${today_habit_rit_callout}${new_line}
${affirm}${preview}${reflection}${gratitude}${hr_line}${new_line}
${work_start_callout}
${meditation}${hr_line}${new_line}
${head_lvl(2)}Related${new_line}
${head_lvl(3)}Tasks and Events${new_line}
${head_lvl(3)}Notes${new_line}
${hr_line}${new_line}
${head_lvl(2)}Resources${new_line}`;

  morn_rit_directory = `${project_dir}${morn_rit_parent_task_value}`;
  // Create subdirectory file
  await tp.file.create_new(
    morn_rit_file_content,
    morn_rit_file_name,
    false,
    app.vault.getAbstractFileByPath(morn_rit_directory)
  );

  work_start_frontmatter = `${hr_line}${new_line}${yaml_work_start_title}${yaml_work_start_alias}${yaml_date}${yaml_due_do}${yaml_work_start_pillar}${yaml_context}${yaml_work_start_goal}${yaml_project}${yaml_work_start_parent_task}${yaml_work_start_organization}${yaml_work_start_contact}${yaml_library}${yaml_work_start_type}${yaml_file_class}${yaml_date_created}${yaml_date_modified}${yaml_tags}${hr_line}`;

  work_start_file_content = `${work_start_frontmatter}
${head_lvl(1)}${work_start_name}${new_line}
${work_start_info}
${head_lvl(2)}Rituals${new_line}
${email}${whatsapp}${hr_line}${new_line}
${morn_rit_callout}
${head_lvl(2)}Related${new_line}
${head_lvl(3)}Tasks and Events${new_line}
${head_lvl(3)}Notes${new_line}
${hr_line}${new_line}
${head_lvl(2)}Resources${new_line}`;

  work_start_directory = `${project_dir}${work_start_parent_task_value}`;
  // Create subdirectory file
  await tp.file.create_new(
    work_start_file_content,
    work_start_file_name,
    false,
    app.vault.getAbstractFileByPath(work_start_directory)
  );
}
%>
