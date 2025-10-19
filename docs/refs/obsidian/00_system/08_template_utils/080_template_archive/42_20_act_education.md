<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const projects_dir = "40_projects/";
const education_proj_dir = "42_education/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";

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
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";
const lib_name_alias = "60_library_content_file_name_alias";

const do_due_date = "40_task_do_due_date";
const task_status = "40_task_status";
const task_event_execution_plan = "42_task_event_plan";
const before_action_item_preview = "43_action_item_preview";
const after_action_item_review = "43_action_item_review";
const child_task_info_callout = "42_child_task_info_callout";

const related_sect_task_child = "142_00_related_sect_task_child";
const related_lib_sect = "100_60_related_lib_sect";
const related_pkm_sect = "100_70_related_pkm_sect";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Education";
const context_value = context_name.toLowerCase();
const context_dir = education_proj_dir;

//-------------------------------------------------------------------
// ACTION ITEM TAG, TYPE, AND FILE CLASS
//-------------------------------------------------------------------
const task_tag = "#task";
const type_name = "Action Item";
const type_value = type_name.replaceAll(/\s/g, "_").toLowerCase();
const file_class = "task_child";

//-------------------------------------------------------------------
// SET LIBRARY CONTENT FILE NAME AND ALIAS
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${lib_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const lib_resource_value = include_arr[0];
const lib_resource_name = include_arr[1];
const lib_resource_link = include_arr[2];

//-------------------------------------------------------------------
// LIBRARY CONTENT METADATA CACHE, TITLE, AND LINK
//-------------------------------------------------------------------
const lib_resource_tfile = tp.file.find_tfile(`${lib_resource_value}.md`);
const lib_resource_file_cache = await app.metadataCache.getFileCache(
  lib_resource_tfile
);
const lib_resource_file_class =
  lib_resource_file_cache?.frontmatter?.file_class;

//-------------------------------------------------------------------
// FILE TITLE AND TITLE VALUE
//-------------------------------------------------------------------
let title_prefix;
if (lib_resource_file_class.endsWith("video")) {
  title_prefix = "Watch";
} else if (lib_resource_file_class.endsWith("chapter")) {
  title_prefix = `Learn Chapter ${lib_resource_value.replace(
    /(\d\d).+/g,
    "$1"
)}`;
} else {
  title_prefix = "Read";
}

const title = `${title_prefix} ${lib_resource_name}`;
const title_value = `${title_prefix
  .replaceAll(/\s\d\d$/g, "")
  .replaceAll(/\s/g, "_")
  .toLowerCase()}_${lib_resource_value.replaceAll(/\s/g, "_")}`;

/* ---------------------------------------------------------- */
/*                      SET DATE AND TIME                     */
/* ---------------------------------------------------------- */
const date = await tp.user.nl_date(tp, "start");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const time = await tp.user.nl_time(tp, type_name);
const full_date_time = moment(`${date}T${time}`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

//-------------------------------------------------------------------
// SET TASK START AND REMINDER TIME
//-------------------------------------------------------------------
const start_time = moment(full_date_time).format("HH:mm");
const reminder_date = moment(full_date_time)
  .subtract(5, "minutes")
  .format("YYYY-MM-DD HH:mm");

//-------------------------------------------------------------------
// SET TASK DURATION AND END TIME
//-------------------------------------------------------------------
const duration_min = await tp.user.durationMin(tp);
const full_end_date = moment(full_date_time).add(
  Number(duration_min),
  "minutes"
);
const end_time = moment(full_end_date).format("HH:mm");
const duration_est = moment
  .duration(full_end_date.diff(full_date_time))
  .as("minutes");

/* ---------------------------------------------------------- */
/*   SET PILLAR FILE NAME AND TITLE; PRESET KNOW. EXPANSION   */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  "20_02_pillar_name_alias_preset_know"
);
const pillar_value = pillar_name_alias.split(";")[0];
const pillar_value_link = pillar_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, `${type_name} Goal?`);

