<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";

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
// RETRIEVE LAST AND CURRENT ACTIVE FILE
//-------------------------------------------------------------------
let files = this.app.workspace.getLastOpenFiles();

let last_file = this.app.vault.getAbstractFileByPath(files[0]);
let last_file_name = last_file.name;

let current_file = this.app.workspace.getActiveFile();
let current_file_name = current_file.name;

//-------------------------------------------------------------------
// METADATA CACHE
//-------------------------------------------------------------------
const tfile = tp.file.find_tfile(current_file_name);
const file_cache = await app.metadataCache.getFileCache(tfile);

const date = file_cache?.frontmatter?.date;
const full_title_name = moment(date).format("dddd, MMMM D, YYYY");
const file_name = date;
const file_section = `${file_name}${hash}`;

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
let heading = "";
let comment = "";
let query_md = "";
let query = "";
const toc_dv_contents = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;

//-------------------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const buttons_table_pdev_today = "00_90_buttons_table_pdev_today";
const buttons_table_note = "00_80_buttons_table_notes";
const buttons_table_task_habit_today = "00_40_buttons_table_task_habit_today";

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
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//-------------------------------------------------------------------
const type_name = "Day";
const type_value = type_name.toLowerCase();
const moment_var = `${type_value}s`;
const file_class = `cal_${type_value}`;

//-------------------------------------------------------------------
// PDEV HEADING
//-------------------------------------------------------------------
heading = "Journal Entries";
const head_pdev = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_pdev = `[[${file_section}${heading}\\|PDEV]]`;

//-------------------------------------------------------------------
// PKM HEADINGS
//-------------------------------------------------------------------
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

//-------------------------------------------------------------------
// LIBRARY HEADINGS
//-------------------------------------------------------------------
heading = "Library";
const head_lib = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_lib = `[[${file_name}${hash}${heading}\\|Library]]`;

heading = "Completed Today";
const head_lib_done = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_done = `[[${file_name}${hash}${heading}\\|Done]]`;

heading = "Modified Today";
const head_lib_mod = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_mod = `[[${file_name}${hash}${heading}\\|Modified]]`;

heading = "Created Today";
const head_lib_new = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_new = `[[${file_name}${hash}${heading}\\|New]]`;

const toc_lib_sect = `${call_tbl_start}${toc_lib_done}${tbl_pipe}${toc_lib_mod}${tbl_pipe}${toc_lib_new}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

//-------------------------------------------------------------------
// TASKS AND EVENTS HEADINGS
//-------------------------------------------------------------------
heading = "Tasks and Events";
const head_task = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_task = `[[${file_name}${hash}${heading}\\|Tasks and Events]]`;

heading = "Due Today";
const head_task_due = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_task_due = `[[${file_name}${hash}${heading}\\|Due]]`;

heading = "Completed Today";
const head_task_done = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_task_done = `[[${file_name}${hash}${heading}\\|Done]]`;

heading = "Created Today";
const head_task_new = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_task_new = `[[${file_name}${hash}${heading}\\|New]]`;

const toc_task_sect = `${call_tbl_start}${toc_task_due}${tbl_pipe}${toc_task_done}${tbl_pipe}${toc_task_new}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

//-------------------------------------------------------------------
// TABLE OF CONTENTS
//-------------------------------------------------------------------
toc_title = `${call_start}[!toc]${space}${type_name}${space}[[${file_section}${full_title_name}\|Contents]]${new_line}${call_start}${new_line}`;

toc_section = `${call_tbl_start}${toc_pdev}${tbl_pipe}${toc_pkm}${tbl_pipe}${toc_lib}${tbl_pipe}${toc_task}${call_tbl_end}${new_line}${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
 toc = `${toc_title}${toc_section}${two_new_line}`;

//-------------------------------------------------------------------
// PDEV DATAVIEW LIST
//-------------------------------------------------------------------
query = await tp.user.dv_pdev_date(date, "true");
const pdev = `${head_pdev}${toc}${pdev_buttons_table}${query}${two_new_line}${hr_line}${new_line}`;

//-------------------------------------------------------------------
// DAILY PKM FILES DATAVIEW TABLE
//-------------------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
query = await tp.user.dv_pkm_type_status_dates({
  type: "tree",
  status: "",
  start_date: date,
  end_date: "",
  md: "true",
});
const pkm_tree = `${head_pkm_tree}${query}${two_new_line}`;
 query = await tp.user.dv_pkm_type_status_dates({
  type: "permanent",
  status: "",
  start_date: date,
  end_date: "",
  md: "true",
});
const pkm_perm = `${head_pkm_perm}${query}${two_new_line}`;
 query = await tp.user.dv_pkm_type_status_dates({
  type: "literature",
  status: "",
  start_date: date,
  end_date: "",
  md: "true",
});
const pkm_lit = `${head_pkm_lit}${query}${two_new_line}`;
 query = await tp.user.dv_pkm_type_status_dates({
  type: "fleeting",
  status: "",
  start_date: date,
  end_date: "",
  md: "true",
});
const pkm_fleet = `${head_pkm_fleet}${query}${two_new_line}`;
 query = await tp.user.dv_pkm_type_status_dates({
  type: "info",
  status: "",
  start_date: date,
  end_date: "",
  md: "true",
});
const pkm_info = `${head_pkm_info}${query}${two_new_line}`;
const pkm = `${head_pkm}${toc}${note_buttons_table}${pkm_tree}${pkm_perm}${pkm_lit}${pkm_fleet}${pkm_info}${hr_line}${new_line}`;

//-------------------------------------------------------------------
// LIBRARY DATAVIEW TABLE
//-------------------------------------------------------------------
// STATUS OPTIONS: 'created', 'modified'
comment = `${cmnt_html_start}Limit 50${cmnt_html_end}${two_new_line}`;
query = await tp.user.dv_lib_status_dates({
  status: "done",
  start_date: date,
  end_date: "",
  md: "true",
});
const lib_done = `${head_lib_done}${comment}${query}${two_new_line}`;
 query = await tp.user.dv_lib_status_dates({
  status: "modified",
  start_date: date,
  end_date: "",
  md: "true",
});
const lib_mod = `${head_lib_mod}${comment}${query}${two_new_line}`;

query = await tp.user.dv_lib_status_dates({
  status: "new",
  start_date: date,
  end_date: "",
  md: "true",
});
const lib_new = `${head_lib_new}${comment}${query}${two_new_line}`;
const lib = `${head_lib}${toc}${lib_done}${lib_mod}${lib_new}${hr_line}${new_line}`;

//-------------------------------------------------------------------
// TASKS AND EVENTS DATAVIEW TABLES
//-------------------------------------------------------------------
// STATUS OPTIONS: 'due', 'done', 'new'
query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: date,
  end_date: "",
  md: "true",
});
const task_due = `${head_task_due}${query}${two_new_line}`;

query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "done",
  start_date: date,
  end_date: "",
  md: "true",
});
const task_done = `${head_task_done}${query}${two_new_line}`;

query = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "new",
  start_date: date,
  end_date: "",
  md: "true",
});
const task_new = `${head_task_new}${query}${two_new_line}`;
const task = `${head_task}${toc}${task_habit_buttons_table}${task_due}${task_done}${task_new}${hr_line}${new_line}`;

file_content = `${new_line}${hr_line}${two_new_line}${pdev}${new_line}${pkm}${new_line}${lib}${new_line}${task}`;

tR += file_content;
%>
