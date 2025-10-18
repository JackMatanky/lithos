<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pkm_dir = "70_pkm/";

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
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const pillar_name_alias_preset_know = "20_02_pillar_name_alias_preset_know";
const pkm_tree_hierarchy_callout = "70_pkm_tree_hierarchy_callout";
const code_type_subtype = "71_code_type_subtype";
const note_status = "80_note_status";
const pkm_code_info_callout = "81_pkm_code_info_callout";

const related_lib_sect_file = "100_60_related_lib_sect";
const related_pkm_sect = "100_70_related_pkm_sect";
const related_pkm_code_sect_func_file = "100_71_related_pkm_code_sect_func_file";
const related_pkm_snip_sect_func_file = "100_71_related_pkm_snip_sect_func_file";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
const null_arr = ["", "null", "[[null|Null]]", null];
const null_link = "[[null|Null]]";
const null_yaml_li = yaml_li(null_link);
const null_obj = {
  key: "Null",
  value: "null",
  value_link: null_yaml_li,
  index: 0,
};

//-------------------------------------------------------------------
// SET PROGRAMMING LANGUAGE AND FILE CLASS
//-------------------------------------------------------------------
const language_name = "Google Sheets";
const language_value = "google_sheets";
const language_value_short = "gs";
const topic_link = `[[${language_value}|${language_name}]]`;
const topic_value_link = yaml_li(topic_link);
const file_class = "pkm_code";

//-------------------------------------------------------------------
// LANGUAGE (TOPIC) METADATA CACHE
//-------------------------------------------------------------------
const language_value_ext = `${language_value}.md`;
const language_file_path = await app.vault
  .getMarkdownFiles()
  .filter((file) => file.path.endsWith(`/${language_value_ext}`))
  .map((file) => file.path)[0];
const language_file_dir = language_file_path.replace(language_value_ext, "");
const language_tfile = await app.vault.getAbstractFileByPath(
  language_file_path
);
const language_file_cache = await app.metadataCache.getFileCache(
  language_tfile
);

//-------------------------------------------------------------------
// PKM TREE FILES AND NAMES
//-------------------------------------------------------------------
const pkm_tree_obj_arr = [
  { key: "Subject", value: "subject", value_link: null_yaml_li, index: 4 },
  { key: "Field", value: "field", value_link: null_yaml_li, index: 3 },
  { key: "Branch", value: "branch", value_link: null_yaml_li, index: 2 },
  { key: "Category", value: "category", value_link: null_yaml_li, index: 1 },
];

for (let i = 0; i < pkm_tree_obj_arr.length; i++) {
  const pkm_yaml =
    language_file_cache?.frontmatter?.[pkm_tree_obj_arr[i].value];
  if (!null_arr.includes(pkm_yaml) && typeof pkm_yaml != "undefined") {
    pkm_tree_obj_arr[i].value_link = pkm_yaml
      .toString()
      .split(",")
      .map((tree_yaml) => yaml_li(tree_yaml))
      .join("");
  }
}
const pkm_tree_value_link = pkm_tree_obj_arr
  .map((tree) => tree.value_link)
  .reverse();

const category_value_link = pkm_tree_value_link[0];
const branch_value_link = pkm_tree_value_link[1];
const field_value_link = pkm_tree_value_link[2];
const subject_value_link = pkm_tree_value_link[3];
const subtopic_value_link = null_yaml_li;

//-------------------------------------------------------------------
// SET TYPE
//-------------------------------------------------------------------
const type_name = "Function";
const type_value = "function";
const type_value_short = "func";

