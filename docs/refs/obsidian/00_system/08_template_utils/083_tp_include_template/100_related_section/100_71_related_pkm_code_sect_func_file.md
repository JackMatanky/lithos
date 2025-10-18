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
// RELATED NOTES BUTTON
//-------------------------------------------------------------------
comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;

const button_name = `name 🕹️Related Code${new_line}`;
const button_type = `type append template${new_line}`;
const button_action = `action 100_81_dvmd_related_code_sect${new_line}`;
const button_replace = `replace [1, 2]${new_line}`;
const button_color = `color purple${new_line}`;

const button = `${comment}${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}`;

//-------------------------------------------------------------------
// RELATED NOTES SECTION
//-------------------------------------------------------------------
const pkm_file_obj_arr = await tp.user.dv_pkm_code_linked({
  type: "snip",
  relation: "lang_ex",
  md: "false",
});

// CODE FILES BY LANGUAGE
// Subtype
heading = `${head_lvl(4)}By Subtype${two_new_line}`;
comment = `${cmnt_html_start}Same language, sublanguage, type, and subtype${cmnt_html_end}${two_new_line}`;
query = await tp.user.dv_pkm_code_linked({
  type: "not_snip",
  relation: "lang_subtype",
  md: "false",
});
let subtype = `${heading}${comment}${query}${two_new_line}`;

// Type
heading = `${head_lvl(4)}By Type${two_new_line}`;
comment = `${cmnt_html_start}Same language, sublanguage, and type${two_space}${new_line}Different subtype${cmnt_html_end}${two_new_line}`;
query = await tp.user.dv_pkm_code_linked({
  type: "not_snip",
  relation: "lang_type_ex",
  md: "false",
});
let type = `${heading}${comment}${query}${two_new_line}`;

// Sublanguage
heading = `${head_lvl(4)}By Sublanguage${two_new_line}`;
comment = `${cmnt_html_start}Same language and sublanguage${two_space}${new_line}Different type and subtype${cmnt_html_end}${two_new_line}`;
query = await tp.user.dv_pkm_code_linked({
  type: "not_snip",
  relation: "lang_sublang_ex",
  md: "false",
});
const sublang = `${heading}${comment}${query}${two_new_line}`;

// Language
heading = `${head_lvl(4)}By Language${two_new_line}`;
comment = `${cmnt_html_start}Same language${two_space}${new_line}Different sublanguage, type, and subtype${cmnt_html_end}${two_new_line}`;
query = await tp.user.dv_pkm_code_linked({
  type: "not_snip",
  relation: "lang_ex",
  md: "false",
});
const lang = `${heading}${comment}${query}${two_new_line}`;

heading = `${head_lvl(3)}Code Files by Language${two_new_line}`;
const code_lang = `${heading}${subtype}${type}${sublang}${lang}`;

// GENERAL CODE FILES
// Subtype
heading = `${head_lvl(4)}By Subtype${two_new_line}`;
comment = `${cmnt_html_start}Same subtype${two_space}${new_line}Different language, sublanguage, and type${cmnt_html_end}${two_new_line}`;
query = await tp.user.dv_pkm_code_linked({
  type: "not_snip",
  relation: "subtype",
  md: "false",
});
subtype = `${heading}${comment}${query}${two_new_line}`;

// Type
heading = `${head_lvl(4)}By Type${two_new_line}`;
comment = `${cmnt_html_start}Same type${two_space}${new_line}Different language, sublanguage, and subtype${cmnt_html_end}${two_new_line}`;
query = await tp.user.dv_pkm_code_linked({
  type: "not_snip",
  relation: "type",
  md: "false",
});
type = `${heading}${comment}${query}${two_new_line}`;

heading = `${head_lvl(3)}General Code Files${two_new_line}`;
const code_general = `${heading}${subtype}${type}`;

// Linked
heading = `${head_lvl(4)}Outgoing Code File Links${two_new_line}`;
comment = `${cmnt_html_start}Link related Code files here${cmnt_html_end}${two_new_line}`;
const outlink_code = `${heading}${comment}`;

heading = `${head_lvl(4)}Code Files${two_new_line}`;
query = await tp.user.dv_pkm_code_linked({
  type: "not_snip",
  relation: "link",
  md: "false",
});
const linked_code = `${heading}${comment}${query}${two_new_line}`;

heading = `${head_lvl(3)}Linked Code Files${two_new_line}`;
const code_link = `${heading}${outlink_code}${linked_code}`;

const code_section = `${button}${code_lang}${code_general}${code_link}${hr_line}${new_line}`;

tR += code_section;
%>