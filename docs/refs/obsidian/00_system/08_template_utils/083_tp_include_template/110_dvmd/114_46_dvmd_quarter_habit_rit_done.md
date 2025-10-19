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

//-------------------------------------------------------------------
// WEEK HABITS AND RITUALS SUBFILE DETAILS
//-------------------------------------------------------------------
// HABITS AND RITUALS WEEK FILE
const habit_rit_name = "Habits and Rituals";
const habit_rit_value = "habit_ritual";
const habit_rit_full_title_name = `${habit_rit_name} for ${long_date}`;
const habit_rit_short_title_value = `${short_date}_${habit_rit_value}`;
const habit_rit_file_name = habit_rit_short_title_value;

const habit_rit_section = `${habit_rit_file_name}${hash}`;

//-------------------------------------------------------------------
// WEEKLY HABITS AND RITUALS HEADERS
//-------------------------------------------------------------------
heading = "Due This Week";
const head_habit_rit_due = `${head_lvl(3)}${heading}${two_new_line}`;
const due_suffix = `${space}${heading}`;

heading = "Completed This Week";
const head_habit_rit_done = `${head_lvl(3)}${heading}${two_new_line}`;
const done_suffix = `${space}${heading}`;

// HABITS
heading = "Habits";
const head_habit_done = `${head_lvl(4)}${heading}${done_suffix}${two_new_line}`;
const toc_habit_due = `[[${habit_rit_section}${heading}${due_suffix}\\|Due]]`;
const toc_habit_done = `[[${habit_rit_section}${heading}${done_suffix}\\|Done]]`;

// MORNING RITUALS
heading = "Morning Rituals";
const head_morn_rit_done = `${head_lvl(4)}${heading}${done_suffix}${two_new_line}`;
const toc_morn_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\\|Due]]`;
const toc_morn_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\\|Done]]`;

// WORKDAY STARTUP RITUALS
heading = "Workday Startup Rituals";
const head_work_start_rit_done = `${head_lvl(4)}${heading}${done_suffix}${two_new_line}`;
const toc_work_start_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\\|Due]]`;
const toc_work_start_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\\|Done]]`;

// WORKDAY SHUTDOWN RITUALS
heading = "Workday Shutdown Rituals";
const head_work_shut_rit_done = `${head_lvl(4)}${heading}${done_suffix}${two_new_line}`;
const toc_work_shut_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\\|Due]]`;
const toc_work_shut_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\\|Done]]`;

// EVENING RITUALS
heading = "Evening Rituals";
const head_eve_rit_done = `${head_lvl(4)}${heading}${done_suffix}${two_new_line}`;
const toc_eve_rit_due = `[[${habit_rit_section}${heading}${due_suffix}\\|Due]]`;
const toc_eve_rit_done = `[[${habit_rit_section}${heading}${done_suffix}\\|Done]]`;

//-------------------------------------------------------------------
// WEEK HABITS AND RITUALS SUBFILE CONTENTS CALLOUT
//-------------------------------------------------------------------
toc_title = `${call_start}[!toc]${space}Week${space}${habit_rit_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

const toc_body_title = `${call_tbl_start}Habits${tbl_pipe}Morning${tbl_pipe}Workday${space}Startup${tbl_pipe}Workday${space}Shutdown${tbl_pipe}Evening${call_tbl_end}${new_line}`
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`
const toc_body_due = `${call_tbl_start}${toc_habit_due}${tbl_pipe}${toc_morn_rit_due}${tbl_pipe}${toc_work_start_rit_due}${tbl_pipe}${toc_work_shut_rit_due}${tbl_pipe}${toc_eve_rit_due}${call_tbl_end}${new_line}`
const toc_body_done = `${call_tbl_start}${toc_habit_done}${tbl_pipe}${toc_morn_rit_done}${tbl_pipe}${toc_work_start_rit_done}${tbl_pipe}${toc_work_shut_rit_done}${tbl_pipe}${toc_eve_rit_done}${call_tbl_end}`;
const toc_body = `${toc_body_title}${toc_body_div}${toc_body_due}${toc_body_done}`

const toc_habit_rit = `${toc_title}${toc_body}${two_new_line}`;

//-------------------------------------------------------------------
// WEEKLY HABITS AND RITUALS DATAVIEW TABLES
//-------------------------------------------------------------------
// TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// STATUS OPTIONS: "due", "done"
// HABITS
query = await tp.user.dv_task_type_status_dates({
  type: "habit",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const habit_done = `${head_habit_done}${toc_habit_rit}${query}${two_new_line}`;

// MORNING RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "morning_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const morn_rit_done = `${head_morn_rit_done}${toc_habit_rit}${query}${two_new_line}`;

// WORKDAY STARTUP RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "workday_startup_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const work_start_rit_done = `${head_work_start_rit_done}${toc_habit_rit}${query}${two_new_line}`;

// WORKDAY SHUTDOWN RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "workday_shutdown_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const work_shut_rit_done = `${head_work_shut_rit_done}${toc_habit_rit}${query}${two_new_line}`;

// EVENING RITUALS
query = await tp.user.dv_task_type_status_dates({
  type: "evening_ritual",
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const eve_rit_done = `${head_eve_rit_done}${toc_habit_rit}${query}${two_new_line}`;

const week_habit_rit = `${habit_done}${morn_rit_done}${work_start_rit_done}${work_shut_rit_done}${eve_rit_done}`;

tR += week_habit_rit;
%>
