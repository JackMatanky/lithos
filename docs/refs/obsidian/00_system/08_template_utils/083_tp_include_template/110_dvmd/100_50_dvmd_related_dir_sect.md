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

//-------------------------------------------------------------------
// RELATED DIRECTORY SECTION
//-------------------------------------------------------------------
// Outgoing Contacts
heading = `${head_lvl(3)}Outgoing Contact Links${two_new_line}`;
comment = `${cmnt_html_start}Link related contacts here${cmnt_html_end}${two_new_line}`;
const contact_outlink = `${heading}${comment}`;

// Contacts
heading = `${head_lvl(3)}Contacts${two_new_line}`;
query = await tp.user.dv_dir_linked("contact", "true");
const dir_contact = `${heading}${query}${two_new_line}`;

// Outgoing Organizations
heading = `${head_lvl(3)}Outgoing Organization Links${two_new_line}`;
comment = `${cmnt_html_start}Link related organizations here${cmnt_html_end}${two_new_line}`;
const org_outlink = `${heading}${comment}`;

// Organizations
heading = `${head_lvl(3)}Organizations${two_new_line}`;
query = await tp.user.dv_dir_linked("organization", "true");
const dir_org = `${heading}${query}${two_new_line}`;

comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;

const dir_section = `${new_line}${comment}${contact_outlink}${dir_contact}${org_outlink}${dir_org}${hr_line}${new_line}`;

tR += dir_section;
%>
