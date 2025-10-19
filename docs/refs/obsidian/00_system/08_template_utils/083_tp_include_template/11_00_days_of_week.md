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
// GENERAL VARIABLES
//-------------------------------------------------------------------
let heading = "";
let comment = "";
let query_md = "";
let query = "";

//-------------------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const buttons_table_pdev_today = "00_90_buttons_table_pdev_today";
const buttons_table_task_habit_today = "00_40_buttons_table_task_habit_today";
const buttons_table_note = "00_80_buttons_table_notes";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// BUTTONS TABLES
//-------------------------------------------------------------------
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

temp_file_path = `${sys_temp_include_dir}${buttons_table_task_habit_today}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const task_habit_buttons_table = `${include_arr}${two_new_line}`;

//-------------------------------------------------------------------
// DAY MD FILE BUTTON
//-------------------------------------------------------------------
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;

const button_name = `name 📆Day MD File${new_line}`;
const button_type = `type append template${new_line}`;
const button_action = `action 111_00_dvmd_day_file${new_line}`;
const button_replace = `replace [44, 620]${new_line}`;
const button_color = `color purple${new_line}`;

const day_md_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}`;

//-------------------------------------------------------------------
// SET THE WEEK AND NUMBER
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

const week_number = moment(full_date).format("ww");

//-------------------------------------------------------------------
// WEEKDAY CALENDAR VARIABLE
//-------------------------------------------------------------------
const sunday = moment(full_date).day(0).format("YYYY-MM-DD");
const monday = moment(full_date).day(1).format("YYYY-MM-DD");
const tuesday = moment(full_date).day(2).format("YYYY-MM-DD");
const wednesday = moment(full_date).day(3).format("YYYY-MM-DD");
const thursday = moment(full_date).day(4).format("YYYY-MM-DD");
const friday = moment(full_date).day(5).format("YYYY-MM-DD");
const saturday = moment(full_date).day(6).format("YYYY-MM-DD");

//-------------------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//-------------------------------------------------------------------
const day_type_name = "Day";
const day_type_value = day_type_name.toLowerCase();
const day_moment_var = `${day_type_value}s`;
const day_file_class = `cal_${day_type_value}`;

//-------------------------------------------------------------------
// DATE CONTEXT
//-------------------------------------------------------------------
const context_title = `${call_start}[!${day_type_value}]${space}${day_type_name}${space}Context${new_line}${call_start}${new_line}`;
const context_header = `${call_tbl_start}Year${tbl_pipe}Quarter${tbl_pipe}Month${tbl_pipe}Week${call_tbl_end}${new_line}`;
const context_tbl_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

//-------------------------------------------------------------------
// WEEKDAY CALENDAR FILES
//-------------------------------------------------------------------
// FRONTMATTER VARIABLES
let fmatter_title;
let fmatter_alias;
let fmatter_date;
let fmatter_year;
let fmatter_year_day;
let fmatter_quarter;
let fmatter_month_name;
let fmatter_month_number;
let fmatter_month_day;
let fmatter_week_number = `week_number:${space}${week_number}${new_line}`;
let fmatter_weekday_name;
let fmatter_weekday_number;
let fmatter_type = `type:${space}${day_type_value}${new_line}`;
let fmatter_file_class = `file_class:${space}${day_file_class}${new_line}`;
let fmatter_cssclasses = `cssclasses:${new_line}${ul_yaml}/read_view_zoom${new_line}${ul_yaml}/read_wide_margin${new_line}${ul_yaml}/inline_title_hide${new_line}`;
let fmatter_date_created = `date_created:${space}${date_created}${new_line}`;
let fmatter_date_modified = `date_modified:${space}${date_modified}${new_line}`;

// FILE CREATION VARIABLES
let day_file_name;
let file_content;
const day_directory = cal_day_dir;

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