/* ------------------- FILE PATH VARIABLES ------------------ */
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split(`/`);
const folder_path_length = folder_path_split.length;

/* ---------------------------------------------------------- */
/*      SET CONTEXT AND PROJECT BY FILE PATH OR SUGGESTER     */
/* ---------------------------------------------------------- */
let project_value;
let project_name;
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 3) {
  project_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[2],
    file_class: "task",
    type: "project",
  });
  project_value = project_obj[1].value;
  project_name = project_obj[1].key;
} else {
  project_obj_arr = await tp.user.file_name_alias_by_class_type_status({
    dir: context_dir,
    file_class: "task",
    type: "project",
    status: "active",
  });
  project_obj = await tp.system.suggester(
    (item) => item.key,
    project_obj_arr,
    false,
    "Project?"
  );
  project_value = project_obj.value;
  project_name = project_obj.key;
}
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);
const project_dir = `${context_dir}${project_value}`;

/* ---------------------------------------------------------- */
/*          SET PARENT TASK BY FILE PATH OR SUGGESTER         */
/* ---------------------------------------------------------- */
let parent_task_value;
let parent_task_name;
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 4) {
  parent_task_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[3],
    file_class: "task",
    type: "parent_task",
  });
  parent_task_value = parent_task_obj[1].value;
  parent_task_name = parent_task_obj[1].key;
} else {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_value,
    file_class: "task",
    type: "parent_task",
  });
  parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    "Parent Task?"
);
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
}
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);
const parent_task_dir = `${project_dir}/${parent_task_value}`;

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const org_name_alias = await tp.user.include_template(
  tp,
  "52_organization_name_alias"
);
const organization_value = org_name_alias.split(";")[0];
const organization_value_link = org_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
const contact_name_alias = await tp.user.include_template(
  tp,
  "51_contact_name_alias"
);
const contact_value = contact_name_alias.split(";")[0];
const contact_name = contact_name_alias.split(";")[1];
const contact_value_link = contact_name_alias.split(";")[2];

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const do_due_date = await tp.user.include_template(tp, "40_task_do_due_date");
const due_do_value = do_due_date.split(";")[0];
const due_do_name = do_due_date.split(";")[1];

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, "40_task_status");
const status_value = task_status.split(";")[0];
const status_name = task_status.split(";")[1];
const status_symbol = task_status.split(";")[2];

