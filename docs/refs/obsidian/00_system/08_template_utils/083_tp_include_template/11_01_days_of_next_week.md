<%*
//-------------------------------------------------------------------
// FOLDER PATH VARIABLES
//-------------------------------------------------------------------
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
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const buttons_table_pdev_today = "00_90_buttons_table_pdev_today";
const buttons_table_note = "00_80_buttons_table_notes";
const buttons_table_task_habit_today = "00_40_buttons_table_task_habit_today";

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
const journal_buttons = include_arr;

temp_file_path = `${sys_temp_include_dir}${buttons_table_note}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const note_buttons = include_arr;

temp_file_path = `${sys_temp_include_dir}${buttons_table_task_habit_today}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const task_habit_buttons = include_arr;

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
const type_name = "Day";
const type_value = type_name.toLowerCase();
const moment_var = `${type_value}s`;
const file_class = `cal_${type_value}`;

//-------------------------------------------------------------------
// WEEKDAY CALENDAR FILES
//-------------------------------------------------------------------
css_class = "/read_view_zoom, /read_wide_margin, /inline_title_hide";
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
let fmatter_week_number = `week_number: ${week_number}`;
let fmatter_weekday_name;
let fmatter_weekday_number;
let fmatter_metatable = `metatable: true`
let fmatter_cssclasses = `cssclasses: [${css_class}]`;
let fmatter_type = `type: ${type_value}`;
let fmatter_file_class = `file_class: ${file_class}`;
let fmatter_date_created = `date_created: ${date_created}`;
let fmatter_date_modified = `date_modified: ${date_modified}`;

