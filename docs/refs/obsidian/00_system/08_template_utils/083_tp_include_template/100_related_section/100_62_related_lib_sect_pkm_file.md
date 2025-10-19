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

const null_link = "[[null|Null]]";

//-------------------------------------------------------------------
// RELATED LIBRARY BUTTON
//-------------------------------------------------------------------
comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;
const button_start = `${three_backtick}button`;
const button_end = `${three_backtick}${two_new_line}`;

const button =
  comment +
  [
    button_start,
    "name 🏫Related Library Content",
    "type append template",
    "action 100_60_dvmd_related_lib_sect",
    "replace [1, 2]",
    "color green",
    button_end,
  ].join(new_line);

//-------------------------------------------------------------------
// SET LIBRARY CONTENT FILE NAME AND ALIAS
//-------------------------------------------------------------------
// Library Files Directory
const library_dir = "60_library/";

// Boolean Arrays of Objects
const bool_obj_arr = [
  { key: "✔️ YES ✔️", value: "yes" },
  { key: "❌ NO ❌", value: "no" },
];

const lib_name_alias_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: library_dir,
  file_class: "lib",
  type: "",
});
const ref_name_alias_obj = await tp.system.suggester(
  (item) => item.key,
  lib_name_alias_obj_arr,
  false,
  "Primary Library Resource Reference?"
);
const ref_value = ref_name_alias_obj.value;
const ref_obj = {
  key: ref_name_alias_obj.key,
  value: ref_name_alias_obj.value,
};
const ref_obj_link = `[[${ref_obj.value}|${ref_obj.key}]]`;

let file_arr = ref_value != "null" ? [ref_obj] : [];
let file_filter = ref_value != "null" ? [ref_value] : [];
for (let i = 0; i < 10; i++) {
  // File Suggester
  const lib_name_alias_obj = await tp.system.suggester(
    (item) => item.key,
    lib_name_alias_obj_arr.filter((file) => !file_filter.includes(file.value)),
    false,
    "Related Library Resource?"
  );
  file_basename = lib_name_alias_obj.value;
  file_alias_name = lib_name_alias_obj.key;

  if (file_basename == "null" && file_arr.length == 0) {
    file_link = `${ul}[[${file_basename}|${file_alias_name}]]`;
    file_arr.push(file_link);
    break;
  } else if (file_basename == "null") {
    break;
  } else if (file_basename == "_user_input") {
    file_alias_name = await tp.system.prompt(
      "Resource URL Page Title?",
      null,
      false,
      false
    );
    file_basename = await tp.system.prompt("Resource URL?", null, false, false);
    file_link = `${ul}[${file_alias_name}](${file_basename})`;
    file_arr.push(file_link);
  }
  file_link = `${ul}[[${file_basename}|${file_alias_name}]]`;
  file_arr.push(file_link);
  file_filter.push(file_basename);

  const bool_obj = await tp.system.suggester(
    (item) => item.key,
    bool_obj_arr,
    false,
    "Another Related Library Resource?"
  );

  if (bool_obj.value == "no") {
    break;
  }
}

const lib_link = file_arr.join(new_line);

//-------------------------------------------------------------------
// RELATED LIBRARY SECTION WITH LIBRARY FILE LINK
//-------------------------------------------------------------------
const ref_title = `${call_start}[!Info]`;

const ref_link =
  ref_value != "null" ? `Link::${space}${ref_obj_link}` : "Link::";
const ref_bib = "Bibliography::";
const ref_foot = "Footnote::";

const reference = [ref_title, ref_link, ref_bib, ref_foot].join(
  (new_line + call_start).repeat(2)
) + two_new_line;

heading = `${head_lvl(3)}Outgoing Library Links${two_new_line}`;
comment = `${cmnt_html_start}Link related library files here${cmnt_html_end}${two_new_line}`;
let outlink = `${heading}${comment}${lib_link}${two_new_line}`;
if (lib_link.endsWith(null_link)) {
  outlink = `${heading}${comment}`;
}

heading = `${head_lvl(3)}Library Content${two_new_line}`;
query = await tp.user.dv_lib_linked("", "", "false");
const content = `${heading}${query}${two_new_line}`;

const lib_section = `${reference}${button}${outlink}${content}${hr_line}${new_line}`;

tR += lib_section;
%>
