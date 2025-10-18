<%*
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
// GENERAL VARIABLES
//-------------------------------------------------------------------
let heading = "";
let comment = "";
let query_md = "";
let query = "";
const toc_dv_contents = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;

//-------------------------------------------------------------------
// RETRIEVE CURRENTLY ACTIVE FILE METADATA CACHE
//-------------------------------------------------------------------
const current_file = this.app.workspace.getActiveFile();
const current_file_name = current_file.name;
const tfile = tp.file.find_tfile(current_file_name);
const file_cache = await app.metadataCache.getFileCache(tfile);

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const date_start = file_cache?.frontmatter?.date_start;
const date_end = file_cache?.frontmatter?.date_end;
const long_date = moment(date_start).format("[Week ]ww[,] YYYY");
const short_date = moment(date_start).format("YYYY-[W]ww");

const full_date = moment(date_start);

//-------------------------------------------------------------------
// WEEK TASKS AND EVENTS SUBFILE DETAILS
//-------------------------------------------------------------------
// TASKS AND EVENTS WEEK FILE
const task_event_name = "Tasks and Events";
const task_event_value = "task_event";
const task_event_full_title_name = `${task_event_name} for ${long_date}`;
const task_event_short_title_value = `${short_date}_${task_event_value}`;
const task_event_file_name = task_event_short_title_value;

const task_event_section = `${task_event_file_name}${hash}`;

//-------------------------------------------------------------------
// WEEKDAY DATE VARIABLES
//-------------------------------------------------------------------
const sunday = moment(full_date).add(0, "day").format("YYYY-MM-DD");
const monday = moment(full_date).add(1, "day").format("YYYY-MM-DD");
const tuesday = moment(full_date).add(2, "day").format("YYYY-MM-DD");
const wednesday = moment(full_date).add(3, "day").format("YYYY-MM-DD");
const thursday = moment(full_date).add(4, "day").format("YYYY-MM-DD");
const friday = moment(full_date).add(5, "day").format("YYYY-MM-DD");
const saturday = moment(full_date).add(6, "day").format("YYYY-MM-DD");

//-------------------------------------------------------------------
// WEEKLY COMPLETED TASKS AND EVENTS BUTTON
//-------------------------------------------------------------------
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;

const button_name = `name ✅Daily Task Schedule${new_line}`;
const button_type = `type append template${new_line}`;
const button_action = `action 114_41_dvmd_quarter_tasks_due${new_line}`;
const button_color = `color blue${new_line}`;

const button = `${button_start}${button_name}${button_type}${button_action}${button_color}${button_end}`;

//-------------------------------------------------------------------
// WEEKLY TASKS AND EVENTS DATAVIEW TABLES
//-------------------------------------------------------------------
// ACTIVE PROJECTS
heading = "Active Projects";
const head_proj_active = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_proj_active = `[[${task_event_section}${heading}\\|Projects]]`;

// ACTIVE PARENT TASKS
heading = "Active Parent Tasks";
const head_parent_active = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_parent_active = `[[${task_event_section}${heading}\\|Parent Tasks]]`;

// ACTIVE PARENT TASKS
heading = "Completed Parent Tasks";
const head_parent_done = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_parent_done = `[[${task_event_section}${heading}\\|Parent Tasks]]`;

// PLANNED TASKS
heading = "Due This Week";
const head_task_week_due = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_task_week_due = `[[${task_event_section}${heading}\\|Tasks Due]]`;
const due_prefix = `Due on${space}`;

// COMPLETED TASKS
heading = "Completed This Week";
const head_task_week_done = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_task_week_done = `[[${task_event_section}${heading}\\|Tasks Done]]`;
const done_prefix = `Completed on${space}`;

// SUNDAY TASKS
heading = "Sunday";
const head_sunday_due = `${head_lvl(4)}${due_prefix}${heading}${two_new_line}`;
const head_sunday_done = `${head_lvl(4)}${done_prefix}${heading}${two_new_line}`;
const toc_sunday_due = `[[${task_event_section}${due_prefix}${heading}\\|Due]]`;
const toc_sunday_done = `[[${task_event_section}${done_prefix}${heading}\\|Done]]`;

// MONDAY TASKS
heading = "Monday";
const head_monday_due = `${head_lvl(4)}${due_prefix}${heading}${two_new_line}`;
const head_monday_done = `${head_lvl(4)}${done_prefix}${heading}${two_new_line}`;
const toc_monday_due = `[[${task_event_section}${due_prefix}${heading}\\|Due]]`;
const toc_monday_done = `[[${task_event_section}${done_prefix}${heading}\\|Done]]`;

// TUESDAY TASKS
heading = "Tuesday";
const head_tuesday_due = `${head_lvl(4)}${due_prefix}${heading}${two_new_line}`;
const head_tuesday_done = `${head_lvl(4)}${done_prefix}${heading}${two_new_line}`;
const toc_tuesday_due = `[[${task_event_section}${due_prefix}${heading}\\|Due]]`;
const toc_tuesday_done = `[[${task_event_section}${done_prefix}${heading}\\|Done]]`;

// WEDNESDAY TASKS
heading = "Wednesday";
const head_wednesday_due = `${head_lvl(4)}${due_prefix}${heading}${two_new_line}`;
const head_wednesday_done = `${head_lvl(4)}${done_prefix}${heading}${two_new_line}`;
const toc_wednesday_due = `[[${task_event_section}${due_prefix}${heading}\\|Due]]`;
const toc_wednesday_done = `[[${task_event_section}${done_prefix}${heading}\\|Done]]`;