// FILE CREATION VARIABLES
let file_name;
let file_content;
const directory = cal_day_dir;

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
  file_name = weekday_arr[i];
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
  prev_date = moment(full_date).subtract(1, moment_var).format("YYYY-MM-DD");
  next_date = moment(full_date).add(1, moment_var).format("YYYY-MM-DD");

  // DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
  full_title_name = `${weekday_name}, ${long_date}`;
  short_title_name = `${long_date}`;
  full_title_value = `${date}`;
  short_title_value = `${short_date}`;

  alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}${full_title_value}${ul_yaml}${short_title_value}`;

  // CALENDAR FILE LINKS AND ALIASES
  year_file = `[[${year_long}]]`;
  quarter_file = `[[${year_long}-Q${quarter_num}]]`;
  month_file = `[[${year_long}-${month_num_long}\\|${month_name_short} '${year_short}]]`;
  week_file = `[[${year_long}-W${week_number}]]`;

  // DAY CONTEXT CALLOUT
  callout = `${call_start}[!${type_value}]${space}${type_name}${space}Context${new_line}${call_start}${new_line}${call_tbl_start}Year${tbl_pipe}Quarter${tbl_pipe}Month${tbl_pipe}Week${call_tbl_end}${new_line}${call_tbl_start}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${call_tbl_end}
${call_tbl_start}${year_file}${tbl_pipe}${quarter_file}${tbl_pipe}${month_file}${tbl_pipe}${week_file}${call_tbl_end}`;

  prev_next_date = `<< [[${prev_date}]] | [[${next_date}]] >>`;

  const toc_title = `${call_start}[!toc]${space}${type_name}${space}[[${file_name}${hash}${full_title_name}\|Contents]]${new_line}${call_start}${new_line}`;
  const toc_section = `${call_tbl_start}[[${file_name}${hash}Journal Entries\\|Journal Entries]]${tbl_pipe}[[${file_name}${hash}Notes\\|Notes]]${tbl_pipe}[[${file_name}${hash}Library\\|Library]]${tbl_pipe}[[${file_name}${hash}Tasks and Events\\|Tasks and Events]]${call_tbl_end}${new_line}`;
  const toc_divide = `${call_tbl_start}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${call_tbl_end}`

  const toc = `${toc_title}${toc_section}${toc_divide}`;

  // DATAVIEW JOURNAL LIST
  const journal_list = await tp.user.dv_pdev_date(date, "false");

  // DAILY TASKS AND EVENTS DATAVIEW TABLES
  // STATUS OPTIONS: 'due', 'done', 'new'
  const tasks_due = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "due",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const tasks_done = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "done",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const tasks_created = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "new",
    start_date: date,
    end_date: "",
    md: "false",
  });

  // DAILY PKM FILES DATAVIEW TABLE
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
    type: "fleeting",
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

  // DAILY LIBRARY DATAVIEW TABLE
  // STATUS OPTIONS: 'created', 'modified'
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

  fmatter_title = `title: ${file_name}`;
  fmatter_alias = `aliases: ${alias_arr}`;
  fmatter_date = `date: ${date}`;
  fmatter_year = `year: ${year_long}`;
  fmatter_year_day = `year_day: ${year_day}`;
  fmatter_quarter = `quarter: ${quarter_num}`;
  fmatter_month_name = `month_name: ${month_name_full}`;
  fmatter_month_number = `month_number: ${month_num_long}`;
  fmatter_month_day = `month_day: ${month_day_long}`;
  fmatter_weekday_name = `weekday_name: ${weekday_name}`;
  fmatter_weekday_number = `weekday_number: ${weekday_number}`;

  frontmatter = `${hr_line}${new_line}${fmatter_title}${new_line}${fmatter_alias}${new_line}${fmatter_date}${new_line}${fmatter_year}${new_line}${fmatter_year_day}${new_line}${fmatter_quarter}${new_line}${fmatter_month_name}${new_line}${fmatter_month_number}${new_line}${fmatter_month_day}${new_line}${fmatter_week_number}${new_line}${fmatter_weekday_name}${new_line}${fmatter_weekday_number}${new_line}${fmatter_metatable}${new_line}${fmatter_cssclasses}${new_line}${fmatter_type}${new_line}${fmatter_file_class}${new_line}${fmatter_date_created}${new_line}${fmatter_date_modified}${new_line}${hr_line}${new_line}`;

  file_content = `${frontmatter}
${hash}${space}${full_title_name}${new_line}
${callout}${new_line}
${prev_next_date}${new_line}
${hr_line}${new_line}
${hash}${hash}${space}Journal Entries${new_line}
${toc}${new_line}
${journal_buttons}${new_line}
${journal_list}${new_line}
${hr_line}${new_line}
${hash}${hash}${space}Notes${new_line}
${toc}${new_line}
${note_buttons}${new_line}
${hash}${hash}${hash}${space}Knowledge Tree
${pkm_tree}${new_line}
${hash}${hash}${hash}${space}Permanent${new_line}
${pkm_note_perm}${new_line}
${hash}${hash}${hash}${space}Literature${new_line}
${pkm_note_lit}${new_line}
${hash}${hash}${hash}${space}Fleeting${new_line}
${pkm_note_fleet}${new_line}
${hash}${hash}${hash}${space}General Info${new_line}
${pkm_note_info}${new_line}
${hr_line}${new_line}
${hash}${hash}${space}Library${new_line}
${toc}${new_line}
${hash}${hash}${hash}${space}Completed Today${new_line}
${cmnt_html_start}Limit 50${cmnt_html_end}${new_line}
${lib_completed}${new_line}
${hash}${hash}${hash}${space}Modified Today${new_line}
${cmnt_html_start}Limit 50${cmnt_html_end}${new_line}
${lib_modified}${new_line}
${hash}${hash}${hash}${space}Created Today${new_line}
${cmnt_html_start}Limit 50${cmnt_html_end}${new_line}
${lib_created}${new_line}
${hr_line}${new_line}
${hash}${hash}${space}Tasks and Events${new_line}
${toc}${new_line}
${task_habit_buttons}${new_line}
${hash}${hash}${hash}${space}Due Today${new_line}
${tasks_due}${new_line}
${hash}${hash}${hash}${space}Completed Today${new_line}
${tasks_done}${new_line}
${hash}${hash}${hash}${space}Created Today${new_line}
${tasks_created}${new_line}
${hr_line}${new_line}
`;

  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(directory)
  );
}
%>
