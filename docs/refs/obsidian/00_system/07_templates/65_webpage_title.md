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

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
let title = await tp.system.prompt("Webpage Title", null, true, false);
title = title.trim();
title = await tp.user.title_case(title);

//-------------------------------------------------------------------
// SET AUTHOR'S CONTACT FILE NAME AND TITLE
//-------------------------------------------------------------------
const contact_name_alias = await tp.user.include_template(
  tp,
  "51_contact_name_alias"
);
const contact_value = contact_name_alias.split(";")[0];
const contact_name = contact_name_alias.split(";")[1];
const contact_value_link = contact_name_alias.split(";")[2];

const name_last_value_arr = contact_value
  .split(", ")
  .map((c) => c.split("_")[0]);
let contact_name_last_value;
if (name_last_value_arr.length >= 3) {
  contact_name_last_value = `${name_last_value_arr[0]}_et_al`;
} else if (name_last_value_arr.length >= 2) {
  contact_name_last_value = `${name_last_value_arr[0]}_${name_last_value_arr[1]}`;
} else if (name_last_value_arr.length >= 1) {
  contact_name_last_value = name_last_value_arr[0];
}

//-------------------------------------------------------------------
// SET PUBLISHED DATE
//-------------------------------------------------------------------
const year_published = await tp.system.prompt(
  "What year was the article published?"
);

//-------------------------------------------------------------------
// BOOK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//-------------------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const full_title_name = lib_content_titles.full_title_name;
const full_title_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sub_title;
const main_title_value = main_title.replaceAll(/[\s-]/g, "_").toLowerCase();

const file_name = `${contact_name_last_value}_${year_published}_${main_title_value}`;
const file_section = file_name + hash;

const file_alias =
  new_line +
  [full_title_name, full_title_value, main_title, main_title_value, file_name]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
main_title: <%* tR += main_title %>
subtitle: <%* tR += subtitle %>
