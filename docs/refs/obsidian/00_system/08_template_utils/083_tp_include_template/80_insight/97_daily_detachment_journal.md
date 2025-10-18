<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const detachment_journals_dir = "80_insight/97_detachment";

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
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// JOURNAL WRITING DATE
//-------------------------------------------------------------------
const date = moment().format("YYYY-MM-DD");
const date_link = `"[[${date}]]"`;
const full_date_time = moment(`${date}T00:00`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

//-------------------------------------------------------------------
// JOURNAL TYPE AND FILE CLASS
//-------------------------------------------------------------------
const full_type_name = "Daily Detachment Journal";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const long_type_name = `${full_type_name.split(" ")[0]} ${
  full_type_name.split(" ")[1]
}`;
const long_type_value = long_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[1];
const type_value = full_type_value.split("_")[1];
const subtype_name = full_type_name.split(" ")[0];
const subtype_value = full_type_value.split("_")[0];
const file_class = `pdev_${full_type_value.split("_")[2]}`;

//-------------------------------------------------------------------
// JOURNAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${date} ${full_type_name}`;
const partial_title_name = `${date} ${long_type_name}`;
const short_title_name = `${date} ${type_name}`;
const full_title_value = `${short_date_value}_${full_type_value}`;
const partial_title_value = `${short_date_value}_${long_type_value}`;
const short_title_value = `${short_date_value}_${type_value}`;

const file_alias =
  new_line +
  [
    long_type_name,
    full_type_name,
    full_title_name,
    partial_title_name,
    short_title_name,
    full_title_value,
    partial_title_value,
    short_title_value,
  ]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

const file_name = partial_title_value;

//-------------------------------------------------------------------
// PILLAR FILE AND FULL NAME
//-------------------------------------------------------------------
const pillar_name = "Mental Health";
const pillar_value = pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const pillar_link = `[[${pillar_value}|${pillar_name}]]`;
const pillar_value_link = yaml_li(pillar_link);

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goal = "null";

//-------------------------------------------------------------------
// RELATED PROJECT FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const year_month_short = moment().format("YYYY-MM");
const year_month_long = moment().format("MMMM [']YY");

const context_name = "Habits and Rituals";
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();

const project_name = `${year_month_long} ${context_name}`;
const project_value = `${year_month_short}_${context_value}`;
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;

//-------------------------------------------------------------------
// RELATED PARENT TASK FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const habit_ritual_order = "01";
const habit_ritual_name = "Habits";
const parent_task_name = `${year_month_long} ${habit_ritual_name}`;
const parent_task_value = `${year_month_short}_${habit_ritual_order}_${habit_ritual_name.toLowerCase()}`;
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${full_type_name}${space}Details`;
const journal_info = await tp.user.include_file("90_pdev_journal_info_callout");

const info = [info_title, call_start, journal_info].join(new_line);

//-------------------------------------------------------------------
// DETACHMENT DEFINITION AND PROMPTS CALLOUT
//-------------------------------------------------------------------
const detachment_definition = await tp.user.include_file(
  "97_detachment_definition_callout"
);

//-------------------------------------------------------------------
// DETACHMENT HEADINGS AND INLINE DATA
//-------------------------------------------------------------------
const head_suffix = `${space}Detachment${two_new_line}`;
const inline_data = ["Object", "Effect", "Reframe", "Prep"]
  .map((x) => `${ul}**${x}**${colon.repeat(2)}`)
  .join(new_line);

const detach_lvl = (ord) => head_lvl(2) + ord + head_suffix + inline_data;
const detachments = ["1st", "2nd", "3rd", "4th"]
  .map((x) => detach_lvl(x))
  .join(two_new_line);

//-------------------------------------------------------------------
// FILE FRONTMATTER AND CONTENT
//-------------------------------------------------------------------
const frontmatter = [
  hr_line,
  `title:${space}${file_name}`,
  `aliases:${space}${file_alias}`,
  `date:${space}${date_link}`,
  `pillar:${space}${pillar_value_link}`,
  `goal:${space}${goal}`,
  `project:${space}${project_value_link}`,
  `parent_task:${space}${parent_task_value_link}`,
  `subtype:${space}${subtype_value}`,
  `type:${space}${type_value}`,
  `file_class:${space}${file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  "tags:",
  hr_line,
].join(new_line);

const file_content = `${frontmatter}
${head_lvl(1)}${full_type_name}${new_line}
${info}${new_line}
${hr_line}${new_line}
${detachment_definition}${new_line}
${hr_line}${new_line}
${detachments}`;

//-------------------------------------------------------------------
// CREATE FILE IN DIRECTORY
//-------------------------------------------------------------------
const directory = detachment_journals_dir;
await tp.file.create_new(
  file_content,
  file_name,
  false,
  app.vault.getAbstractFileByPath(directory)
);
%>