// THURSDAY TASKS
heading = "Thursday";
const head_thursday_due = `${head_lvl(4)}${due_prefix}${heading}${two_new_line}`;
const head_thursday_done = `${head_lvl(4)}${done_prefix}${heading}${two_new_line}`;
const toc_thursday_due = `[[${task_event_section}${due_prefix}${heading}\\|Due]]`;
const toc_thursday_done = `[[${task_event_section}${done_prefix}${heading}\\|Done]]`;

// FRIDAY TASKS
heading = "Friday";
const head_friday_due = `${head_lvl(4)}${due_prefix}${heading}${two_new_line}`;
const head_friday_done = `${head_lvl(4)}${done_prefix}${heading}${two_new_line}`;
const toc_friday_due = `[[${task_event_section}${due_prefix}${heading}\\|Due]]`;
const toc_friday_done = `[[${task_event_section}${done_prefix}${heading}\\|Done]]`;

// SATURDAY TASKS
heading = "Saturday";
const head_saturday_due = `${head_lvl(4)}${due_prefix}${heading}${two_new_line}`;
const head_saturday_done = `${head_lvl(4)}${done_prefix}${heading}${two_new_line}`;
const toc_saturday_due = `[[${task_event_section}${due_prefix}${heading}\\|Due]]`;
const toc_saturday_done = `[[${task_event_section}${done_prefix}${heading}\\|Done]]`;

//-------------------------------------------------------------------
// WEEK TASKS AND EVENTS SUBFILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_title = `${call_start}[!toc]${space}Week${space}${task_event_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

const high_level = "**High-Level Tasks and Events**";
const toc_body_high_head = `${call_tbl_start}${toc_proj_active}${tbl_pipe}${toc_parent_active}${tbl_pipe}${toc_parent_done}${tbl_pipe}${toc_task_week_due}${tbl_pipe}${toc_task_week_done}${call_tbl_end}${new_line}`;
const toc_body_high_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const toc_body_high = `${toc_body_high_head}${toc_body_high_div}`;

const low_level = "**Daily Tasks and Events**";
const toc_body_low_head = `${call_tbl_start}Sunday${tbl_pipe}Monday${tbl_pipe}Tuesday${tbl_pipe}Wednesday${tbl_pipe}Thursday${tbl_pipe}Friday${tbl_pipe}Saturday${call_tbl_end}${new_line}`;
const toc_body_low_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
const toc_body_low_due = `${call_tbl_start}${toc_sunday_due}${tbl_pipe}${toc_monday_due}${tbl_pipe}${toc_tuesday_due}${tbl_pipe}${toc_wednesday_due}${tbl_pipe}${toc_thursday_due}${tbl_pipe}${toc_friday_due}${tbl_pipe}${toc_saturday_due}${call_tbl_end}${new_line}`;
const toc_body_low_done = `${call_tbl_start}${toc_sunday_done}${tbl_pipe}${toc_monday_done}${tbl_pipe}${toc_tuesday_done}${tbl_pipe}${toc_wednesday_done}${tbl_pipe}${toc_thursday_done}${tbl_pipe}${toc_friday_done}${tbl_pipe}${toc_saturday_done}${call_tbl_end}`;
const toc_body_low = `${toc_body_low_head}${toc_body_low_div}${toc_body_low_due}${toc_body_low_done}`;

const toc_body = `${call_start}${high_level}${new_line}${call_start}${new_line}${toc_body_high}${new_line}${call_start}${new_line}${call_start}${low_level}${new_line}${call_start}${new_line}${toc_body_low}`;

const toc_tasks_events = `${toc_title}${toc_body}${two_new_line}`;
const toc_tasks_events_low = `${toc_title}${toc_body_low}${two_new_line}`;
const toc_tasks_events_high = `${toc_title}${toc_body_high}${two_new_line}`;

//-------------------------------------------------------------------
// WEEKLY TASKS AND EVENTS DATAVIEW TABLES
//-------------------------------------------------------------------
// ACTIVE PROJECTS
query = await tp.user.dv_task_type_status_dates({
  type: "project",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const proj_active = `${head_proj_active}${toc_tasks_events_high}${query}${two_new_line}`;

// ACTIVE PARENT TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "parent_task",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const parent_active = `${head_parent_active}${toc_tasks_events_high}${query}${two_new_line}`;

// SUNDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: sunday,
  end_date: "week",
  md: "true",
});
const sunday_due = `${head_sunday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// MONDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: monday,
  end_date: "week",
  md: "true",
});
const monday_due = `${head_monday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// TUESDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: tuesday,
  end_date: "week",
  md: "true",
});
const tuesday_due = `${head_tuesday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// WEDNESDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: wednesday,
  end_date: "week",
  md: "true",
});
const wednesday_due = `${head_wednesday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// THURSDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: thursday,
  end_date: "week",
  md: "true",
});
const thursday_due = `${head_thursday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// FRIDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: friday,
  end_date: "week",
  md: "true",
});
const friday_due = `${head_friday_due}${toc_tasks_events_low}${query}${two_new_line}`;

// SATURDAY TASKS
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: saturday,
  end_date: "week",
  md: "true",
});
const saturday_due = `${head_saturday_due}${toc_tasks_events_low}${query}${two_new_line}`;

const week_tasks_done = `${head_task_week_done}${toc_tasks_events}${button}`;

const week_tasks_events = `${new_line}${proj_active}${parent_active}${head_task_week_due}${sunday_due}${monday_due}${tuesday_due}${wednesday_due}${thursday_due}${friday_due}${saturday_due}${week_tasks_done}`;

tR += week_tasks_events;
%>