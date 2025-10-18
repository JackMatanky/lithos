<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const habit_ritual_proj_dir = "45_habit_ritual/";
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
const pillar_name_alias = "20_00_pillar_name_alias";
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";

const do_due_date = "40_task_do_due_date";
const task_status = "40_task_status";
const project_preview_review = "40_project_preview_review";
const project_info_callout = "40_00_project_info_callout";
const task_start_end_info_callout = "40_task_start_end_info_callout";

const related_task_sect_proj = "140_00_related_task_sect_proj";
const related_task_sect = "100_42_related_task_sect_task_file";
const related_dir_sect = "100_50_related_dir_sect";
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
const context_name = "Habits and Rituals";
const context_value = context_name.replaceAll(/s\sand\s/g, "_").replaceAll(/s$/g, "").toLowerCase();
const context_dir = habit_ritual_proj_dir;

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = "Project";
const type_value = type_name.toLowerCase();
const file_class = `task_${type_value}`;

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt(`Title for ${context_name} ${type_name}?`, null, true, false);
} else {
  title = tp.file.title;
};
title = title.trim();
title = await tp.user.title_case(title);

/* ---------------------------------------------------------- */
/*                   SET START AND END DATES                  */
/* ---------------------------------------------------------- */
const date_start = await tp.user.nl_date(tp, "start");
const date_start_link = `[[${date_start}]]`;
const date_start_value_link = `"${date_start_link}"`;
const date_end = await tp.user.nl_date(tp, "end");
const date_end_link = `[[${date_end}]]`;
const date_end_value_link = `"${date_end_link}"`;

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  "20_00_pillar_name_alias"
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
  `Goal for ${context_name} ${type_name}?`
);

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

//-------------------------------------------------------------------
// PROJECT TITLES, ALIAS, FILE NAME, AND DIRECTORY
//-------------------------------------------------------------------
const full_title_name = title;
const short_title_name = full_title_name.toLowerCase();
const short_title_value = short_title_name
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/'/g, "");

const alias_arr = [full_title_name, short_title_name, short_title_value];
let file_alias = "";
for (var i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
};

const file_name = short_title_value;
const file_section = `${file_name}${hash}`;

const project_dir = `${context_dir}${file_name}/`;

//-------------------------------------------------------------------
// PROJECT OBJECTIVE, PREVIEW, AND REVIEW CALLOUTS
//-------------------------------------------------------------------
heading = "Prepare and Reflect";
const head_preview_review = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_preview_review = `[[${file_section}${heading}\\|Preview and Review]]`;

temp_file_path = `${sys_temp_include_dir}${project_preview_review}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const project_preview_review_section = `${include_arr}${two_new_line}`;

//-------------------------------------------------------------------
// PROJECT TASKS AND EVENTS SECTION
//-------------------------------------------------------------------
heading = "Tasks and Events";
const head_task_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_task_sect = `[[${file_section}${heading}\\|Tasks]]`;

temp_file_path = `${sys_temp_include_dir}${related_task_sect_proj}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_proj_task_section = include_arr;

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION
//-------------------------------------------------------------------
heading = "Related Tasks and Events";
const head_related_task_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_related_task_sect = `[[${file_section}${heading}\\|Related Tasks]]`;

temp_file_path = `${sys_temp_include_dir}${related_task_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_task_section = include_arr;

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

//-------------------------------------------------------------------
// RELATED DIRECTORY SECTION
//-------------------------------------------------------------------
heading = "Related Directory";
const head_related_dir_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_related_dir_sect = `[[${file_section}${heading}\\|Directory]]`;

temp_file_path = `${sys_temp_include_dir}${related_dir_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_dir_section = include_arr;

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_dv_contents = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;
const toc_title = `${call_start}[!toc]${space}${toc_dv_contents}${two_space}${new_line}${call_start}${new_line}`;

const toc_body_high = `${call_tbl_start}${toc_preview_review}${tbl_pipe}${toc_task_sect}${tbl_pipe}${toc_related_task_sect}${call_tbl_end}${new_line}`;
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
const toc_body_low = `${call_tbl_start}${toc_related_pkm_sect}${tbl_pipe}${toc_related_lib_sect}${tbl_pipe}${toc_related_dir_sect}${call_tbl_end}`;
const toc_body = `${toc_body_high}${toc_body_div}${toc_body_low}`;

const toc = `${toc_title}${toc_body}${two_new_line}`;

//-------------------------------------------------------------------
// PROJECT FILE SECTIONS
//-------------------------------------------------------------------
const proj_preview_review = `${head_preview_review}${toc}${project_preview_review_section}${hr_line}${new_line}`;
const proj_task_section = `${head_task_sect}${toc}${related_proj_task_section}`;
const proj_related_task_section = `${head_related_task_sect}${toc}${related_task_section}`;
const proj_related_pkm_section = `${head_related_pkm_sect}${toc}${related_pkm_section}`;
const proj_related_lib_section = `${head_related_lib_sect}${toc}${related_lib_section}`;
const proj_related_dir_section = `${head_related_dir_sect}${toc}${related_dir_section}`;

//-------------------------------------------------------------------  
// PROJECT INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${project_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const project_info = include_arr;

//-------------------------------------------------------------------  
// TASK START AND END INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${task_start_end_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const task_start_end_info = `${call_start}${new_line}${include_arr}`;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${type_name}${space}Details${new_line}${call_start}${new_line}`;

const info = `${info_title}${project_info}${new_line}${task_start_end_info}${two_new_line}${hr_line}${new_line}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const folder_path = `${tp.file.folder(true)}/`;
const directory = project_dir;

if (folder_path!= directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
task_start: <%* tR += date_start_value_link %>
task_end: <%* tR += date_end_value_link %>
due_do: <%* tR += due_do_value %>
pillar: <%* tR += pillar_value_link %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
organization: <%* tR += organization_value_link %>
contact: <%* tR += contact_value_link %>
library:
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_title_name %>

> [!<%* tR += type_value %> ] <%* tR += type_name %> Details
> 
> - **Context**: `dv: choice(this.file.frontmatter.context = "habit_ritual", "Habits and Rituals", upper(substring(this.file.frontmatter.context, 0, 1)) + substring(this.file.frontmatter.context, 1))`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Goal**: `dv: this.file.frontmatter.goal`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) => !contains(lower(x), "null")), " | ")`
> 
> - **Start**:: <%* tR += date_start_link %>
> - **End**:: <%* tR += date_end_link %>

---

<%* tR += proj_preview_review %>
<%* tR += proj_task_section %>
<%* tR += proj_related_task_section %>
<%* tR += proj_related_pkm_section %>
<%* tR += proj_related_lib_section %>
<%* tR += proj_related_dir_section %>