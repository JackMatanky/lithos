<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const education_proj_dir = "42_education/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";
const lib_books_dir = "60_library/61_books/";

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
// PKM TREE FILES AND NAMES
//-------------------------------------------------------------------
// Knowledge Tree directory
const pkm_dir = "70_pkm/";

const null_arr = ["", "null", "[[null|Null]]", null];
const null_link = "[[null|Null]]";
const null_yaml_li = yaml_li(null_link);
const null_obj = { key: "Null", value: "null", value_link: null_yaml_li, index: 0 };

// SET KNOWLEDGE LEVEL
const pkm_tree_obj_arr = [
  { key: "Subtopic", value: "subtopic", value_link: null_yaml_li, index: 6 },
  { key: "Topic", value: "topic", value_link: null_yaml_li, index: 5 },
  { key: "Subject", value: "subject", value_link: null_yaml_li, index: 4 },
  { key: "Field", value: "field", value_link: null_yaml_li, index: 3 },
  { key: "Branch", value: "branch", value_link: null_yaml_li, index: 2 },
  { key: "Category", value: "category", value_link: null_yaml_li, index: 1 },
];

const pkm_type_obj_arr = [null_obj, pkm_tree_obj_arr].flat();

const pkm_type_obj = await tp.system.suggester(
  (item) => item.key,
  pkm_type_obj_arr,
  false,
  "Direct Knowledge Tree Level?"
);
const pkm_type_value = pkm_type_obj.value;
const pkm_type_name = pkm_type_obj.key;

// SET KNOWLEDGE TREE OBJECT NAME AND VALUE
let pkm_file_dir = "";
let pkm_file_cache = "";
let pkm_link = null_link;
if (pkm_type_value != "null") {
  const pkm_file_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: pkm_dir,
    file_class: "pkm",
    type: "tree",
    subtype: pkm_type_value,
  });
  const pkm_file_obj = await tp.system.suggester(
    (item) => item.key,
    pkm_file_obj_arr,
    false,
    `${pkm_type_name}?`
  );
  pkm_link = `[[${pkm_file_obj.value}|${pkm_file_obj.key}]]`;

  // PKM METADATA CACHE
  const pkm_file_ext = `${pkm_file_obj.value}.md`;
  const pkm_file_path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${pkm_file_ext}`))
    .map((file) => file.path)[0];
  pkm_file_dir = pkm_file_path.replace(pkm_file_ext, "");

  const pkm_tfile = await app.vault.getAbstractFileByPath(pkm_file_path);
  pkm_file_cache = await app.metadataCache.getFileCache(pkm_tfile);
}
const pkm_value_link = yaml_li(pkm_link);

const tree_index = pkm_tree_obj_arr
  .filter((tree) => tree.value == pkm_type_value)
  .map((tree) => tree.index);

tR += pkm_type_value;
tR += two_new_line;
tR += pkm_type_name;
tR += two_new_line;
tR += tree_index;
%>