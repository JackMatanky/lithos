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
// WEEK PDEV SUBFILE DETAILS
//-------------------------------------------------------------------
// PDEV WEEK FILE
const pdev_full_name = "Personal Development Journals";
const pdev_name = "PDEV";
const pdev_value = "pdev";
const pdev_full_title_name = `${pdev_full_name} for ${long_date}`;
const pdev_short_title_value = `${short_date}_${pdev_value}`;
const pdev_file_name = pdev_short_title_value;

const pdev_section = `${pdev_file_name}${hash}`;

//-------------------------------------------------------------------
// WEEKLY PDEV HEADINGS
//-------------------------------------------------------------------
// JOURNAL RECOUNTING LIST
heading = "Recount";
const head_recount = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_recount = `[[${pdev_section}${heading}\\|Recount]]`;

// JOURNAL BEST EXPERIENCE LIST
heading = "Best Experiences";
const head_experience = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_best_experience = `[[${pdev_section}${heading}\\|Experiences]]`;

// JOURNAL ACHIEVEMENTS LIST
heading = "Achievements";
const head_achievement = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_achievement = `[[${pdev_section}${heading}\\|Achievements]]`;

// JOURNAL GRATITUDE LIST
heading = "Gratitude and Self Gratitude";
const head_gratitude = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_gratitude = `[[${pdev_section}${heading}\\|Gratitude]]`;

// JOURNAL BLIND SPOTS LIST
heading = "Blind Spots";
const head_blindspot = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_blindspot = `[[${pdev_section}${heading}\\|Blindspots]]`;

// JOURNAL DETACHMENT LIST
heading = "Detachment";
const head_detach = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_detach = `[[${pdev_section}${heading}\\|Detachment]]`;

// JOURNAL LIMITING BELIEF LIST
heading = "Limiting Beliefs";
const head_limiting_belief = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_limiting_belief = `[[${pdev_section}${heading}\\|Limiting Beliefs]]`;

// JOURNAL LESSONS LIST
heading = "Lessons Learned";
const head_lesson = `${head_lvl(3)}${heading}${two_new_line}`;
const toc_lesson = `[[${pdev_section}${heading}\\|Lessons]]`;

//-------------------------------------------------------------------
// WEEK PDEV SUBFILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_title = `${call_start}[!toc]${space}Week${space}${pdev_name}${space}${toc_dv_contents}${new_line}${call_start}${new_line}`;

const toc_body_head = `${call_tbl_start}${toc_recount}${tbl_pipe}${toc_best_experience}${tbl_pipe}${toc_achievement}${tbl_pipe}${toc_gratitude}${call_tbl_end}${new_line}`;
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
const toc_body_low = `${call_tbl_start}${toc_blindspot}${tbl_pipe}${toc_detach}${tbl_pipe}${toc_limiting_belief}${tbl_pipe}${toc_lesson}${call_tbl_end}`;
const toc_body = `${toc_body_head}${toc_body_div}${toc_body_low}`;


const toc_pdev = `${toc_title}${toc_body}${two_new_line}`;

//-------------------------------------------------------------------
// WEEKLY PDEV DATAVIEW TABLE
//-------------------------------------------------------------------
// MD: "true", "false"
// JOURNAL: "file",
// ATTR: "recount", "best-experience", "blindspot", "achievement",
// ATTR: "gratitude", "detachment", "limiting_belief", "lesson"

// WEEK PDEV FILES
query = await tp.user.dv_pdev_attr_dates({
  attribute: "file",
  start_date: date_start,
  end_date: date_end,
  md: "true",
})
const files = `${toc_pdev}${query}${two_new_line}`;

// JOURNAL RECOUNTING LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "recount",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const recount = `${head_recount}${toc_pdev}${query}${two_new_line}`;

// JOURNAL BEST EXPERIENCE LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "best-experience",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const experience = `${head_experience}${toc_pdev}${query}${two_new_line}`;

// JOURNAL ACHIEVEMENTS LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "achievement",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const achievement = `${head_achievement}${toc_pdev}${query}${two_new_line}`;

// JOURNAL GRATITUDE LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "gratitude",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const gratitude = `${head_gratitude}${toc_pdev}${query}${two_new_line}`;

// JOURNAL BLIND SPOTS LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "blindspot",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const blindspot = `${head_blindspot}${toc_pdev}${query}${two_new_line}`;

// JOURNAL DETACHMENT LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "detachment",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const detach = `${head_detach}${toc_pdev}${query}${two_new_line}`;

// JOURNAL LIMITING BELIEF LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "limiting_belief",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const limiting_belief = `${head_limiting_belief}${toc_pdev}${query}${two_new_line}`;

// JOURNAL LESSONS LIST
query = await tp.user.dv_pdev_attr_dates({
  attribute: "lesson",
  start_date: date_start,
  end_date: date_end,
  md: "true",
});
const lesson = `${head_lesson}${toc_pdev}${query}${two_new_line}`;

const week_pdev = `${new_line}${files}${recount}${experience}${achievement}${gratitude}${blindspot}${detach}${limiting_belief}${lesson}`;

tR += week_pdev;
%>
