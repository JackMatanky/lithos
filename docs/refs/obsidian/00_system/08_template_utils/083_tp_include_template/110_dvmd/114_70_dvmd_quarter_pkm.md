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
// WEEK PKM SUBFILE DETAILS
//-------------------------------------------------------------------
// PKM WEEK FILE
const pkm_full_name = "Personal Knowledge Management";
const pkm_name = "PKM";
const pkm_value = "pkm";
const pkm_full_title_name = `${pkm_full_name} for ${long_date}`;
const pkm_short_title_value = `${short_date}_${pkm_value}`;
const pkm_file_name = pkm_short_title_value;

const pkm_section = `${pkm_file_name}${hash}`;

//-------------------------------------------------------------------
// WEEKLY PKM HEADERS
//-------------------------------------------------------------------
heading = "Notes Taken";
const head_notes_taken = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_notes_taken = `[[${pkm_section}${heading}\\|Notes Taken]]`;

// Knowledge Tree
heading = "Knowledge Tree";
const head_tree = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_tree = `[[${pkm_section}${heading}\\|Tree]]`;

// Permanent
heading = "Permanent";
const head_permanent = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_permanent = `[[${pkm_section}${heading}\\|Permanent]]`;

// Literature
heading = "Literature";
const head_literature = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_literature = `[[${pkm_section}${heading}\\|Literature]]`;

// Fleeting
heading = "Fleeting";
const head_fleeting = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_fleeting = `[[${pkm_section}${heading}\\|Fleeting]]`;

// Info
heading = "General";
const head_info = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_info = `[[${pkm_section}${heading}\\|General]]`;

heading = "Note Making";
const head_note_making = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_note_making = `[[${pkm_section}${heading}\\|Note Making]]`;

// Review
heading = "Review";
const head_review = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_review = `[[${pkm_section}${heading}\\|Review]]`;

// Clarify
heading = "Clarify";
const head_clarify = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_clarify = `[[${pkm_section}${heading}\\|Clarify]]`;

// Develop
heading = "Develop";
const head_develop = `${head_lvl(4)}${heading}${two_new_line}`;
const toc_develop = `[[${pkm_section}${heading}\\|Develop]]`;

//-------------------------------------------------------------------
// WEEK PKM SUBFILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_title = `${call_start}[!toc]${space}Week${space}${pkm_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

const toc_body_high = `${call_tbl_start}${toc_tree}${tbl_pipe}${toc_permanent}${tbl_pipe}${toc_literature}${tbl_pipe}${toc_fleeting}${tbl_pipe}${toc_info}${call_tbl_end}${new_line}`;
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
const toc_body_low = `${call_tbl_start}${toc_note_making}${tbl_pipe}${toc_review}${tbl_pipe}${toc_clarify}${tbl_pipe}${toc_develop}${tbl_pipe}${two_space}${call_tbl_end}`;
const toc_body = `${toc_body_high}${toc_body_div}${toc_body_low}`;

const toc_pkm = `${toc_title}${toc_body}${two_new_line}`;

//-------------------------------------------------------------------
// WEEKLY PKM FILES DATAVIEW TABLE
//-------------------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
// Knowledge Tree
query = await tp.user.dv_pkm_type_status_dates({
  type: "tree",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const tree = `${head_tree}${toc_pkm}${query}${two_new_line}`;

// Permanent
query = await tp.user.dv_pkm_type_status_dates({
  type: "permanent",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const permanent = `${head_permanent}${toc_pkm}${query}${two_new_line}`;

// Literature
query = await tp.user.dv_pkm_type_status_dates({
  type: "literature",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const literature = `${head_literature}${toc_pkm}${query}${two_new_line}`;

// Fleeting
query = await tp.user.dv_pkm_type_status_dates({
  type: "fleeting",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const fleeting = `${head_fleeting}${toc_pkm}${query}${two_new_line}`;

// Info
query = await tp.user.dv_pkm_type_status_dates({
  type: "info",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const info = `${head_info}${toc_pkm}${query}${two_new_line}`;

// Review
query = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "review",
  start_date: "",
  end_date: "",
  md: "true",
});
const review = `${head_review}${toc_pkm}${query}${two_new_line}`;

// Clarify
query = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "clarify",
  start_date: "",
  end_date: "",
  md: "true",
});
const clarify = `${head_clarify}${toc_pkm}${query}${two_new_line}`;

// Develop
query = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "develop",
  start_date: "",
  end_date: "",
  md: "true",
});
const develop = `${head_develop}${toc_pkm}${query}${two_new_line}`;

const week_pkm = `${head_notes_taken}${toc_pkm}${tree}${permanent}${literature}${fleeting}${info}${head_note_making}${toc_pkm}${review}${clarify}${develop}`;

tR += week_pkm;
%>