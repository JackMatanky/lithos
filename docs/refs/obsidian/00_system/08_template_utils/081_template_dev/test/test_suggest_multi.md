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
// NULL VALUE, NAME, LINK, AND YAML LINK
//-------------------------------------------------------------------
const null_value = "null";
const null_name = "Null";
const null_link = `[[${null_value}|${null_name}]]`;
const null_yaml_li = yaml_li(null_link);

const file_section = "_file_#"

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
// CAREER DEVELOPMENT PILLAR FILE AND FULL NAME
const preset_pillar_name = "Career Development";
const preset_pillar_value = preset_pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const preset_pillar_link = `[[${preset_pillar_value}|${preset_pillar_name}]]`;
const preset_pillar_value_link = yaml_li(preset_pillar_link);

// Pillar Files Directory
const type_name = "Pillar";

// Files Directory
const directory = pillars_dir;

// Boolean Arrays of Objects
const bool_obj_arr = [
  { key: "✔️ YES ✔️", value: "yes" },
  { key: "❌ NO ❌", value: "no" },
];

// Retrieve all files in the Pillars directory
const file_by_status_obj_arr = await tp.user.file_by_status({
  dir: directory,
  status: "active",
});

async function multi_suggester(type, obj_arr, preset_arr) {
  let file_obj_arr = [];
  if (typeof preset_arg == "object") {
    // try object assignment syntax preset.key = key
    file_obj_arr = preset_arg.map((x) => ({key: x.replace(/.+\|(.+)\]\]/g, "$1"), value: x.replace(/\[\[([\w_]+)\|.+/g, "$1") }));
  }
  let file_filter = [];
  if (file_obj_arr) {
    file_filter.push(file_obj_arr.map((x) => x.value));
  }
  for (let i = 0; i < 10; i++) {
    // File Suggester
    const file_suggest_obj = await tp.system.suggester(
      (item) => item.key,
      obj_arr.filter((file) => !file_filter.includes(file.value)),
      false,
      `${type}?`
    );
    const file_obj = {
      key: file_suggest_obj.key,
      value: file_suggest_obj.value,
    };

    if (file_obj.value == "null") {
      if (file_obj_arr) {
        break;
      }
      file_obj_arr.push(file_obj);
      break;
    }
    file_obj_arr.push(file_obj);
    file_filter.push(file_obj.value);

    const bool_obj = await tp.system.suggester(
      (item) => item.key,
      bool_obj_arr,
      false,
      `Another ${type}?`
    );

    if (bool_obj.value == "no") {
      break;
    }
  }
  return file_obj_arr
    .map((file) => `${new_line}${ul_yaml}"[[${file.value}|${file.key}]]"`)
    .join("");
}

const pillar = await multi_suggester(type_name, file_by_status_obj_arr, [preset_pillar_link]);

tR += pillar;
tR += two_new_line;
%>
