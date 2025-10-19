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
// WEEK LIBRARY SUBFILE DETAILS
//-------------------------------------------------------------------
// LIBRARY WEEK FILE
const lib_name = "Library";
const lib_value = "lib";
const lib_full_title_name = `${lib_name} for ${long_date}`;
const lib_short_title_value = `${short_date}_${lib_value}`;
const lib_file_name = lib_short_title_value;

const lib_section = `${lib_file_name}${hash}`;

//-------------------------------------------------------------------
// WEEKLY LIBRARY HEADINGS
//-------------------------------------------------------------------
comment = `${cmnt_html_start}Limit 25${cmnt_html_end}${two_new_line}`;

// Completed
heading = "Completed This Week";
const head_lib_done = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_done = `[[${lib_section}${heading}\\|Completed]]`;

// Active Content
heading = "Active Content";
const head_lib_active = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_active = `[[${lib_section}${heading}\\|Active]]`;

// New Content
heading = "Created This Week";
const head_lib_new = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_new = `[[${lib_section}${heading}\\|New]]`;

// Content to Schedule
heading = "Content to Schedule";
const head_lib_sched = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_sched = `[[${lib_section}${heading}\\|Schedule]]`;

// Undetermined Content
heading = "Undetermined Content";
const head_lib_undetermined = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lib_undetermined = `[[${lib_section}${heading}\\|Undetermined]]`;

//-------------------------------------------------------------------
// WEEK LIBRARY SUBFILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_title = `${call_start}[!toc]${space}Week${space}${lib_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

const toc_body_head = `${call_tbl_start}${toc_lib_done}${tbl_pipe}${toc_lib_active}${tbl_pipe}${toc_lib_new}${tbl_pipe}${toc_lib_sched}${tbl_pipe}${toc_lib_undetermined}${call_tbl_end}${new_line}`;
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const toc_body = `${toc_body_head}${toc_body_div}`;

const toc_lib = `${toc_title}${toc_body}${two_new_line}`;

//-------------------------------------------------------------------
// WEEKLY LIBRARY DATAVIEW TABLE
//-------------------------------------------------------------------
// Completed
query = await tp.user.dv_lib_status_dates({
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const lib_done = `${head_lib_done}${toc_lib}${comment}${query}${two_new_line}`;

// Active Content
query = await tp.user.dv_lib_status_dates({
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const lib_active = `${head_lib_active}${toc_lib}${comment}${query}${two_new_line}`;

// New Content
query = await tp.user.dv_lib_status_dates({
  status: "new",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const lib_new = `${head_lib_new}${toc_lib}${comment}${query}${two_new_line}`;

// Content to Schedule
query = await tp.user.dv_lib_status_dates({
  status: "schedule",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const lib_sched = `${head_lib_sched}${toc_lib}${comment}${query}${two_new_line}`;

// Undetermined Content
query = await tp.user.dv_lib_status_dates({
  status: "determine",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const lib_undetermined = `${head_lib_undetermined}${toc_lib}${comment}${query}${two_new_line}`;

const week_lib = `${new_line}${lib_done}${lib_active}${lib_new}${lib_sched}${lib_undetermined}`;

tR += week_lib;
%>