// LOOP THROUGH WEEKDAY DATES ARRAY
for (let i = 0; i < weekday_arr.length; i++) {
  day_file_name = weekday_arr[i];
  day_file_section = `${day_file_name}${hash}`;
  full_date = moment(weekday_arr[i]);

  // DATE VARIABLES
  date = moment(full_date).format("YYYY-MM-DD");
  long_date = moment(full_date).format("MMMM D, YYYY");
  short_date = moment(full_date).format("YY-M-D");
  year_long = moment(full_date).format("YYYY");
  year_short = moment(full_date).format("YY");
  year_day = moment(full_date).format("DDDD");
  quarter_num = moment(full_date).format("Q");
  quarter_ord = moment(full_date).format("Qo");
  month_name_full = moment(full_date).format("MMMM");
  month_name_short = moment(full_date).format("MMM");
  month_num_long = moment(full_date).format("MM");
  month_num_short = moment(full_date).format("M");
  month_day_long = moment(full_date).format("DD");
  month_day_short = moment(full_date).format("D");
  weekday_name = moment(full_date).format("dddd");
  weekday_number = moment(full_date).format("[0]e");
  prev_date = moment(full_date).subtract(1, day_moment_var).format("YYYY-MM-DD");
  next_date = moment(full_date).add(1, day_moment_var).format("YYYY-MM-DD");
  prev_next_date = `<<${space}[[${prev_date}]]${tbl_pipe}[[${next_date}]]${space}>>`;

  // DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
  day_full_title_name = `${weekday_name}, ${long_date}`;
  day_short_title_name = long_date;
  day_full_title_value = date;
  day_short_title_value = short_date;

  alias_arr = [day_full_title_name, day_short_title_name, day_short_title_value, day_full_title_value];
  day_file_alias = "";
  for (var j = 0; j < alias_arr.length; j++) {
    alias = yaml_li(alias_arr[j]);
    day_file_alias += alias;
  };

  // CALENDAR FILE LINKS AND ALIASES
  year_file = `[[${year_long}]]`;
  quarter_file = `[[${year_long}-Q${quarter_num}]]`;
  month_file = `[[${year_long}-${month_num_long}\\|${month_name_short} '${year_short}]]`;
  week_file = `[[${year_long}-W${week_number}]]`;

  // DAY CONTEXT CALLOUT
  context_links = `${call_tbl_start}${year_file}${tbl_pipe}${quarter_file}${tbl_pipe}${month_file}${tbl_pipe}${week_file}${call_tbl_end}`;

  context = `${context_title}${context_header}${context_tbl_div}${context_links}${two_new_line}`;

  // PDEV HEADING
  heading = "Journal Entries";
  const head_pdev = `${head_lvl(2)}${heading}${two_new_line}`;
  const toc_pdev = `[[${file_section}${heading}\\|PDEV]]`;

  // PKM HEADINGS
  heading = "Personal Knowledge Management";
  const head_pkm = `${head_lvl(2)}${heading}${two_new_line}`;
  const toc_pkm = `[[${file_section}${heading}\\|PKM]]`;

  heading = "Knowledge Tree";
  const head_pkm_tree = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_pkm_tree = `[[${file_section}${heading}\\|Tree]]`;

  heading = "Permanent";
  const head_pkm_perm = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_pkm_perm = `[[${file_section}${heading}\\|Permanent]]`;

  heading = "Literature";
  const head_pkm_lit = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_pkm_lit = `[[${file_section}${heading}\\|Literature]]`;

  heading = "Fleeting";
  const head_pkm_fleet = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_pkm_fleet = `[[${file_section}${heading}\\|Fleeting]]`;

  heading = "General Info";
  const head_pkm_info = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_pkm_info = `[[${file_section}${heading}\\|Info]]`;

  const toc_pkm_sect = `${call_tbl_start}${toc_pkm_tree}${tbl_pipe}${toc_pkm_perm}${tbl_pipe}${toc_pkm_lit}${tbl_pipe}${toc_pkm_fleet}${tbl_pipe}${toc_pkm_info}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

  // LIBRARY HEADINGS
  heading = "Library";
  const head_lib = `${head_lvl(2)}${heading}${two_new_line}`;
  const toc_lib = `[[${day_file_section}${heading}\\|Library]]`;

  heading = "Completed Today";
  const head_lib_done = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_lib_done = `[[${day_file_section}${heading}\\|Done]]`;

  heading = "Modified Today";
  const head_lib_mod = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_lib_mod = `[[${day_file_section}${heading}\\|Modified]]`;

  heading = "Created Today";
  const head_lib_new = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_lib_new = `[[${day_file_section}${heading}\\|New]]`;

  const toc_lib_sect = `${call_tbl_start}${toc_lib_done}${tbl_pipe}${toc_lib_mod}${tbl_pipe}${toc_lib_new}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

  // TASKS AND EVENTS HEADINGS
  heading = "Tasks and Events";
  const head_task = `${head_lvl(2)}${heading}${two_new_line}`;
  const toc_task = `[[${day_file_section}${heading}\\|Tasks and Events]]`;

  heading = "Due Today";
  const head_task_due = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_task_due = `[[${day_file_section}${heading}\\|Due]]`;

  heading = "Completed Today";
  const head_task_done = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_task_done = `[[${day_file_section}${heading}\\|Done]]`;

  heading = "Created Today";
  const head_task_new = `${head_lvl(3)}${heading}${two_new_line}`;
  const toc_task_new = `[[${day_file_section}${heading}\\|New]]`;

  const toc_task_sect = `${call_tbl_start}${toc_task_due}${tbl_pipe}${toc_task_done}${tbl_pipe}${toc_task_new}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

  // TABLE OF CONTENTS
  toc_title = `${call_start}[!toc]${space}${day_type_name}${space}[[${file_section}${day_full_title_name}\|Contents]]${new_line}${call_start}${new_line}`;

  toc_section = `${call_tbl_start}${toc_pdev}${tbl_pipe}${toc_pkm}${tbl_pipe}${toc_lib}${tbl_pipe}${toc_task}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;

  toc = `${toc_title}${toc_section}${two_new_line}`;

  // PDEV DATAVIEW LIST
  query = await tp.user.dv_pdev_date(date, "false");
  const pdev = `${head_pdev}${toc}${pdev_buttons_table}${query}${two_new_line}${hr_line}${new_line}`;

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
  const pkm_tree = `${head_pkm_tree}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "permanent",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const pkm_perm = `${head_pkm_perm}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "literature",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const pkm_lit = `${head_pkm_lit}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "fleeting",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const pkm_fleet = `${head_pkm_fleet}${query}${two_new_line}`;

  query = await tp.user.dv_pkm_type_status_dates({
    type: "info",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const pkm_info = `${head_pkm_info}${query}${two_new_line}`;
  const pkm = `${head_pkm}${toc}${note_buttons_table}${pkm_tree}${pkm_perm}${pkm_lit}${pkm_fleet}${pkm_info}${hr_line}${new_line}`;

  // LIBRARY DATAVIEW TABLE
  // STATUS OPTIONS: 'created', 'modified'
  comment = `${cmnt_html_start}Limit 50${cmnt_html_end}${two_new_line}`;
  query = await tp.user.dv_lib_status_dates({
    status: "done",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const lib_done = `${head_lib_done}${comment}${query}${two_new_line}`;

  query = await tp.user.dv_lib_status_dates({
    status: "modified",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const lib_mod = `${head_lib_mod}${comment}${query}${two_new_line}`;

  query = await tp.user.dv_lib_status_dates({
    status: "new",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const lib_new = `${head_lib_new}${comment}${query}${two_new_line}`;
  const lib = `${head_lib}${toc}${lib_done}${lib_mod}${lib_new}${hr_line}${new_line}`;

  // TASKS AND EVENTS DATAVIEW TABLES
  // STATUS OPTIONS: 'due', 'done', 'new'
  query = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "due",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const task_due = `${head_task_due}${query}${two_new_line}`;

  query = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "done",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const task_done = `${head_task_done}${query}${two_new_line}`;

  query = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "new",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const task_new = `${head_task_new}${query}${two_new_line}`;
  const task = `${head_task}${toc}${task_habit_buttons_table}${task_due}${task_done}${task_new}${hr_line}${new_line}`;

  fmatter_title = `title:${space}${day_file_name}${new_line}`;
  fmatter_alias = `aliases:${day_file_alias}${new_line}`;
  fmatter_date = `date:${space}${date}${new_line}`;
  fmatter_year = `year:${space}${year_long}${new_line}`;
  fmatter_year_day = `year_day:${space}${year_day}${new_line}`;
  fmatter_quarter = `quarter:${space}${quarter_num}${new_line}`;
  fmatter_month_name = `month_name:${space}${month_name_full}${new_line}`;
  fmatter_month_number = `month_number:${space}${month_num_long}${new_line}`;
  fmatter_month_day = `month_day:${space}${month_day_long}${new_line}`;
  fmatter_weekday_name = `weekday_name:${space}${weekday_name}${new_line}`;
  fmatter_weekday_number = `weekday_number:${space}${weekday_number}${new_line}`;

  frontmatter = `${hr_line}${new_line}${fmatter_title}${fmatter_alias}${fmatter_date}${fmatter_year}${fmatter_year_day}${fmatter_quarter}${fmatter_month_name}${fmatter_month_number}${fmatter_month_day}${fmatter_week_number}${fmatter_weekday_name}${fmatter_weekday_number}${fmatter_type}${fmatter_file_class}${fmatter_cssclasses}${fmatter_date_created}${fmatter_date_modified}${hr_line}`;

  file_content = `${frontmatter}
${head_lvl(1)}${day_full_title_name}${new_line}
${context}${prev_next_date}${new_line}
${day_md_button}${new_line}
${hr_line}${new_line}
${pdev}${new_line}${pkm}${new_line}${lib}${new_line}${task}`;

  await tp.file.create_new(
    file_content,
    day_file_name,
    false,
    app.vault.getAbstractFileByPath(day_directory)
  );
}
%>