//-------------------------------------------------------------------
// SET SUBTYPE
//-------------------------------------------------------------------
const subtype_obj_arr = [
  { key: "Null", value: "null", value_short: "null" },
  { key: "Aggregate", value: "aggregate", value_short: "agg" },
  { key: "Array", value: "array", value_short: "arr" },
  { key: "Boolean", value: "boolean", value_short: "bool" },
  { key: "Converter", value: "converter", value_short: "convert" },
  { key: "Database", value: "database", value_short: "db" },
  { key: "DataFrame", value: "dataframe", value_short: "df" },
  { key: "Date", value: "date", value_short: "date" },
  { key: "Dictionary", value: "dictionary", value_short: "dict" },
  { key: "Engineering", value: "engineering", value_short: "eng" },
  { key: "File", value: "file", value_short: "file" },
  { key: "Filter", value: "filter", value_short: "fltr" },
  { key: "Financial", value: "financial", value_short: "fin" },
  { key: "Google", value: "google", value_short: "goog" },
  { key: "Info", value: "info", value_short: "info" },
  { key: "List", value: "list", value_short: "list" },
  { key: "Logical", value: "logical", value_short: "logic" },
  { key: "Lookup", value: "lookup", value_short: "look" },
  { key: "Math", value: "math", value_short: "math" },
  { key: "Numeric", value: "numeric", value_short: "num" },
  { key: "Object", value: "object", value_short: "obj" },
  { key: "Operator", value: "operator", value_short: "opr" },
  { key: "Parser", value: "parser", value_short: "prs" },
  {
    key: "Pass-Through RAWSQL",
    value: "pass_through_rawsql",
    value_short: "rawsql",
  },
  { key: "Regular Expression", value: "regex", value_short: "regex" },
  { key: "Series", value: "series", value_short: "ser" },
  { key: "Set", value: "set", value_short: "set" },
  { key: "Spatial", value: "spatial", value_short: "space" },
  { key: "Statistics", value: "statistics", value_short: "stat" },
  { key: "String", value: "string", value_short: "str" },
  {
    key: "Table Calculation",
    value: "table_calculation",
    value_short: "table_calc",
  },
  { key: "Tuple", value: "tuple", value_short: "tupl" },
  { key: "User", value: "user", value_short: "user" },
  { key: "Web", value: "web", value_short: "web" },
  { key: "General", value: "general", value_short: "general" },
];

//-------------------------------------------------------------------
// PARAMETER CALLOUT
//-------------------------------------------------------------------
const parameter_title = `${call_start}[!param]${space}${type_name}${space}Parameters${two_space}${new_line}${call_start}${new_line}`;
const parameter_char_include = /[^\s\w\d\*,_=\\-]/g;

/* ---------------------------------------------------------- */
/*   SET PILLAR FILE NAME AND TITLE; PRESET KNOW. EXPANSION   */
/* ---------------------------------------------------------- */
const pillar_know = `${new_line}${ul_yaml}"[[knowledge_expansion|Knowledge Expansion]]"`;
const pillar_data = `${new_line}${ul_yaml}"[[data_analyst|Data Analyst]]"`;
const pillar_value_link = `${pillar_know}${pillar_data}`;

/* ---------------------------------------------------------- */
/*                       SET NOTE STATUS                      */
/* ---------------------------------------------------------- */
const status_value = "resource";

//-------------------------------------------------------------------
// CODE INFORMATION SECTION
//-------------------------------------------------------------------
heading = "Code Information";
const head_code_info = `${head_lvl(2)}${heading}${two_new_line}`;

heading = "Syntax";
const head_code_info_syntax = `${head_lvl(3)}${heading}${two_new_line}`;

heading = "Parameter Values";
const head_code_info_parameter = `${head_lvl(3)}${heading}${two_new_line}`;

heading = "Examples";
const head_code_info_example = `${head_lvl(3)}${heading}${two_new_line}`;
const block_code_info_example = `${three_backtick}sql${two_new_line}${three_backtick}`;
const code_info_example = `${head_code_info_example}${block_code_info_example}${two_new_line}`;

//-------------------------------------------------------------------
// RELATED SNIPPET SECTION
//-------------------------------------------------------------------
heading = "Related Snippets";
const head_related_snip = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_pkm_snip_sect_func_file}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_snip_section = include_arr;

//-------------------------------------------------------------------
// RELATED CODE SECTION
//-------------------------------------------------------------------
heading = "Related Code";
const head_related_code = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_pkm_code_sect_func_file}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_code_section = include_arr;

//-------------------------------------------------------------------
// RELATED PKM SECTION
//-------------------------------------------------------------------
heading = "Related Knowledge";
const head_related_pkm = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_pkm_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_pkm_section = include_arr;

