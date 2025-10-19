<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";
const library_dir = "60_library/";
const pkm_dir = "70_pkm/";
const tree_category_dir = "70_pkm/01_category/";
const tree_branch_dir = "70_pkm/02_branch/";
const tree_field_dir = "70_pkm/03_field/";
const tree_subject_dir = "70_pkm/04_subject/";

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
const related_task_sect_related_project = "100_41_related_task_sect_related_proj";
const related_dir_sect = "100_50_related_dir_sect";
const related_lib_sect_file = "100_61_related_lib_sect_related_file";
const related_pkm_lab_sect = "100_70_related_pkm_lab_sect";
const note_status = "80_note_status";
const note_idea_compass = "80_note_idea_compass";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// PKM TREE BUTTON
//-------------------------------------------------------------------
const button_start = `${three_backtick}button${new_line}`;
const button_end = `${three_backtick}${two_new_line}`;
const button_comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}${two_new_line}`;

// KNOWLEDGE OBJECT INFORMATION BUTTONS
button_name = `name 🗃️PKM Tree Info${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 170_00_dvmd_pkm_tree_info${new_line}`;
button_replace = `replace [54, 86]${new_line}`;
button_color = `color purple${new_line}`;

const pkm_info_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

// KNOWLEDGE CONTEXT BUTTON
button_name = `name 🗃️PKM Tree Context${new_line}`;
button_type = `type append template${new_line}`;
button_action = `action 170_01_dvmd_pkm_tree_context${new_line}`;
button_replace = `replace [103, 209]${new_line}`;
button_color = `color purple${new_line}`;

const pkm_context_button = `${button_start}${button_name}${button_type}${button_action}${button_replace}${button_color}${button_end}${button_comment}`;

//-------------------------------------------------------------------
// TYPE, SUBTYPE, AND FILE CLASS
//-------------------------------------------------------------------
const type_name = "Knowledge Tree";
const type_value = type_name.split(" ")[1].toLowerCase();
const subtype_name = "Category";
const subtype_value = subtype_name.toLowerCase();
const file_class = "pkm_tree";

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt(`Title for ${type_name} ${subtype_name}`, null, true, false);
} else {
  title = tp.file.title;
}
title = title.trim();
title = await tp.user.title_case(title);

/* ---------------------------------------------------------- */
/*          FRONTMATTER TITLE, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
const full_title_name = title;
const short_title_name = title.toLowerCase();
const short_title_value = short_title_name.replaceAll(/\s/g, "_");

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}${short_title_value}`;

const file_name = short_title_value;

/* ---------------------------------------------------------- */
/*                       SET DESCRIPTION                      */
/* ---------------------------------------------------------- */
const about = await tp.system.prompt(
  `${subtype_name} Description?`,
  null,
  false,
  true
);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, "$2")
  .replaceAll(/(\s*)\n/g, "\n")
  .replaceAll(/([^\s])(\s*)$/g, "$1")
  .replaceAll(/\n{1,6}/g, "<new_line>")
  .replaceAll(/(<new_line>)(\w)/g, "\n \n $2")
  .replaceAll(/(<new_line>)(-\s|\d\.\s)/g, "\n $2");

/* ---------------------------------------------------------- */
/*                      SET REFERENCE URL                     */
/* ---------------------------------------------------------- */
const url = await tp.system.prompt(`${subtype_name} Reference URL?`);

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
/*                      SET RELATED GOAL                      */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${type_name} ${subtype_name}?`
);

/* ---------------------------------------------------------- */
/*                       SET NOTE STATUS                      */
/* ---------------------------------------------------------- */
const note_status = await tp.user.include_template(tp, "80_note_status");
const status_name = note_status.split(";")[0];
const status_value = note_status.split(";")[1];

//-------------------------------------------------------------------
// NOTE IDEA COMPASS
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${note_idea_compass}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const idea_compass = include_arr;

//-------------------------------------------------------------------
// RELATED PKM TREE SECTION
//-------------------------------------------------------------------
const pkm_category_child_branch = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_branch",
  md: "false",
})

const pkm_category_child_field = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_field",
  md: "false",
})

const pkm_category_child_subject = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_subject",
  md: "false",
})

const pkm_category_child_topic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_topic",
  md: "false",
})

//-------------------------------------------------------------------
// RELATED NOTE SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_pkm_lab_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_note_section = include_arr;

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION WITH PROJECT FILE LINK
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_task_sect_related_project}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const related_task_event_section = include_arr;

//-------------------------------------------------------------------
// RELATED LIBRARY SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_lib_sect_file}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const related_library_section = include_arr;

//-------------------------------------------------------------------
// RELATED DIRECTORY SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_dir_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const related_directory_section = include_arr;

//-------------------------------------------------------------------
// FILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const file_section = `${file_name}#`;
const tbl_pipe =`${space}|${space}`;
const tbl_pipe_end =`${space}|`;

