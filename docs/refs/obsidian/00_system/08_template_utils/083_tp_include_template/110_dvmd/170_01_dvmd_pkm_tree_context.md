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

const type = file_cache?.frontmatter?.type;
const subtype = file_cache?.frontmatter?.subtype;

//-------------------------------------------------------------------
// RELATED PERSONAL KNOWLEDGE MANAGEMENT SECTION
//-------------------------------------------------------------------
heading = "Knowledge Ancestors";
heading = `${head_lvl(3)}${heading}${two_new_line}`;
query = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: subtype,
  relation: "parent",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: subtype,
  relation: "in_parent",
  md: "false",
})
const parent = `${heading}${query_md}${two_new_line}${query}${two_new_line}`;

heading = "Knowledge Siblings";
heading = `${head_lvl(3)}${heading}${two_new_line}`;
query = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: subtype,
  relation: "sibling",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: subtype,
  relation: "in_sibling",
  md: "false",
})
const sibling = `${heading}${query_md}${two_new_line}${query}${two_new_line}`;

heading = "General PKM Tree Items";
comment = `${cmnt_html_start}Link ${heading.toLowerCase()} here${cmnt_html_end}${two_new_line}`;
heading = `${head_lvl(3)}${heading}${two_new_line}`;
query = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: subtype,
  relation: "unrelated",
  md: "true",
})
query = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: subtype,
  relation: "in_unrelated",
  md: "false",
})
const unrelated = `${heading}${comment}${query_md}${two_new_line}${query}${two_new_line}`;

comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;

const pkm_context = `${new_line}${comment}${parent}${sibling}${unrelated}`;

tR += pkm_context;
%>