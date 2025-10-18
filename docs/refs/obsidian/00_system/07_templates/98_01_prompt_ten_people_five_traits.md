<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const insights_dir = "80_insight/";
const prompt_journals_dir = "80_insight/98_prompt_journals/";
const goals_dir = "30_goals/";

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
const pillar_name_alias_preset_mental = "20_03_pillar_name_alias_preset_mental";
const related_project = "40_related_project";
const pdev_journal_info_callout = "90_pdev_journal_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// JOURNAL TYPE, SUBTYPE, AND FILE CLASS
//-------------------------------------------------------------------
const full_type_name = "General Prompt Journal";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[1];
const type_value = full_type_value.split("_")[1];
const subtype_name = full_type_name.split(" ")[0];
const subtype_value = full_type_value.split("_")[0];
const file_class = `pdev_${full_type_value.split("_")[2]}`;

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const prompt = "Who are ten people I admire and five of their character traits?";
const title = "Ten People and Their Five Admirable Characteristics";
const title_case = await tp.user.title_case(title);

//-------------------------------------------------------------------
// SET WRITING DATE
//-------------------------------------------------------------------
const date = await tp.user.nl_date(tp, "");
const date_link = `"[[${date}]]"`;
const full_date_time = moment(`${date}T00:00`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

//-------------------------------------------------------------------
// JOURNAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${date} ${title_case}`;
const short_title_name = title_case
  .replaceAll(/[#:\*<>\|\\/-]/g, " ")
  .replaceAll(/\?/g, "")
  .replaceAll(/"/g, "'")
  .toLowerCase();
const short_title_value = short_title_name.replaceAll(/\s/g, "_").replaceAll(/\s/g, "_");
const full_title_value = `${short_date_value}_${short_title_value}`;

const alias_arr = [
  prompt,
  title_case,
  full_title_name,
  short_title_name,
  short_title_value,
  full_title_value
];
let file_alias = "";
for (let i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
};

const file_name = full_title_value;

//-------------------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET MENTAL HEALTH
//-------------------------------------------------------------------
const pillar_name_alias = await tp.user.include_template(
  tp,
  "20_03_pillar_name_alias_preset_mental"
);
const pillar_value = pillar_name_alias.split(";")[0];
const pillar_value_link = pillar_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${full_type_name}?`
);

//-------------------------------------------------------------------
// SET RELATED PROJECT
//-------------------------------------------------------------------
const project_name_alias = await tp.user.include_template(
  tp,
  "40_related_project"
);

const project_value = project_name_alias.split(";")[0];
const project_name = project_name_alias.split(";")[1];
const project_value_link = yaml_li(`[[${project_value}|${project_name}]]`);

//-------------------------------------------------------------------
// SET RELATED PARENT TASK
//-------------------------------------------------------------------
let parent_task_value = "null";
let parent_task_name = "Null";
if (project_value!== "null") {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_value,
    file_class: "task",
    type: "parent_task",
  });
  const parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    `Related Parent Task to the ${full_type_name}?`
);
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
};
const parent_task_value_link = yaml_li(`[[${parent_task_value}|${parent_task_name}]]`);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${full_type_name}${space}Details`;
const journal_info = await tp.user.include_file("90_pdev_journal_info_callout");

const info = [info_title, call_start, journal_info].join(new_line) + new_line;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = prompt_journals_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
   await tp.file.move(`${directory}${file_name}`);
};

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_link %>
pillar: <%* tR += pillar_value_link %>
goal: <%* tR += goal %>
project: <%* tR += project_value_link %>
parent_task: <%* tR += parent_task_value_link %>
subtype: <%* tR += subtype_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title_case %>

<%* tR += info %>

---

## Prompt

> [!question] Prompt
> 
> - Who are ten people I admire?
> - What are five of their characteristics I find admirable?