// PKM TREE HIERARCHY CALLOUT
temp_file_path = `${sys_temp_include_dir}${pkm_tree_hierarchy_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const pkm_tree_hierarchy = `${include_arr}${two_new_line}`;

//-------------------------------------------------------------------
// RELATED LIBRARY SECTION
//-------------------------------------------------------------------
heading = "Related Library Content";
const head_related_lib = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_lib_sect_file}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_lib_section = include_arr;

//-------------------------------------------------------------------
// FILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_dv_contents = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;
const toc_title = `${call_start}[!toc]${space}${toc_dv_contents}${two_space}${new_line}${call_start}${new_line}`;

const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${type_name}${space}Details${new_line}${call_start}${new_line}`;

temp_file_path = `${sys_temp_include_dir}${pkm_code_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const info_body = include_arr;

const info = `${info_title}${info_body}${two_new_line}${hr_line}${new_line}`;

//-------------------------------------------------------------------
// CODE FRONTMATTER
//-------------------------------------------------------------------
let yaml_title;
let yaml_alias;
let yaml_pillar = `pillar:${space}${pillar_value_link}${new_line}`;
let yaml_category = `category:${space}${category_value_link}${new_line}`;
let yaml_branch = `branch:${space}${branch_value_link}${new_line}`;
let yaml_field = `field:${space}${field_value_link}${new_line}`;
let yaml_subject = `subject:${space}${subject_value_link}${new_line}`;
let yaml_topic = `topic:${space}${topic_value_link}${new_line}`;
let yaml_subtopic = `subtopic:${space}${subtopic_value_link}${new_line}`;
let yaml_syntax;
let yaml_url;
let yaml_about;
let yaml_status = `status:${space}${status_value}${new_line}`;
let yaml_subtype;
let yaml_type = `type:${space}${type_value}${new_line}`;
let yaml_file_class = `file_class:${space}${file_class}${new_line}`;
let yaml_date_created = `date_created:${space}${date_created}${new_line}`;
let yaml_date_modified = `date_modified:${space}${date_modified}${new_line}`;
let yaml_tags = `tags:${new_line}`;

const directory = `${language_file_dir}${type_value}s`;
let file_name;
let file_content;



//-------------------------------------------------------------------
// CREATE NEW CODE FUNCTION FILES
//-------------------------------------------------------------------
for (let i = 0; i < obj_arr.length; i++) {
  // SUBTYPE
  const subtype_name = obj_arr[i].subtype_name;
  const subtype_value = obj_arr[i].subtype_value;
  const subtype_value_short = subtype_obj_arr
    .filter((x) => x.value == subtype_value)
    .map((x) => x.value_short);
  yaml_subtype = `subtype:${space}${subtype_value}${new_line}`;
  // TITLES, ALIAS, AND FILE NAME
  const title_name = `${obj_arr[i].title}()`;
  const title_value = obj_arr[i].title_value;

  const full_title_name = `${language_name} ${title_name} ${subtype_name} ${type_name}`;
  const full_title_value = `${language_value}_${title_value}_${subtype_value}_${type_value}`;
  const partial_title_name = `${language_name} ${title_name} ${type_name}`;
  const partial_title_value = `${language_value}_${title_value}_${type_value}`;
  const short_title_name = `${language_name} ${title_name}`;
  const short_title_value = `${language_value}_${title_value}`;

  const file_name = `${language_value_short}_${title_value}_${subtype_value_short}_${type_value_short}`;
  const file_section = `${file_name}${hash}`;

  const alias_arr = [
    title_name,
    full_title_name,
    full_title_value,
    partial_title_name,
    partial_title_value,
    short_title_name,
    short_title_value,
    file_name,
  ];
  let file_alias = "";
  for (let j = 0; j < alias_arr.length; j++) {
    alias = yaml_li(alias_arr[j]);
    file_alias += alias;
  }
  yaml_title = `title:${space}${file_name}${new_line}`;
  yaml_alias = `aliases:${space}${file_alias}${new_line}`;

  // SYNTAX
  let syntax = obj_arr[i].syntax;
  const syntax_parameters = syntax.replace(/^.+\(/g, "(");
  const syntax_value = `"${syntax}"`;
  const block_code_info_syntax = `${three_backtick}sql${new_line}${syntax}${new_line}${three_backtick}`;
  const code_info_syntax = `${head_code_info_syntax}${block_code_info_syntax}${two_new_line}`;
  yaml_syntax = `syntax:${space}${syntax_value}${new_line}`;

  // PARAMETER CALLOUT
  const parameters = syntax_parameters
    .replaceAll(parameter_char_include, "")
    .replaceAll(/\*/g, "\\*");
  const parameter_arr = parameters.split(",");
  
  let ol_parameter_count = 0;
  let parameter_body = "";
  for (let k = 0; k < parameter_arr.length; k++) {
    ol_parameter_count = ol_parameter_count + 1;
    parameter_name = `${call_start}${ol_parameter_count}.${space}${backtick}${parameter_arr[k]}${backtick}${new_line}`;
    parameter_type = `${call_ul_indent}**Type**:${new_line}`;
    parameter_description = `${call_ul_indent}**Description**:${new_line}`;
    parameter_section = `${parameter_name}${parameter_type}${parameter_description}`;
    parameter_body += parameter_section;
  }
  const parameter_callout = `${parameter_title}${parameter_body}`;
  const code_info_parameter = `${head_code_info_parameter}${parameter_callout}${two_new_line}`;

  // DESCRIPTION
  const about = obj_arr[i].definition;
  const about_value = about
    .replaceAll(/^(\s*)([^\s])/g, "$2")
    .replaceAll(/(\s*)\n/g, "\n")
    .replaceAll(/([^\s])(\s*)$/g, "$1")
    .replaceAll(/\n{1,6}/g, "<new_line>")
    .replaceAll(/(<new_line>)(\w)/g, "\n \n $2")
    .replaceAll(/(<new_line>)(-\s|\d\.\s)/g, "\n $2");
  yaml_about = `about:${space}|${new_line}${space}${about_value}${new_line}`;

  // REFERENCE URL
  const url = obj_arr[i].url;
  yaml_url = `url:${space}${url}${new_line}`;

  heading = "Code Information";
  const toc_code_info = `[[${file_section}${heading}\\|Code Info]]`;
  heading = "Related Snippets";
  const toc_related_snip = `[[${file_section}${heading}\\|Use Cases]]`;
  heading = "Related Code";
  const toc_related_code = `[[${file_section}${heading}\\|Related Code]]`;
  heading = "Related Knowledge";
  const toc_related_pkm = `[[${file_section}${heading}\\|PKM]]`;
  heading = "Related Library Content";
  const toc_related_lib = `[[${file_section}${heading}\\|Library]]`;

  // TABLE OF CONTENTS CALLOUT
  toc_body_high = `${call_tbl_start}${toc_code_info}${tbl_pipe}${toc_related_snip}${tbl_pipe}${toc_related_code}${tbl_pipe}${toc_related_pkm}${tbl_pipe}${toc_related_lib}${call_tbl_end}${new_line}`;
  toc_body = `${toc_body_high}${toc_body_div}`;
  toc = `${toc_title}${toc_body}${two_new_line}`;

  const code_info_section = `${code_info_syntax}${code_info_parameter}${code_info_example}${hr_line}${new_line}`;

  // FILE SECTIONS
  const code_info = `${head_code_info}${toc}${code_info_section}`;
  const related_snip = `${head_related_snip}${toc}${related_snip_section}`;
  const related_code = `${head_related_code}${toc}${related_code_section}`;
  const related_pkm = `${head_related_pkm}${toc}${pkm_tree_hierarchy}${related_pkm_section}`;
  const related_lib = `${head_related_lib}${toc}${related_lib_section}`;

  frontmatter = `${hr_line}${new_line}${yaml_title}${yaml_alias}${yaml_pillar}${yaml_category}${yaml_branch}${yaml_field}${yaml_subject}${yaml_topic}${yaml_subtopic}${yaml_syntax}${yaml_url}${yaml_about}${yaml_status}${yaml_subtype}${yaml_type}${yaml_file_class}${yaml_date_created}${yaml_date_modified}${yaml_tags}${hr_line}`;

  file_content = `${frontmatter}
${head_lvl(1)}${full_title_name}${new_line}
${info}
${code_info}
${related_snip}
${related_code}
${related_pkm}
${related_lib}
${head_lvl(2)}Flashcards${new_line}`;

  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(directory)
  );
}
%>