/* ---------------------------------------------------------- */
/*         FRONTMATTER TITLES, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
const full_title_name = `${short_date} ${title}`;
const short_title_name = `${title.toLowerCase()}`;
const short_title_value = title
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/[,']/g, "")
  .toLowerCase();
const full_title_value = `${short_date_value}_${short_title_value}`;

const alias_arr = [title, full_title_name, short_title_name, short_title_value, full_title_value];
let file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
};

const file_name = full_title_value;
const file_section = `${file_name}${hash}`;

//-------------------------------------------------------------------
// TASK CHECKBOX
//-------------------------------------------------------------------
heading = "Tasks and Events";
const head_task_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_task_sect = `[[${file_section}${heading}\\|Tasks]]`;

const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}${space}`;

const inline_time_start = `[time_start${dv_colon}${start_time}]`;
const inline_time_end = `[time_end${dv_colon}${end_time}]`;
const inline_duration_est = `[duration_est${dv_colon}${duration_est}]`;
const inline_metadata_time = `${space}${inline_time_start}${two_space}${inline_time_end}${two_space}${inline_duration_est}`;

const inline_reminder_date = `${space}⏰${space}${reminder_date}`;
const inline_creation_date = `${space}➕${space}${moment().format("YYYY-MM-DD")}`;
const inline_due_date = `${space}📅${space}${date}`;
const inline_metadata_date = `${inline_reminder_date}${inline_creation_date}${inline_due_date}`;

let task_checkbox;
if (status_value == "done") {
  task_checkbox = `${checkbox_task_tag}${title}_${type_value}${inline_metadata_time}${inline_metadata_date}${space}✅${space}${date}`;
} else {
  task_checkbox = `${checkbox_task_tag}${title}_${type_value}${inline_metadata_time}${inline_metadata_date}`;
}

//-------------------------------------------------------------------
// ACTION PREVIEW, PLAN, AND REVIEW
//-------------------------------------------------------------------
heading = "Prepare and Reflect";
const head_preview_review = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_preview_review = `[[${file_section}${heading}\\|Preview and Review]]`;

temp_file_path = `${sys_temp_include_dir}${before_action_item_preview}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const action_item_preview = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${task_event_execution_plan}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const execution_plan = `${include_arr}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${after_action_item_review}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const action_item_review = `${include_arr}${two_new_line}`;

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION FOR CHILD TASKS
//-------------------------------------------------------------------
heading = "Related Tasks and Events";
const head_related_task_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_related_task_sect = `[[${file_section}${heading}\\|Related Tasks]]`;

temp_file_path = `${sys_temp_include_dir}${related_sect_task_child}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_child_task_section = include_arr;

//-------------------------------------------------------------------
// RELATED PKM SECTION
//-------------------------------------------------------------------
heading = "Related Knowledge";
const head_related_pkm_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_related_pkm_sect = `[[${file_section}${heading}\\|PKM]]`;

temp_file_path = `${sys_temp_include_dir}${related_pkm_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_pkm_section = include_arr;

//-------------------------------------------------------------------
// RELATED LIBRARY SECTION
//-------------------------------------------------------------------
heading = "Related Library Content";
const head_related_lib_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_related_lib_sect = `[[${file_section}${heading}\\|Library]]`;

temp_file_path = `${sys_temp_include_dir}${related_lib_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_lib_section = include_arr;

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_dv_contents = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;
const toc_title = `${call_start}[!toc]${space}${toc_dv_contents}${two_space}${new_line}${call_start}${new_line}`;

const toc_body_high = `${call_tbl_start}${toc_task_sect}${tbl_pipe}${toc_preview_review}${tbl_pipe}${toc_related_task_sect}${tbl_pipe}${toc_related_pkm_sect}${tbl_pipe}${toc_related_lib_sect}${call_tbl_end}${new_line}`;
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const toc_body = `${toc_body_high}${toc_body_div}`;

const toc = `${toc_title}${toc_body}${two_new_line}`;

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const task_section = `${head_task_sect}${toc}${task_checkbox}${two_new_line}${hr_line}${new_line}`;
const preview_review = `${head_preview_review}${toc}${action_item_preview}${execution_plan}${action_item_review}${hr_line}${new_line}`;
const related_task = `${head_related_task_sect}${toc}${related_child_task_section}`;
const related_pkm = `${head_related_pkm_sect}${toc}${related_pkm_section}`;
const related_lib = `${head_related_lib_sect}${toc}${related_lib_section}`;

//-------------------------------------------------------------------
// CHILD TASK INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${child_task_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const child_task_info = include_arr;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${type_name}${space}Details${new_line}${call_start}${new_line}`;

const dv_date = `${backtick}dv:${space}this.file.frontmatter.date${backtick}`;
const info_date = `${call_start}Date::${space}${dv_date}${two_space}`;

const info_body = `${child_task_info}${new_line}${info_date}`;

const info = `${info_title}${info_body}${two_new_line}${hr_line}${new_line}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
let directory;
if (parent_task_value == "null") {
  directory = `${project_dir}/`;
} else {
  directory = `${project_dir}/${parent_task_value}/`;
}

if (folder_path!= directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
due_do: <%* tR += due_do_value %>
pillar: <%* tR += pillar_value_link %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
project: <%* tR += project_value_link %>
parent_task: <%* tR += parent_task_value_link %>
organization: <%* tR += organization_value_link %>
contact: <%* tR += contact_value_link %>
library:
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title %>

<%* tR += info %>
<%* tR += task_section %>
<%* tR += preview_review %>
<%* tR += related_task %>
<%* tR += related_pkm %>
<%* tR += related_lib %>
