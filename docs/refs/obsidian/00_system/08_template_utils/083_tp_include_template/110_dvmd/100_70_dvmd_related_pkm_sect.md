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
let query = "";
let query_md = "";

//-------------------------------------------------------------------
// RELATED PERSONAL KNOWLEDGE MANAGEMENT SECTION
//-------------------------------------------------------------------
// Outgoing PKM
heading = `${head_lvl(3)}Outgoing PKM Links${two_new_line}`;
comment = `${cmnt_html_start}Link related pkm files here${cmnt_html_end}${two_new_line}`;
const outlink = `${heading}${comment}`;

// PKM Tree
heading = `${head_lvl(3)}Knowledge Tree${two_new_line}`;
query_md = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "",
  relation: "link",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "",
  relation: "inlink",
  md: "false",
})
const tree = `${heading}${query_md}${two_new_line}${query}${two_new_line}`;

// Permanent Notes
heading = `${head_lvl(3)}Permanent${two_new_line}`;
query_md = await tp.user.dv_pkm_linked({
  type: "permanent",
  subtype: "",
  relation: "link",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "permanent",
  subtype: "",
  relation: "inlink",
  md: "false",
})
const permanent = `${heading}${query_md}${two_new_line}${query}${two_new_line}`;

// Literature Notes
heading = `${head_lvl(3)}Literature${two_new_line}`;
query_md = await tp.user.dv_pkm_linked({
  type: "literature",
  subtype: "",
  relation: "link",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "literature",
  subtype: "",
  relation: "inlink",
  md: "false",
})
const literature = `${heading}${query_md}${two_new_line}${query}${two_new_line}`;

// Fleeting Notes
heading = `${head_lvl(3)}Fleeting${two_new_line}`;
query_md = await tp.user.dv_pkm_linked({
  type: "fleeting",
  subtype: "",
  relation: "link",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "fleeting",
  subtype: "",
  relation: "inlink",
  md: "false",
})
const fleeting = `${heading}${query_md}${two_new_line}${query}${two_new_line}`;

// Info Notes
heading = `${head_lvl(3)}Info${two_new_line}`;
query_md = await tp.user.dv_pkm_linked({
  type: "info",
  subtype: "",
  relation: "link",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "info",
  subtype: "",
  relation: "inlink",
  md: "false",
})
const info = `${heading}${query_md}${two_new_line}${query}${two_new_line}`;

comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;

const pkm_section = `${new_line}${comment}${outlink}${tree}${permanent}${literature}${fleeting}${info}${hr_line}${new_line}`;

tR += pkm_section;
%>
