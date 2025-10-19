<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const work_proj_dir = "44_work/";
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

const dv_yaml = "file.frontmatter";
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Work";
const context_value = context_name.toLowerCase();
const context_dir = work_proj_dir;

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
const task_start = await tp.user.nl_date(tp, "start");
const task_start_link = `"[[${task_start}]]"`;
const task_end = await tp.user.nl_date(tp, "end");
const task_end_link = `"[[${task_end}]]"`;

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
for (let i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
}

const file_name = short_title_value;
const file_section = `${file_name}${hash}`;

const project_dir = `${context_dir}${file_name}/`;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: "Prepare and Reflect",
    toc_level: 1,
    toc_key: "Insight",
    file: "40_project_preview_review",
  },
  {
    head_key: "Tasks and Events",
    toc_level: 1,
    toc_key: "Tasks and Events",
    file: "140_00_related_task_sect_proj",
  },
  {
    head_key: "Related Tasks and Events",
    toc_level: 1,
    toc_key: "Related Tasks",
    file: "100_42_related_task_sect_task_file",
  },
  {
    head_key: "Related Knowledge",
    toc_level: 2,
    toc_key: "PKM",
    file: "100_70_related_pkm_sect",
  },
  {
    head_key: "Related Library Content",
    toc_level: 2,
    toc_key: "Library",
    file: "100_60_related_lib_sect",
  },
  {
    head_key: "Related Directory",
    toc_level: 2,
    toc_key: "Directory",
    file: "100_50_related_dir_sect",
  },
];

// Content, heading, and table of contents link
for (let i = 0; i < section_obj_arr.length; i++) {
  section_obj_arr[i].content = await tp.user.include_template(
    tp,
    section_obj_arr[i].file
  );
}
section_obj_arr.map((x) => (x.head = head_lvl(2) + x.head_key));
section_obj_arr.map(
  (x) => (x.toc = `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
);

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_title = `${call_start}[!toc]${space}${dv_content_link}`;

const toc_lvl = (int) =>
  call_tbl_start +
  section_obj_arr
    .filter((x) => x.toc_level == int)
    .map((x) => x.toc)
    .join(tbl_pipe) +
  call_tbl_end;

const toc_body_div =
  call_tbl_start + Array(3).fill(tbl_cent).join(tbl_pipe) + call_tbl_end;

const toc = [toc_title, call_start, toc_lvl(1), toc_body_div, toc_lvl(2)].join(
  new_line
);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => `${s.head}${toc}${s.content}`)
  .join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info = await tp.user.include_file("40_00_project_info_callout");

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = project_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
task_start: <%* tR += task_start_link %>
task_end: <%* tR += task_end_link %>
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

<%* tR += info %>
<%* tR += sections_content %>