const toc_title = `>${space}[!toc]${space}[[${file_section}${full_title_name}|Contents]]${two_space}

> ${space}`;
const toc_tree_info = `[[${file_section}Knowledge Tree Information\\|PKM Tree Info]]`;
const toc_related_tree = `[[${file_section}Knowledge Context\\|PKM Context]]`;
const toc_notes = `[[${file_section}Related Notes\\|Notes]]`;
const toc_library = `[[${file_section}Related Library Content\\|Library]]`;
const toc_pillar_goal = `[[${file_section}Related Pillars and Goals\\|Pillars and Goals]]`;
const toc_task_event = `[[${file_section}Related Tasks and Events\\|Tasks and Events]]`;
const toc_directory = `[[${file_section}Related Directory\\|Directory]]`;

const toc_main_headings = `>${tbl_pipe}Page Sections${tbl_pipe}${toc_tree_info}${tbl_pipe}${toc_related_tree}${tbl_pipe}${toc_notes}${tbl_pipe_end}${two_space}

> ${tbl_pipe}:-------:${tbl_pipe}:-------:${tbl_pipe}:-------:${tbl_pipe}:-------:${tbl_pipe_end}${two_space}
> ${tbl_pipe}${toc_library}${tbl_pipe}${toc_pillar_goal}${tbl_pipe}${toc_task_event}${tbl_pipe}${toc_directory}${tbl_pipe_end}${two_space}`;

const toc_callout = `${toc_title}
${toc_main_headings}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = tree_category_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += "---";
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
pillar: <%* tR += pillar_value_link %>
category:
branch:
field:
subject:
topic:
subtopic:
about: |
 <%* tR += about_value %>
url: <%* tR += url %>
status: <%* tR += status_value %>
subtype: <%* tR += subtype_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>

# <%* tR += full_title_name %>

> [!<%* tR += type_value %> ] <%* tR += type_name %> Details
>
> - **Name**: `dv: choice(regextest("\w", this.file.frontmatter.url), elink(this.file.frontmatter.url, this.file.frontmatter.aliases[0]), this.file.frontmatter.aliases[0])`
>
> - **Description**: `dv: this.file.frontmatter.about`

---

## <%* tR += type_name %> Information

<%* tR += toc_callout %>

### Branches

<!-- Link the category's branches here  -->

<%* tR += pkm_category_child_branch %>

### Fields

<!-- Link the category's fields here  -->

<%* tR += pkm_category_child_field %>

### Subjects

<!-- Link the category's subjects here  -->

<%* tR += pkm_category_child_subject %>

### Topics

<!-- Link the category's fields here  -->

<%* tR += pkm_category_child_topic %>

### Key Terms

<!-- Link the category's key terms here  -->

### General Information

<!-- Link the category's key terms here  -->

---

## Related Notes

<%* tR += toc_callout %>

<%* tR += related_note_section %>

## Related Library Content

<%* tR += toc_callout %>

<%* tR += related_library_section %>

## Related Pillars and Goals

<%* tR += toc_callout %>

### Outgoing Pillars and Goals Links

<!-- Link related pillars and goals here  -->

- <%* tR += pillar_link %>
- <%* tR += goal %>

### Pillars

### Value Goals

### Outcome Goals

---

## Related Tasks and Events

<%* tR += toc_callout %>

<%* tR += related_task_event_section %>

## Related Directory

<%* tR += toc_callout %>

<%* tR += related_directory_section %>

## Flashcards
