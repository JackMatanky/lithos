<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const cal_dir = "10_calendar";
const cal_day_dir = "10_calendar/11_days/";
const cal_week_dir = "10_calendar/12_weeks/";
const cal_month_dir = "10_calendar/13_months/";
const cal_quarter_dir = "10_calendar/14_quarters/";
const cal_year_dir = "10_calendar/15_years/";

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

const toc_lvl = (obj_arr, int) =>
  call_tbl_start +
  obj_arr
    .filter((x) => x.sect_level == int)
    .map((x) => x.toc)
    .join(tbl_pipe) +
  call_tbl_end;

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// GENERAL VARIABLES AND FUNCTIONS
//-------------------------------------------------------------------
const null_arr = ["", "null", "[[null|Null]]", null];
const null_link = "[[null|Null]]";
const null_yaml_li = yaml_li(null_link);

let heading = "";
let comment = "";
let query = "";

const dv_yaml = "file.frontmatter.";
const dv_current_yaml = `dv.current().${dv_yaml}`;
const dv_luxon_iso = "dv.luxon.DateTime.fromISO";
const dv_sub_type_name = `(${dv_current_yaml}type.includes("_")? ${dv_current_yaml}type.split("_").map((x) => x[0].toUpperCase() + x.slice(1) + "s").join(" and "): (${dv_current_yaml}type == "lib"? "Library": ${dv_current_yaml}toUpperCase())))`;
const dv_task_habit_filter = `(t.text.includes("ritual") || t.text.includes("habit"))`;
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}aliases[0],${space}"Contents")${backtick}`;
const dv_context_link = `${backtick}dv:${space}link(this.${dv_yaml}date,${space}"Context")${backtick}`;

// TABLE OF CONTENTS FUNCTION
function toc_func(date_type, clean_name, file_name, toc_body) {
  const title = [
    call_start + "[!toc]",
    date_type,
    clean_name,
    dv_content_link,
  ].join(space);
  const body = toc_body.replaceAll("_file_section_", file_name + "#");
  return [title, call_start, body].join(new_line);
}

// PREVIOUS AND NEXT DATE LINKS
// OPTIONS: day, month_sub, week, quarter_sub
function date_prev_next(date_arg) {
  const dv_date_yaml = "date_start";
  const date_prev = ").minus({days: 1})";
  let date_next = ").plus({months: 1, days: 1})";
  let dv_luxon_format = `.toFormat("yyyy-MM")`;
  if (date_arg.startsWith("quarter")) {
    date_next = ").plus({months: 3, days: 1})";
    dv_luxon_format = `.toFormat("yyyy-'W'WW")`;
  }
  const dv_link_date = (prev_next) =>
    !date_arg.endsWith("sub")
      ? `${backtick}dvjs: dv.fileLink(${dv_luxon_iso}(${dv_current_yaml}${dv_date_yaml}${prev_next}${dv_luxon_format})${backtick}`
      : `${backtick}dvjs: dv.fileLink(${dv_luxon_iso}(${dv_current_yaml}${dv_date_yaml}${prev_next}${dv_luxon_format} + "_" + ${dv_current_yaml}type, false, ${dv_luxon_iso}(${dv_current_yaml}${dv_date_yaml}${prev_next}${dv_luxon_format} + " " + ${dv_sub_type_name})${backtick}`;
  return (
    "<< " +
    [date_prev, date_next].map((x) => dv_link_date(x)).join(tbl_pipe) +
    " >>" +
    two_new_line +
    hr_line +
    new_line
  );
}

// TASK COUNTS
function dv_task_status_day(status_arg, type_arg) {
  let dv_task_date_comp = `${backtick}dvjs: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks.filter((t) => dv.equal(${dv_luxon_iso}(${dv_current_yaml}date), ${dv_luxon_iso}`;
  if (status_arg == "due") {
    dv_task_date_comp += "(t.due))";
  } else if (status_arg == "done") {
    dv_task_date_comp += `(t.completion)) && t.status == "x"`;
  }
  if (type_arg == "task") {
    return `${dv_task_date_comp} && !${dv_task_habit_filter}).length${backtick}`;
  } else if (type_arg == "hab_rit") {
    return `${dv_task_date_comp} && ${dv_task_habit_filter}).length${backtick}`;
  } else {
    return `${dv_task_date_comp}).length${backtick}`;
  }
}

const total_due_day = `due_total${dv_colon}${dv_task_status_day("due", "")}`;
const task_event_due_day = `due_task_event${dv_colon}${dv_task_status_day(
  "due",
  "task"
)}`;
const habit_ritual_due_day = `due_habit_ritual${dv_colon}${dv_task_status_day(
  "due",
  "hab_rit"
)}`;
const task_due_day =
  ul + [total_due_day, task_event_due_day, habit_ritual_due_day].join(tbl_pipe);

const total_done_day = `done_total${dv_colon}${dv_task_status_day("done", "")}`;
const task_event_done_day = `done_task_event${dv_colon}${dv_task_status_day(
  "done",
  "task"
)}`;
const habit_ritual_done_day = `done_habit_ritual${dv_colon}${dv_task_status_day(
  "done",
  "hab_rit"
)}`;
const task_done_day =
  ul +
  [total_done_day, task_event_done_day, habit_ritual_done_day].join(tbl_pipe);

//-------------------------------------------------------------------
// BUTTONS TABLES AND CALLOUTS
//-------------------------------------------------------------------
const button_table_obj_arr = [
  { path: "00_40_buttons_callout_task_event", content: null },
  { path: "00_40_buttons_table_task_habit_today", content: null },
  { path: "00_42_buttons_table_habit_rit_week", content: null },
  { path: "00_80_buttons_callout_notes", content: null },
  { path: "00_90_buttons_callout_pdev_today", content: null },
];

for (let i = 0; i < button_table_obj_arr.length; i++) {
  button_table_obj_arr[i].content =
    (await tp.user.include_file(button_table_obj_arr[i].path)) + two_new_line;
}

const button_task = button_table_obj_arr[0].content;
const button_task_habit = button_table_obj_arr[1].content;
const button_habit_ritual = button_table_obj_arr[2].content;
const button_notes = button_table_obj_arr[3].content;
const button_pdev = button_table_obj_arr[4].content;

//-------------------------------------------------------------------
// BUTTONS
//-------------------------------------------------------------------
const button_start = `${three_backtick}button`;
const button_end = `${three_backtick}${two_new_line}`;
comment = "Adjust replace lines";
const button_comment = `${cmnt_html_start}${comment}${cmnt_html_end}`;

const month_button_obj_arr = [
  {
    name: "name 🕯️Monthly PDEV Files",
    type: "type append template",
    action: "action 112_90_dvmd_week_pdev",
    replace: "replace [61, 600]",
    color: "color purple",
  },
  {
    name: "name 🏫Monthly Library Content",
    type: "type append template",
    action: "action 112_60_dvmd_week_lib",
    replace: "replace [56, 600]",
    color: "color green",
  },
  {
    name: "name 🗃️Monthly PKM Files",
    type: "type append template",
    action: "action 112_70_dvmd_week_pkm",
    replace: "replace [63, 600]",
    color: "color green",
  },
  {
    name: "name 🦾Monthly Habits and Rituals",
    type: "type append template",
    action: "action 112_45_dvmd_week_habit_rit_due",
    replace: "replace [62, 500]",
    color: "color blue",
  },
  {
    name: "name ✅Monthly Tasks and Events",
    type: "type append template",
    action: "action 112_40_dvmd_week_tasks_due",
    replace: "replace [68, 500]",
    color: "color blue",
  },
];
const quarter_button_obj_arr = [
  {
    name: "name 🕯️Quarterly PDEV Files",
    type: "type append template",
    action: "action 112_90_dvmd_week_pdev",
    replace: "replace [61, 600]",
    color: "color purple",
  },
  {
    name: "name 🏫Quarterly Library Content",
    type: "type append template",
    action: "action 112_60_dvmd_week_lib",
    replace: "replace [56, 600]",
    color: "color green",
  },
  {
    name: "name 🗃️Quarterly PKM Files",
    type: "type append template",
    action: "action 112_70_dvmd_week_pkm",
    replace: "replace [63, 600]",
    color: "color green",
  },
  {
    name: "name 🦾Quarterly Habits and Rituals",
    type: "type append template",
    action: "action 112_45_dvmd_week_habit_rit_due",
    replace: "replace [62, 500]",
    color: "color blue",
  },
  {
    name: "name ✅Quarterly Tasks and Events",
    type: "type append template",
    action: "action 112_40_dvmd_week_tasks_due",
    replace: "replace [68, 500]",
    color: "color blue",
  },
];

const month_button_arr = month_button_obj_arr.map(
  (b) =>
    (b.replace
      ? [button_start, b.name, b.type, b.action, b.replace, b.color, button_end]
      : [button_start, b.name, b.type, b.action, b.color, button_end]
    ).join(new_line) + button_comment
);
const pdev_month_button = month_button_arr[0];
const lib_month_button = month_button_arr[1];
const pkm_month_button = month_button_arr[2];
const habit_rit_month_button = month_button_arr[3];
const tasks_event_month_button = month_button_arr[4];

const quarter_button_arr = quarter_button_obj_arr.map(
  (b) =>
    (b.replace
      ? [button_start, b.name, b.type, b.action, b.replace, b.color, button_end]
      : [button_start, b.name, b.type, b.action, b.color, button_end]
    ).join(new_line) + button_comment
);
const pdev_quarter_button = quarter_button_arr[0];
const lib_quarter_button = quarter_button_arr[1];
const pkm_quarter_button = quarter_button_arr[2];
const habit_rit_quarter_button = quarter_button_arr[3];
const tasks_event_quarter_button = quarter_button_arr[4];

//-------------------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//-------------------------------------------------------------------
const quarter_type_name = "Quarter";
const quarter_type_value = quarter_type_name.toLowerCase();
const quarter_moment_var = `${quarter_type_value}s`;
const quarter_file_class = `cal_${quarter_type_value}`;

//-------------------------------------------------------------------
// SET THE DATE
//-------------------------------------------------------------------
const date_obj_arr = [
  { key: `Current ${quarter_type_name}`, value: "current" },
  { key: `Last ${quarter_type_name}`, value: "last" },
  { key: `Next ${quarter_type_name}`, value: "next" },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  `Which ${quarter_type_name}?`
);

const date_value = date_obj.value;

let full_date = "";
if (date_value == "current") {
  full_date = moment();
} else if (date_value == "next") {
  full_date = moment().add(1, quarter_moment_var);
} else if (date_value == "last") {
  full_date = moment().subtract(1, quarter_moment_var);
} else if (date_value == "last_two") {
  full_date = moment().subtract(2, quarter_moment_var);
}

//-------------------------------------------------------------------
// GENERAL DATE VARIABLES
//-------------------------------------------------------------------
let long_date = moment(full_date).format("Qo [ Quarter of ] YYYY");
let med_date = moment(full_date).format("[Q]Q YYYY");
let short_date = moment(full_date).format("[Q]Q [']YY");
let year_long = moment(full_date).format("YYYY");
let year_short = moment(full_date).format("YY");
let quarter_num = moment(full_date).format("Q");
let month_name_full = moment(full_date).format("MMMM");
let month_name_short = moment(full_date).format("MMM");
let month_num_long = moment(full_date).format("MM");
let month_num_short = moment(full_date).format("M");

//-------------------------------------------------------------------
// WEEK DATE VARIABLES
//-------------------------------------------------------------------
const quarter_number = moment(full_date).format("Q");
const quarter_start = moment(full_date)
  .startOf(quarter_type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const quarter_end = moment(full_date)
  .endOf(quarter_type_value)
  .format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// WEEKDAY DATES ARRAY
//-------------------------------------------------------------------
const moment_month = (int) =>
  moment(full_date)
    .month(quarter_number * 3 - 3 + int)
    .format("YYYY-MM");
const quarter_month_arr = [0, 1, 2].map((x) => moment_month(x));

//-------------------------------------------------------------------
// CALENDAR FILE LINKS AND ALIASES
//-------------------------------------------------------------------
let year_file = `[[${year_long}]]`;
let quarter_file = `[[${year_long}-Q${quarter_num}]]`;
let month_file = `[[${year_long}-${month_num_long}\\|${month_name_short}${space}'${year_short}]]`;

//-------------------------------------------------------------------
// WEEKLY CALENDAR TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const quarter_full_title_name = long_date;
const quarter_short_title_name = med_date;
const quarter_short_title_value = short_date;

const quarter_file_name = med_date;
const quarter_file_section = quarter_file_name + hash;

const quarter_file_alias =
  new_line +
  [quarter_full_title_name, quarter_short_title_value, quarter_short_title_name]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

const quarter_file_dir = `${cal_quarter_dir}${quarter_file_name}/`;

//-------------------------------------------------------------------
// QUARTER CONTEXT CALLOUT
//-------------------------------------------------------------------
const quarter_context_title = `${call_start}[!${quarter_type_value}]${space}${quarter_type_name}${space}Context`;

const quarter_dates_high =
  call_tbl_start + ["Year", "Quarter", "Month"].join(tbl_pipe) + call_tbl_end;
const week_dates_low =
  call_tbl_start +
  [year_file, quarter_file, month_file].join(tbl_pipe) +
  call_tbl_end;
const week_dates = [
  week_dates_high,
  call_tbl_div(3),
  week_dates_low,
  call_start,
].join(new_line);

const week_sub_dates_high =
  call_tbl_start +
  ["Year", "Quarter", "Month", "Week"].join(tbl_pipe) +
  call_tbl_end;
const week_sub_dates_low =
  call_tbl_start +
  [year_file, quarter_file, month_file, week_file].join(tbl_pipe) +
  call_tbl_end;
const week_sub_dates = [
  week_sub_dates_high,
  call_tbl_div(4),
  week_sub_dates_low,
  call_start,
].join(new_line);

const week_days_title = `${call_start}**DAYS**`;
const week_days_high =
  call_tbl_start +
  weekday_arr
    .map((x) => `**[[${x}\\|${moment(x).format("dddd")}]]**`)
    .join(tbl_pipe) +
  call_tbl_end;
const week_days = [
  week_days_title,
  call_start,
  week_days_high,
  call_tbl_div(7),
].join(new_line);

const week_context =
  [week_context_title, call_start, week_dates, week_days].join(new_line) +
  new_line;
const week_sub_context =
  [week_context_title, call_start, week_sub_dates, week_days].join(new_line) +
  new_line;

//-------------------------------------------------------------------
// YAML FRONTMATTER FOR INDIVIDUAL FILES
//-------------------------------------------------------------------
const yaml_top_week = [
  `date_start:${space}${week_start}`,
  `date_end:${space}${week_end}`,
  `year:${space}${year_long}`,
  `quarter:${space}${quarter_num}`,
  `month_name:${space}${month_name_full}`,
  `month_number:${space}${month_num_long}`,
  `week_number:${space}${week_number}`,
].join(new_line);
const yaml_low_week = [
  `file_class:${space}${week_file_class}`,
  "cssclasses:",
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  hr_line,
].join(new_line);

//-------------------------------------------------------------------
// WEEK SUBFILE DETAILS
//-------------------------------------------------------------------
const week_subfile_obj_arr = [
  {
    head_key: "Personal Development",
    key: "PDEV",
    value: "pdev",
  },
  {
    head_key: "Library",
    key: "Library",
    value: "lib",
  },
  {
    head_key: "Knowledge Management",
    key: "PKM",
    value: "pkm",
  },
  {
    head_key: "Habits and Rituals",
    key: "Habits and Rituals",
    value: "habit_ritual",
  },
  {
    head_key: "Tasks and Events",
    key: "Tasks and Events",
    value: "task_event",
  },
];

//-------------------------------------------------------------------
// WEEK SUBFILES CALLOUT
//-------------------------------------------------------------------
const week_files_title = `${call_start}[!dir]${space}Week Subfiles`;
const week_files_high =
  call_tbl_start +
  week_subfile_obj_arr
    .map((x) => `[[${short_date}_${x.value}\\|${x.key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const week_files = [
  week_files_title,
  call_start,
  week_files_high,
  call_tbl_div(5),
  new_line,
].join(new_line);

//-------------------------------------------------------------------
// WEEK FILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_week_title = `${call_start}[!toc]${space}Week${space}${dv_content_link}`;
const toc_week_body =
  call_tbl_start +
  week_subfile_obj_arr
    .map((x) => `[[${week_file_section}${x.head_key}\\|${x.key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week = [
  toc_week_title,
  call_start,
  toc_week_body,
  call_tbl_div(5),
].join(new_line);

//-------------------------------------------------------------------
// WEEKLY REFLECTION FILE LINK AND CALLOUT
//-------------------------------------------------------------------
const reflection_alias = "Weekly Reflection";
const reflection_file_value = reflection_alias
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const reflection_file_name = `${week_file_name}_${reflection_file_value}`;
const reflection_link = `[[${reflection_file_name}\\|${reflection_alias}]]`;
const reflection_button = `${backtick}button-reflection-weekly${backtick}`;
const reflection_callout = `${call_start}[!reflection]${space}${reflection_link}${new_line}${call_start}${new_line}${call_start}${reflection_button}${two_new_line}`;

//-------------------------------------------------------------------
// DAILY PDEV JOURNAL BUTTON CALLOUT
//-------------------------------------------------------------------
const pdev_day_in_button = `${backtick}button-reflection-daily${backtick}`;
const call_pdev_day_button = `${call_start}[!insight]${space}Daily Journals${new_line}${call_start}${new_line}${call_start}${pdev_day_in_button}`;

//-------------------------------------------------------------------
// WEEKLY PDEV OBJECT ARRAY
//-------------------------------------------------------------------
const pdev_obj_arr = [
  {
    sect_level: 1,
    head_key: "Recount",
    toc_key: "Recount",
    type: "recount",
  },
  {
    sect_level: 1,
    head_key: "Best Experiences",
    toc_key: "Experiences",
    type: "best-experience",
  },
  {
    sect_level: 1,
    head_key: "Achievements",
    toc_key: "Achievements",
    type: "achievement",
  },
  {
    sect_level: 1,
    head_key: "Gratitude and Self Gratitude",
    toc_key: "Gratitude",
    type: "gratitude",
  },
  {
    sect_level: 2,
    head_key: "Blind Spots",
    toc_key: "Blindspots",
    type: "blindspot",
  },
  {
    sect_level: 2,
    head_key: "Detachment",
    toc_key: "Detachment",
    type: "detachment",
  },
  {
    sect_level: 2,
    head_key: "Limiting Beliefs",
    toc_key: "Limiting Beliefs",
    type: "limiting_belief",
  },
  {
    sect_level: 2,
    head_key: "Lessons Learned",
    toc_key: "Lessons Learned",
    type: "lesson",
  },
];

pdev_obj_arr.map((x) => (x.head = head_lvl(3) + x.head_key));
pdev_obj_arr.map(
  (x) => (x.toc = `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
);

// WEEK PDEV SUBFILE TABLE OF CONTENTS
const toc_pdev_body = [
  toc_lvl(pdev_obj_arr, 1),
  call_tbl_div(4),
  toc_lvl(pdev_obj_arr, 2),
].join(new_line);

// WEEK PDEV SUBFILE DATAVIEW QUERIES
const query_pdev_week_files = await tp.user.dv_pdev_attr_dates({
  attribute: "file",
  start_date: week_start,
  end_date: week_end,
  md: "false",
});

for (let i = 0; i < pdev_obj_arr.length; i++) {
  pdev_obj_arr[i].query = await tp.user.dv_pdev_attr_dates({
    attribute: pdev_obj_arr[i].type,
    start_date: week_start,
    end_date: week_end,
    md: "false",
  });
}

//-------------------------------------------------------------------
// WEEKLY LIBRARY OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const lib_comment = `${cmnt_html_start}Limit 25${cmnt_html_end}`;

const week_lib_obj_arr = [
  {
    head_key: "Completed This Week",
    toc_key: "Completed",
    status: "done",
  },
  {
    head_key: "Active Content",
    toc_key: "Active",
    status: "active",
  },
  {
    head_key: "Created This Week",
    toc_key: "New",
    status: "new",
  },
  {
    head_key: "Content to Schedule",
    toc_key: "Schedule",
    status: "schedule",
  },
  {
    head_key: "Undetermined Content",
    toc_key: "Undetermined",
    status: "determine",
  },
];

week_lib_obj_arr.map((x) => (x.head = head_lvl(3) + x.head_key));
week_lib_obj_arr.map(
  (x) => (x.toc = `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
);

// WEEK LIBRARY SUBFILE TABLE OF CONTENTS
const toc_week_lib_body =
  call_tbl_start +
  week_lib_obj_arr.map((x) => x.toc).join(tbl_pipe) +
  call_tbl_end +
  new_line +
  call_tbl_div(5);

for (let i = 0; i < week_lib_obj_arr.length; i++) {
  week_lib_obj_arr[i].query = await tp.user.dv_lib_status_dates({
    status: week_lib_obj_arr[i].status,
    start_date: week_start,
    end_date: week_end,
    md: "false",
  });
}

//-------------------------------------------------------------------
// WEEKLY PKM OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const head_notes_taken = head_lvl(3) + "Notes Taken";

heading = "Note Making";
const head_note_making = head_lvl(3) + heading;
const toc_note_making = `[[_file_section_${heading}\\|${heading}]]${tbl_pipe}`;

const week_pkm_obj_arr = [
  {
    sect_level: 1,
    head_key: "Knowledge Tree",
    toc_key: "Tree",
    type: "tree",
    status: "",
  },
  {
    sect_level: 1,
    head_key: "Permanent",
    toc_key: "Permanent",
    type: "permanent",
    status: "",
  },
  {
    sect_level: 1,
    head_key: "Literature",
    toc_key: "Literature",
    type: "literature",
    status: "",
  },
  {
    sect_level: 1,
    head_key: "Fleeting",
    toc_key: "Fleeting",
    type: "fleeting",
    status: "",
  },
  {
    sect_level: 1,
    head_key: "General",
    toc_key: "General",
    type: "info",
    status: "",
  },
  {
    sect_level: 2,
    head_key: "Review",
    toc_key: "Review",
    type: "not_tree",
    status: "review",
  },
  {
    sect_level: 2,
    head_key: "Clarify",
    toc_key: "Clarify",
    type: "not_tree",
    status: "clarify",
  },
  {
    sect_level: 2,
    head_key: "Develop",
    toc_key: "Develop",
    type: "not_tree",
    status: "develop",
  },
];
week_pkm_obj_arr.map((x) => (x.head = head_lvl(4) + x.head_key));
week_pkm_obj_arr.map(
  (x) => (x.toc = `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
);

// WEEK PKM SUBFILE TABLE OF CONTENTS
const toc_week_pkm_low =
  call_tbl_start +
  toc_note_making +
  week_pkm_obj_arr
    .filter((x) => x.sect_level == 2)
    .map((x) => `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  tbl_pipe +
  call_tbl_end;
const toc_week_pkm_body = [
  toc_lvl(week_pkm_obj_arr, 1),
  call_tbl_div(5),
  toc_week_pkm_low,
].join(new_line);

for (let i = 0; i < week_pkm_obj_arr.length; i++) {
  if (week_pkm_obj_arr[i].sect_level == 1) {
    week_pkm_obj_arr[i].query = await tp.user.dv_pkm_type_status_dates({
      type: week_pkm_obj_arr[i].type,
      status: week_pkm_obj_arr[i].status,
      start_date: week_start,
      end_date: week_end,
      md: "false",
    });
  } else {
    week_pkm_obj_arr[i].query = await tp.user.dv_pkm_type_status_dates({
      type: week_pkm_obj_arr[i].type,
      status: week_pkm_obj_arr[i].status,
      start_date: "",
      end_date: "",
      md: "false",
    });
  }
}

//-------------------------------------------------------------------
// WEEKLY HABITS AND RITUALS OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const hab_rit_due = " Due This Week";
const hab_rit_done = " Completed This Week";
const regex_rit = /\sRituals$/g;

const week_habit_rit_obj_arr = [
  {
    head_key: "Habits",
    toc_key: "Due",
    type: "habit",
    status: "due",
  },
  {
    head_key: "Morning Rituals",
    toc_key: "Due",
    type: "morning_ritual",
    status: "due",
  },
  {
    head_key: "Workday Startup Rituals",
    toc_key: "Due",
    type: "workday_startup_ritual",
    status: "due",
  },
  {
    head_key: "Workday Shutdown Rituals",
    toc_key: "Due",
    type: "workday_shutdown_ritual",
    status: "due",
  },
  {
    head_key: "Evening Rituals",
    toc_key: "Due",
    type: "evening_ritual",
    status: "due",
  },
];

// WEEK HABITS AND RITUALS SUBFILE TABLE OF CONTENTS
const toc_week_habit_rit_type =
  call_tbl_start +
  week_habit_rit_obj_arr
    .map(
      (x) =>
        `[[_file_section_${x.head_key}\\|${
          x.head_key.match(regex_rit)
            ? x.head_key.replace(regex_rit, "")
            : x.head_key
        }]]`
    )
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_habit_rit_due =
  call_tbl_start +
  week_habit_rit_obj_arr
    .map((x) => `[[_file_section_${x.head_key}${hab_rit_due}\\|Due]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_habit_rit_done =
  call_tbl_start +
  week_habit_rit_obj_arr
    .map((x) => `[[_file_section_${x.head_key}${hab_rit_done}\\|Done]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_habit_ritual_body = [
  toc_week_habit_rit_type,
  call_tbl_div(5),
  toc_week_habit_rit_due,
  toc_week_habit_rit_done,
].join(new_line);

for (let i = 0; i < week_habit_rit_obj_arr.length; i++) {
  week_habit_rit_obj_arr[i].query = await tp.user.dv_task_type_status_dates({
    type: week_habit_rit_obj_arr[i].type,
    status: week_habit_rit_obj_arr[i].status,
    start_date: week_start,
    end_date: week_end,
    md: "false",
  });
}

week_habit_rit_obj_arr.map((x) => (x.head = head_lvl(3) + x.head_key));
week_habit_rit_obj_arr.map(
  (x) =>
    (x.due = head_lvl(4) + x.head_key + hab_rit_due + two_new_line + x.query)
);
week_habit_rit_obj_arr.map(
  (x) =>
    (x.done =
      head_lvl(4) +
      x.head_key +
      hab_rit_done +
      two_new_line +
      `${cmnt_html_start}dv${colon}${space}${x.type},${space}done${cmnt_html_end}`)
);

//-------------------------------------------------------------------
// WEEKLY TASKS AND EVENTS OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
heading = "Projects";
const head_proj = head_lvl(3) + heading;
const toc_proj = `[[_file_section_${heading}\\|${heading}]]${tbl_pipe}`;

heading = "Parent Tasks";
const head_parent = head_lvl(3) + heading;
const toc_parent = `[[_file_section_${heading}\\|${heading}]]`;

const task_due = "Due on ";
const task_done = "Completed on ";

const week_task_event_obj_arr = [
  {
    sect_level: 1,
    head_key: "Active Projects",
    toc_key: "Active",
    type: "project",
    status: "active",
  },
  {
    sect_level: 1,
    head_key: "Overdue Projects",
    toc_key: "Overdue",
    type: "project",
    status: "overdue",
  },
  {
    sect_level: 2,
    head_key: "Active Parent Tasks",
    toc_key: "Active",
    type: "parent",
    status: "active",
  },
  {
    sect_level: 2,
    head_key: "Overdue Parent Tasks",
    toc_key: "Overdue",
    type: "parent",
    status: "overdue",
  },
  {
    sect_level: 2,
    head_key: "Completed Parent Tasks",
    toc_key: "Completed",
    type: "parent",
    status: "done",
  },
  {
    sect_level: 3,
    head_key: "Sunday",
    toc_key: "Due",
    type: "child",
    status: "due",
  },
  {
    sect_level: 3,
    head_key: "Monday",
    toc_key: "Due",
    type: "child",
    status: "due",
  },
  {
    sect_level: 3,
    head_key: "Tuesday",
    toc_key: "Due",
    type: "child",
    status: "due",
  },
  {
    sect_level: 3,
    head_key: "Wednesday",
    toc_key: "Due",
    type: "child",
    status: "due",
  },
  {
    sect_level: 3,
    head_key: "Thursday",
    toc_key: "Due",
    type: "child",
    status: "due",
  },
  {
    sect_level: 3,
    head_key: "Friday",
    toc_key: "Due",
    type: "child",
    status: "due",
  },
  {
    sect_level: 3,
    head_key: "Saturday",
    toc_key: "Due",
    type: "child",
    status: "due",
  },
];

const toc_week_task_proj_parent =
  call_tbl_start +
  toc_proj +
  toc_parent +
  call_tbl_end +
  new_line +
  call_tbl_div(2);

const toc_week_task_proj =
  call_tbl_start +
  toc_proj +
  week_task_event_obj_arr
    .filter((x) => x.sect_level == 1)
    .map((x) => `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_task_parent =
  call_tbl_start +
  toc_parent +
  tbl_pipe +
  week_task_event_obj_arr
    .filter((x) => x.sect_level == 2)
    .map((x) => `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;

const toc_week_task_child_day =
  call_tbl_start +
  week_task_event_obj_arr
    .filter((x) => x.sect_level == 3)
    .map((x) => `[[_file_section_${x.head_key}${space}Tasks\\|${x.head_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end +
  new_line +
  call_tbl_div(7);
const toc_week_task_child_status =
  call_tbl_start +
  week_task_event_obj_arr
    .filter((x) => x.sect_level == 3)
    .map((x) => `[[_file_section_${task_due}${x.head_key}\\|Due]]`)
    .join(tbl_pipe) +
  call_tbl_end +
  new_line +
  call_tbl_start +
  week_task_event_obj_arr
    .filter((x) => x.sect_level == 3)
    .map((x) => `[[_file_section_${task_done}${x.head_key}\\|Done]]`)
    .join(tbl_pipe) +
  call_tbl_end;

const toc_week_task_body = [
  toc_week_task_proj,
  call_tbl_div(4),
  toc_week_task_parent,
  call_start,
  call_start + "**Daily Tasks**",
  call_start,
  toc_week_task_child_day,
  toc_week_task_child_status,
].join(new_line);
const toc_week_task_parent_body = [
  toc_week_task_proj,
  call_tbl_div(4),
  toc_week_task_parent,
  call_start,
  call_start + "**Daily Tasks**",
  call_start,
  toc_week_task_child_day,
].join(new_line);
const toc_week_task_child_body = [
  toc_week_task_proj_parent,
  call_start,
  call_start + "**Daily Tasks**",
  call_start,
  toc_week_task_child_day,
  toc_week_task_child_status,
].join(new_line);

for (let i = 0; i < week_task_event_obj_arr.length; i++) {
  if (
    week_task_event_obj_arr[i].type == "parent" &&
    week_task_event_obj_arr[i].status == "done"
  ) {
    week_task_event_obj_arr[
      i
    ].query = `${cmnt_html_start}dv${colon}${space}${week_task_event_obj_arr[i].type},${space}done${cmnt_html_end}`;
  } else {
    week_task_event_obj_arr[i].query = await tp.user.dv_task_type_status_dates({
      type: week_task_event_obj_arr[i].type,
      status: week_task_event_obj_arr[i].status,
      start_date: week_start,
      end_date: week_end,
      md: "false",
    });
  }
}

week_task_event_obj_arr
  .filter((x) => x.sect_level != 3)
  .map((x) => (x.head = head_lvl(4) + x.head_key));
week_task_event_obj_arr
  .filter((x) => x.sect_level == 3)
  .map((x) => (x.head = head_lvl(3) + x.head_key + space + "Tasks"));
week_task_event_obj_arr
  .filter((x) => x.sect_level == 3)
  .map(
    (x) =>
      (x.due = head_lvl(4) + task_due + x.head_key + two_new_line + x.query)
  );
week_task_event_obj_arr
  .filter((x) => x.sect_level == 3)
  .map(
    (x) =>
      (x.done =
        head_lvl(4) +
        task_done +
        x.head_key +
        two_new_line +
        `${cmnt_html_start}dv${colon}${space}${x.type},${space}done${cmnt_html_end}`)
  );

//-------------------------------------------------------------------
// WEEK FILE SECTIONS
//-------------------------------------------------------------------
week_subfile_obj_arr.map(
  (x) => (x.embed = `![[${week_file_name}_${x.value}${hash}${x.head_key}]]`)
);
week_subfile_obj_arr.map(
  (x) =>
    (x.section = [head_lvl(2) + x.head_key, toc_week, x.embed, hr_line].join(
      two_new_line
    ))
);
const sections_content_week = week_subfile_obj_arr
  .map((x) => x.section)
  .join(two_new_line);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = week_file_dir;
const folder_path = tp.file.folder(true) + "/";

if (folder_path != directory) {
  await tp.file.move(week_file_dir + week_file_name);
}

//-------------------------------------------------------------------
// WEEK SUBFILES
//-------------------------------------------------------------------
for (let i = 0; i < week_subfile_obj_arr.length; i++) {
  const name_full = week_subfile_obj_arr[i].head_key;
  const name_short = week_subfile_obj_arr[i].key;
  const name_value = week_subfile_obj_arr[i].value;

  const full_title_name = `${name_full}${space}for${space}${long_date}`;
  const short_title_name = `${week_file_name}${space}${name_short}`;
  const short_title_value = `${week_file_name}_${name_value}`;

  const file_name = short_title_value;
  const file_section = file_name + hash;

  const file_alias =
    new_line +
    [full_title_name, short_title_name, short_title_value]
      .map((x) => `${ul_yaml}"${x}"`)
      .join(new_line);

  const frontmatter = [
    hr_line,
    `title:${space}${file_name}`,
    `aliases:${file_alias}`,
    yaml_top_week,
    `type:${space}${name_value}`,
    yaml_low_week,
  ].join(new_line);

  let subfile_content;
  if (name_value == "pdev") {
    const toc_pdev_week = toc_func(
      "Week",
      name_short,
      file_name,
      toc_pdev_body
    );
    subfile_content = [
      toc_pdev_week,
      reflection_callout,
      pdev_week_button,
      query_pdev_week_files,
      pdev_obj_arr
        .map((x) => [x.head, toc_pdev_week, x.query].join(two_new_line))
        .join(two_new_line),
    ].join(two_new_line);
  } else if (name_value == "lib") {
    const toc_lib_week = toc_func(
      "Week",
      name_short,
      file_name,
      toc_week_lib_body
    );
    subfile_content = [
      toc_lib_week,
      lib_week_button,
      week_lib_obj_arr
        .map((x) =>
          [x.head, toc_lib_week, lib_comment, x.query].join(two_new_line)
        )
        .join(two_new_line),
    ].join(two_new_line);
  } else if (name_value == "pkm") {
    const toc_pkm_week = toc_func(
      "Week",
      name_short,
      file_name,
      toc_week_pkm_body
    );
    const sect_high = [
      head_notes_taken,
      toc_pkm_week,
      week_pkm_obj_arr
        .filter((x) => x.sect_level == 1)
        .map((x) => [x.head, toc_pkm_week, x.query].join(two_new_line))
        .join(two_new_line),
    ].join(two_new_line);
    const sect_low = [
      head_note_making,
      toc_pkm_week,
      week_pkm_obj_arr
        .filter((x) => x.sect_level == 2)
        .map((x) => [x.head, toc_pkm_week, x.query].join(two_new_line))
        .join(two_new_line),
    ].join(two_new_line);
    subfile_content = [
      toc_pkm_week,
      button_notes + pkm_week_button,
      sect_high,
      sect_low,
    ].join(two_new_line);
  } else if (name_value == "habit_ritual") {
    const toc_hab_rit_week = toc_func(
      "Week",
      name_short,
      file_name,
      toc_week_habit_ritual_body
    );
    subfile_content = [
      toc_hab_rit_week,
      button_habit_ritual + habit_rit_due_week_button,
      week_habit_rit_obj_arr
        .map((x) =>
          [x.head, toc_hab_rit_week, x.due, x.done].join(two_new_line)
        )
        .join(two_new_line),
    ].join(two_new_line);
  } else if (name_value == "task_event") {
    const toc_task_full_week = toc_func(
      "Week",
      name_short,
      file_name,
      toc_week_task_body
    );
    const toc_task_parent_week = toc_func(
      "Week",
      name_short,
      file_name,
      toc_week_task_parent_body
    );
    const toc_task_child_week = toc_func(
      "Week",
      name_short,
      file_name,
      toc_week_task_child_body
    );
    const sect_proj = [
      head_proj,
      toc_task_full_week,
      week_task_event_obj_arr
        .filter((x) => x.sect_level == 1)
        .map((x) => [x.head, toc_task_parent_week, x.query].join(two_new_line))
        .join(two_new_line),
    ].join(two_new_line);
    const sect_parent = [
      head_parent,
      toc_task_full_week,
      week_task_event_obj_arr
        .filter((x) => x.sect_level == 2)
        .map((x) => [x.head, toc_task_parent_week, x.query].join(two_new_line))
        .join(two_new_line),
    ].join(two_new_line);
    const sect_child = week_task_event_obj_arr
      .filter((x) => x.sect_level == 3)
      .map((x) =>
        [x.head, toc_task_child_week, x.due, x.done].join(two_new_line)
      )
      .join(two_new_line);
    subfile_content = [
      toc_task_full_week,
      button_task + tasks_due_week_button,
      sect_proj,
      sect_parent,
      sect_child,
    ].join(two_new_line);
  }

  const file_path = `${week_file_dir}${file_name}.md`;
  const file_content = [
    frontmatter,
    head_lvl(1) + full_title_name + new_line,
    week_sub_context,
    week_files,
    date_prev_next("week_sub"),
    head_lvl(2) + name_full + new_line,
    subfile_content,
  ].join(new_line);

  await app.vault.create(file_path, file_content);
}

//-------------------------------------------------------------------
// DAY CALENDAR FILES
//-------------------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
const month_type_name = "Month";
const month_type_value = month_type_name.toLowerCase();
const month_moment_var = `${month_type_value}s`;
const month_file_class = `cal_${month_type_value}`;

//-------------------------------------------------------------------
// DAY SUBFILE DETAILS
//-------------------------------------------------------------------
const month_subfile_obj_arr = [
  {
    head_key: "Personal Development",
    key: "PDEV",
    value: "pdev",
  },
  {
    head_key: "Library",
    key: "Library",
    value: "lib",
  },
  {
    head_key: "Knowledge Management",
    key: "PKM",
    value: "pkm",
  },
  {
    head_key: "Tasks and Events",
    key: "Tasks and Events",
    value: "task_event",
  },
];

month_subfile_obj_arr.map(
  (x) => (x.embed = `![[<file_name>_${x.value}${hash}${x.head_key}]]`)
);

//-------------------------------------------------------------------
// DAY CONTEXT CALLOUT
//-------------------------------------------------------------------
const context_month_title = `${call_start}[!day]${space}Day${space}${dv_context_link}`;
const context_month_high =
  call_tbl_start +
  ["Year", "Quarter", "Month", "Week"].join(tbl_pipe) +
  call_tbl_end;

//-------------------------------------------------------------------
// DAY SUBFILES CALLOUT
//-------------------------------------------------------------------
const files_month_title = `${call_start}[!dir]${space}Day Subfiles`;

//-------------------------------------------------------------------
// DAY FILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_month_title = `${call_start}[!toc]${space}Day${space}${dv_content_link}`;

//-------------------------------------------------------------------
// DAILY LIBRARY OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const month_lib_obj_arr = [
  {
    head_key: "Completed Today",
    toc_key: "Done",
    status: "done",
  },
  {
    head_key: "Modified Today",
    toc_key: "Modified",
    status: "modified",
  },
  {
    head_key: "Created Today",
    toc_key: "New",
    status: "new",
  },
];

const toc_month_lib_body =
  call_tbl_start +
  month_lib_obj_arr
    .map((x) => `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end +
  new_line +
  call_tbl_div(3);

month_lib_obj_arr.map((x) => (x.head = head_lvl(3) + x.head_key));

//-------------------------------------------------------------------
// DAILY PKM OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const month_pkm_obj_arr = [
  {
    head_key: "Knowledge Tree",
    toc_key: "Tree",
    type: "tree",
  },
  {
    head_key: "Permanent",
    toc_key: "Permanent",
    type: "permanent",
  },
  {
    head_key: "Literature",
    toc_key: "Literature",
    type: "literature",
  },
  {
    head_key: "Fleeting",
    toc_key: "Fleeting",
    type: "fleeting",
  },
  {
    head_key: "General",
    toc_key: "General",
    type: "info",
  },
];
const toc_month_pkm_body =
  call_tbl_start +
  month_pkm_obj_arr
    .map((x) => `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end +
  new_line +
  call_tbl_div(5);

month_pkm_obj_arr.map((x) => (x.head = head_lvl(3) + x.head_key));

//-------------------------------------------------------------------
// DAILY TASKS AND EVENTS OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const month_task_obj_arr = [
  {
    head_key: "Due Today",
    toc_key: "Due",
    status: "preview",
    content: task_month_button + two_new_line + task_due_day,
  },
  {
    head_key: "Completed Today",
    toc_key: "Done",
    status: null,
    content: task_done_day,
  },
  {
    head_key: "Created Today",
    toc_key: "New",
    status: null,
    content: null,
  },
];
const toc_month_task_body =
  call_tbl_start +
  month_task_obj_arr
    .map((x) => `[[_file_section_${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end +
  new_line +
  call_tbl_div(3);

month_task_obj_arr.map(
  (x) =>
    (x.head = x.content
      ? head_lvl(3) + x.head_key + two_new_line + x.content
      : head_lvl(3) + x.head_key)
);

//-------------------------------------------------------------------
// WEEKDAY FRONTMATTER VARIABLES
//-------------------------------------------------------------------
const yaml_cssclasses_day = [
  "cssclasses:",
  "/read_view_zoom",
  "/read_wide_margin",
  "/inline_title_hide",
].join(`${new_line}${ul_yaml}`);

const yaml_low_day = [
  `file_class:${space}${month_file_class}`,
  `${yaml_cssclasses_day}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  hr_line,
].join(new_line);

// LOOP THROUGH WEEKDAY DATES ARRAY
for (let i = 0; i < weekday_arr.length; i++) {
  const file_name = weekday_arr[i];
  const file_section = file_name + hash;

  // DATE VARIABLES
  long_date = moment(file_name).format("MMMM D, YYYY");
  short_date = moment(file_name).format("YY-M-D");
  year_long = moment(file_name).format("YYYY");
  year_short = moment(file_name).format("YY");
  quarter_num = moment(file_name).format("Q");
  month_name_full = moment(file_name).format("MMMM");
  month_name_short = moment(file_name).format("MMM");
  month_num_long = moment(file_name).format("MM");
  month_num_short = moment(file_name).format("M");
  const year_day = moment(file_name).format("DDDD");
  const month_day_long = moment(file_name).format("DD");
  const weekday_name = moment(file_name).format("dddd");
  const weekday_number = moment(file_name).format("[0]e");

  const yaml_mid_day = [
    `date:${space}${file_name}`,
    `year:${space}${year_long}`,
    `year_day:${space}${year_day}`,
    `quarter:${space}${quarter_num}`,
    `month_name:${space}${month_name_full}`,
    `month_number:${space}${month_num_long}`,
    `month_day:${space}${month_day_long}`,
    `week_number:${space}${week_number}`,
    `weekday_name:${space}${weekday_name}`,
    `weekday_number:${space}${weekday_number}`,
  ].join(new_line);

  // DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
  const month_full_title = `${weekday_name}, ${long_date}`;

  // CALENDAR FILE LINKS AND ALIASES
  year_file = `[[${year_long}]]`;
  quarter_file = `[[${year_long}-Q${quarter_num}]]`;
  month_file = `[[${year_long}-${month_num_long}\\|${month_name_short}${space}'${year_short}]]`;
  week_file = `[[${year_long}-W${week_number}]]`;

  // DAY CONTEXT CALLOUT
  const context_month_low =
    call_tbl_start +
    [year_file, quarter_file, month_file, week_file].join(tbl_pipe) +
    call_tbl_end;
  const context_day =
    [
      context_month_title,
      call_start,
      context_month_high,
      call_tbl_div(4),
      context_month_low,
    ].join(new_line) + two_new_line;

  // DAY SUBFILES
  const files_month_body =
    call_tbl_start +
    month_subfile_obj_arr
      .map((x) => `[[${file_name}_${x.value}\\|${x.key}]]`)
      .join(tbl_pipe) +
    call_tbl_end;
  const files_day = [
    files_month_title,
    call_start,
    files_month_body,
    call_tbl_div(4),
  ].join(new_line);

  // DAY TOC
  const toc_month_body =
    call_tbl_start +
    month_subfile_obj_arr
      .map((x) => `[[${file_section}${x.head_key}\\|${x.key}]]`)
      .join(tbl_pipe) +
    call_tbl_end;
  const toc_day = [
    toc_month_title,
    call_start,
    toc_day_body,
    call_tbl_div(4),
  ].join(new_line);

  const day_file_dir = `${cal_day_dir}${file_name}`;
  await this.app.vault.createFolder(day_file_dir);

  for (let j = 0; j < day_subfile_obj_arr.length; j++) {
    const name_full = day_subfile_obj_arr[j].head_key;
    const name_short = day_subfile_obj_arr[j].key;
    const name_value = day_subfile_obj_arr[j].value;

    const full_title_name = `${name_full}${space}for${space}${day_full_title}`;
    const short_title_name = `${name_full}${space}for${space}${long_date}`;
    const full_title_value = `${file_name}_${name_value}`;
    const short_title_value = `${short_date}_${name_value}`;

    const sub_file_name = full_title_value;
    const sub_file_section = sub_file_name + hash;

    const file_alias =
      new_line +
      [full_title_name, short_title_name, short_title_value, sub_file_name]
        .map((x) => `${ul_yaml}"${x}"`)
        .join(new_line);

    const frontmatter = [
      hr_line,
      `title:${space}${file_name}`,
      `aliases:${file_alias}`,
      yaml_mid_day,
      `type:${space}${name_value}`,
      yaml_low_day,
    ].join(new_line);

    let subfile_content;
    if (name_value == "pdev") {
      const toc_pdev_day = toc_func(
        "Day",
        name_short,
        sub_file_name,
        toc_pdev_body
      );
      const query_pdev_day_files = await tp.user.dv_pdev_attr_dates({
        attribute: "file",
        start_date: file_name,
        end_date: "",
        md: "false",
      });
      for (let l = 0; l < pdev_obj_arr.length; l++) {
        pdev_obj_arr[l].query = await tp.user.dv_pdev_attr_dates({
          attribute: pdev_obj_arr[l].type,
          start_date: file_name,
          end_date: "",
          md: "false",
        });
      }
      subfile_content = [
        toc_pdev_day,
        pdev_day_button,
        call_pdev_day_button,
        query_pdev_day_files,
        pdev_obj_arr
          .map((x) => [x.head, toc_pdev_day, x.query].join(two_new_line))
          .join(two_new_line),
      ].join(two_new_line);
    } else if (name_value == "lib") {
      const toc_lib_day = toc_func(
        "Day",
        name_short,
        sub_file_name,
        toc_day_lib_body
      );
      for (let l = 0; l < day_lib_obj_arr.length; l++) {
        day_lib_obj_arr[l].query = await tp.user.dv_lib_status_dates({
          status: week_lib_obj_arr[l].status,
          start_date: file_name,
          end_date: "",
          md: "false",
        });
      }
      subfile_content = [
        toc_lib_day,
        lib_day_button,
        day_lib_obj_arr
          .map((x) =>
            [x.head, toc_lib_day, lib_comment, x.query].join(two_new_line)
          )
          .join(two_new_line),
      ].join(two_new_line);
    } else if (name_value == "pkm") {
      const toc_pkm_day = toc_func(
        "Day",
        name_short,
        sub_file_name,
        toc_day_pkm_body
      );
      for (let l = 0; l < day_pkm_obj_arr.length; l++) {
        day_pkm_obj_arr[l].query = await tp.user.dv_pkm_type_status_dates({
          type: week_pkm_obj_arr[l].type,
          status: "",
          start_date: file_name,
          end_date: "",
          md: "false",
        });
      }
      subfile_content = [
        toc_pkm_day,
        button_notes + pkm_day_button,
        day_pkm_obj_arr
          .map((x) => [x.head, toc_pkm_day, x.query].join(two_new_line))
          .join(two_new_line),
      ].join(two_new_line);
    } else if (name_value == "task_event") {
      const toc_task_day = toc_func(
        "Day",
        name_short,
        sub_file_name,
        toc_day_task_body
      );
      for (let l = 0; l < day_task_obj_arr.length; l++) {
        if (!day_task_obj_arr[l].status) {
          continue;
        }
        day_task_obj_arr[l].query = await tp.user.dv_task_type_status_dates({
          type: "child",
          status: day_task_obj_arr[l].status,
          start_date: file_name,
          end_date: "",
          md: "false",
        });
      }
      subfile_content = [
        toc_task_day,
        button_task,
        day_task_obj_arr
          .map((x) => [x.head, toc_task_day, x.query].join(two_new_line))
          .join(two_new_line),
      ].join(two_new_line);
    }

    const file_path = `${day_file_dir}/${sub_file_name}.md`;
    const file_content = [
      frontmatter,
      head_lvl(1) + full_title_name + new_line,
      context_day,
      files_day + new_line,
      date_prev_next("day_sub"),
      head_lvl(2) + name_full + new_line,
      subfile_content,
    ].join(new_line);

    await app.vault.create(file_path, file_content);
  }
  const alias_arr = [day_full_title, long_date, short_date, file_name];
  let day_file_alias = "";
  for (let j = 0; j < alias_arr.length; j++) {
    alias = yaml_li(alias_arr[j]);
    day_file_alias += alias;
  }

  const frontmatter = [
    hr_line,
    `title:${space}${file_name}`,
    `aliases:${day_file_alias}`,
    yaml_mid_day,
    yaml_low_day,
  ].join(new_line);

  const sections_content_day = day_subfile_obj_arr
    .map((x) =>
      [
        head_lvl(2) + x.head_key,
        toc_day,
        x.embed.replaceAll("<file_name>", file_name),
        hr_line,
      ].join(two_new_line)
    )
    .join(two_new_line);
  const file_path = `${day_file_dir}/${file_name}.md`;
  const file_content = [
    frontmatter,
    head_lvl(1) + day_full_title + new_line,
    context_day,
    files_day + new_line,
    date_prev_next("day"),
    sections_content_day,
  ].join(new_line);

  await app.vault.create(file_path, file_content);
}
const yaml_week = [
  hr_line,
  `title:${space}${week_file_name}`,
  `aliases:${week_file_alias}`,
  yaml_top_week,
  yaml_low_week,
].join(new_line);

tR += yaml_week;
%>
# <%* tR += week_full_title_name %>

<%* tR += week_context %>
<%* tR += week_files %>
<%* tR += date_prev_next("week") %>
<%* tR += sections_content_week %>