<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const insights_dir = "80_insight/";
const vis_three_yr_dir = "80_insight/91_vision/03_three_year/";

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
const full_type_name = "Three Year Vision";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.replace(" Vision", "");
const type_value = full_type_value.replace("_vision", "");
const file_class = `pdev_${full_type_value.split("_")[2]}`;

//-------------------------------------------------------------------
// SET WRITING DATE
//-------------------------------------------------------------------
const date = await tp.user.nl_date(tp, "");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const full_date_time = moment(`${date}T00:00`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

//-------------------------------------------------------------------
// JOURNAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${date} ${full_type_name}`;
const short_title_name = full_type_name
  .replaceAll(/[#:\*<>\|\\/-]/g, " ")
  .replaceAll(/\?/g, "")
  .replaceAll(/"/g, "'")
  .toLowerCase();
const short_title_value = full_type_value;
const full_title_value = `${short_date_value}_${short_title_value}`;

const alias_arr = [
  full_type_name,
  full_title_name,
  short_title_name,
  short_title_value,
  full_title_value];
let file_alias = "";
for (let i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
};

const file_name = full_title_value;

//-------------------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET MENTAL HEALTH
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias_preset_mental}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar_value = include_arr[0];
const pillar_value_link = include_arr[1];

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
temp_file_path = `${sys_temp_include_dir}${related_project}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const project_value = include_arr[0];
const project_name = include_arr[1];
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);

//-------------------------------------------------------------------
// SET RELATED PARENT TASK
//-------------------------------------------------------------------
let parent_task_value = "null";
let parent_task_name = "Null";
if (project_value !== "null") {
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
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);

//-------------------------------------------------------------------  
// PDEV JOURNAL INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pdev_journal_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const journal_info = include_arr;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!vision]${space}${full_type_name}${space}Details${new_line}${call_start}${new_line}`;

const info = `${info_title}${journal_info}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = vis_three_yr_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
   await tp.file.move(`${directory}${file_name}`);
};

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
pillar: <%* tR += pillar_value_link %>
goal: <%* tR += goal %>
project: <%* tR += project_value_link %>
parent_task: <%* tR += parent_task_value_link %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_type_name %>

<%* tR += info %>

---

> [!question] Prompt
> 
> Write in the present tense as if it is three years in the future.
>  
> Where do I want to be and how do I want to feel in three years?

## Personal

- Growth
	- How do you want to grow and evolve as a person?
	- What are the personal qualities, traits, or values that you want to develop or enhance?
	- What are the personal passions, hobbies, or interests that you want to explore or pursue?
- Mental Health
	- How do you want to feel and think?
	- What are the sources of joy, peace, or satisfaction that you want to experience more of?
	- What are the coping strategies, practices, or resources that you want to use or access to improve your mental well-being and happiness?
- Physical Health
	- How do you want to look and perform?
	- What are the health goals or outcomes that you want to achieve or maintain?
	- What are the lifestyle changes or habits that you want to adopt or sustain to enhance your physical health and fitness?
- Education
	- How do you want to learn and grow?
	- What are the new skills, knowledge, or abilities that you want to learn or master?
	- What are the educational or academic goals or opportunities that you want to pursue or complete?
	- What are the benefits or rewards that you expect from advancing your education or career?
- Financial
	- How do you want to manage and use your money?
	- What are the financial goals or milestones that you want to achieve or maintain?
	- What are the financial habits or behaviors that you want to change or adopt to improve your financial security and freedom?

## Interpersonal

- Marriage
	- How do you want to relate and connect with your spouse?
	- What are the expectations, values, or principles that you and your spouse share or want to establish for your marriage?
	- What are the ways that you want to communicate, support, and respect each other?
- Family
	- How do you want to interact and bond with your parents and siblings?
	- How do you want to interact and bond with your nieces and nephews in the next five years?
	- What are the activities or events that you want to do or have with your family?
	- What are the memories or stories that you want to preserve or create with your family?
- Friends
	- How do you want to socialize and have fun with your friends?
	- Who are the friends that you want to keep or make in your life?
	- What are the qualities or characteristics that you look for or appreciate in your friends?

## Professional

- Career
	- How do you want to work and succeed?
	- What are the career aspirations, opportunities, or challenges that you want to pursue or face?
	- What are the skills, talents, or achievements that you want to showcase or improve in your field?