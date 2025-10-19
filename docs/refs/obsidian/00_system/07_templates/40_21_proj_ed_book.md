<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
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

const regex_link_to_value = /^(\[\[)|(\|.+)/g;

const dv_yaml = "file.frontmatter";
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Education";
const context_value = context_name.toLowerCase();
const context_dir = education_proj_dir;

//-------------------------------------------------------------------
// PROJECT TASK TYPE AND FILE CLASS
//-------------------------------------------------------------------
const proj_type_name = "Project";
const proj_type_value = proj_type_name.toLowerCase();
const proj_file_class = `task_${proj_type_value}`;

/* ---------------------------------------------------------- */
/*                  PROJECT SETUP PARENT TASK                 */
/* ---------------------------------------------------------- */

/* ------------------- FILE TYPE AND CLASS ------------------ */
const parent_type_name = "Parent Task";
const parent_type_value = parent_type_name.replace(/\s/g, "_").toLowerCase();
const parent_file_class = `task_${parent_type_value.split("_")[0]}`;

//-------------------------------------------------------------------
// SET BOOK FILE NAME AND TITLE
//-------------------------------------------------------------------
const book_name_alias = await tp.user.include_template(
  tp,
  "61_book_name_alias"
);
const book_value = book_name_alias.split(";")[0];
const book_name = book_name_alias.split(";")[1];
const book_dir = `${lib_books_dir}${book_value}/`;

//-------------------------------------------------------------------
// BOOK METADATA CACHE, TITLE, AND LINK
//-------------------------------------------------------------------
const book_file_path = `${book_dir}${book_value}.md`;
const book_tfile = await app.vault.getAbstractFileByPath(book_file_path);
const book_file_cache = await app.metadataCache.getFileCache(book_tfile);
const book_main_title = book_file_cache?.frontmatter?.main_title;

const book_value_link = yaml_li(`[[${book_value}|${book_main_title}]]`);

//-------------------------------------------------------------------
// BOOK PUBLISHED DATE
//-------------------------------------------------------------------
const date_published = book_file_cache?.frontmatter?.date_published;

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
const { value: pillar_value, link: pillar_yaml } =
  await tp.user.multi_suggester({
    tp,
    items: await tp.user.file_by_status({
      dir: pillars_dir,
      status: "active",
    }),
    type: "pillar",
    context: context_value,
  });

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, `Goal?`);

//-------------------------------------------------------------------
// PUBLISHER ORGANIZATION FILE NAME, TITLE, AND LINK
//-------------------------------------------------------------------
const publisher_yaml = book_file_cache?.frontmatter?.publisher;

let publisher_value_link = null_yaml_li;
if (
  !null_arr.includes(publisher_yaml) &&
  typeof publisher_yaml != "undefined"
) {
  publisher_arr = publisher_yaml.toString().split(",");
  publisher_value_link = publisher_arr
    .map((publisher) => yaml_li(publisher))
    .join("");
};

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
let { value: organization_value, link: organization_yaml } =
  await tp.user.multi_suggester({
    tp,
    items: await tp.user.md_file_name_alias(organizations_dir),
    type: "organization",
  });

if (organization_value != "null") {
  organization_yaml = `${organization_yaml}${publisher_value_link}`;
} else {
  organization_yaml = `${publisher_value_link}`;
}

//-------------------------------------------------------------------
// AUTHOR CONTACT FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const author_yaml = book_file_cache?.frontmatter?.author;

let author_value_link = null_yaml_li;
if (
  !null_arr.includes(author_yaml) &&
  typeof author_yaml != "undefined"
) {
  author_arr = author_yaml.toString().split(",");
  author_value_link = author_arr
    .map((author) => yaml_li(author))
    .join("");
}

const author_file_name_arr = author_yaml.map((file) =>
  file.toString().replaceAll(regex_link_to_value, "")
);
if (author_file_name_arr.length >= 3 || author_file_name_arr.length == 1) {
  author_file_name = author_file_name_arr[0];
  author_file_path = `${contacts_dir}${author_file_name}.md`;
  author_tfile = await app.vault.getAbstractFileByPath(author_file_path);
  author_file_cache = await app.metadataCache.getFileCache(author_tfile);
  if (author_file_name_arr.length == 1) {
    author_name_last = author_file_cache?.frontmatter?.name_last;
  } else {
    author_name_last = `${author_file_cache?.frontmatter?.name_last} et al`;
  }
} else if (author_file_name_arr.length >= 2) {
  author1_file_name = author_file_name_arr[0];
  author1_file_path = `${contacts_dir}${author1_file_name}.md`;
  author1_tfile = await app.vault.getAbstractFileByPath(author1_file_path);
  author1_file_cache = await app.metadataCache.getFileCache(author1_tfile);
  author1_last_name = author1_file_cache?.frontmatter?.name_last;
  author2_file_name = author_file_name_arr[1];
  author2_file_path = `${contacts_dir}${author2_file_name}.md`;
  author2_tfile = await app.vault.getAbstractFileByPath(author2_file_path);
  author2_file_cache = await app.metadataCache.getFileCache(author2_tfile);
  author2_last_name = author2_file_cache?.frontmatter?.name_last;
  author_name_last = `${author1_last_name}'s and ${author2_last_name}`;
}

const author_last_name = author_name_last;
const author_last_name_value = author_last_name
  .replace(/'s\sand/g, "")
  .replaceAll(/\s/g, "_");

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
let {
  value: contact_value,
  name: contact_name,
  link: contact_yaml,
} = await tp.user.multi_suggester({
  tp,
  items: await tp.user.md_file_name_alias(contacts_dir),
  type: "contact",
});

if (contact_value != "null") {
  contact_yaml = `${contact_yaml}${author_value_link}`;
} else {
  contact_yaml = author_value_link;
}

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
const full_title_name = `Read ${author_last_name}'s ${book_main_title}`;
const short_title_name = full_title_name.toLowerCase();
const full_title_value = `Read_${author_last_name_value}_${date_published}_${book_main_title}`;
const short_title_value = `read_${book_value}`;

const file_alias =
  new_line +
  [full_title_name, short_title_name, full_title_value, short_title_value]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

const file_name = short_title_value;
const file_section = file_name + hash;
const file_value_link = yaml_li(`[[${file_name}|${full_title_name}]]`);

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
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info = await tp.user.include_file("40_21_proj_ed_book_info_callout");

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = project_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

/* -- FRONTMATTER TITLE, ALIASES, FILE NAME, AND DIRECTORY -- */
const par_full_title_name = "Project Setup for " + full_title_name;
const par_short_title_name = par_full_title_name.toLowerCase();
const par_full_title_value = par_short_title_name
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/'/g, "");
const par_short_title_value =
  "proj_setup_" +
  file_name.replaceAll(/[\s-]/g, "_").replaceAll(/'/g, "").toLowerCase();

const par_file_alias =
  new_line +
  [
    par_full_title_name,
    par_short_title_name,
    par_full_title_value,
    par_short_title_value,
  ]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

const par_file_name = par_short_title_value;
const par_file_section = par_file_name + hash;

const par_dir = `${project_dir}${par_file_name}`;
const par_file_path = `${par_dir}/${par_file_name}.md`;

/* ------------ PARENT TASK FILE DETAILS CALLOUT ------------ */
const par_info = await tp.user.include_file(
  "41_21_par_ed_book_ch_info_callout"
);

/* --------- PARENT TASK PREVIEW AND REVIEW SECTION --------- */
section_obj_arr[0].content = await tp.user.include_template(
  tp,
  "41_parent_task_preview_review"
);

/* ---------- PARENT TASK TASKS AND EVENTS SECTION ---------- */
section_obj_arr[1].content = await tp.user.include_template(
  tp,
  "141_00_related_task_sect_parent"
);

/* ---------- PARENT TASK TABLE OF CONTENTS CALLOUT --------- */
const par_toc = toc.replaceAll(file_name, par_file_name);

/* ---------------- PARENT TASK FILE SECTIONS --------------- */
const par_sections_content = section_obj_arr
  .map((s) => [s.head, par_toc, s.content].join(two_new_line))
  .join(new_line);

/* --------- PARENT TASK FRONTMATTER YAML PROPERTIES -------- */
const par_yaml = [
  hr_line,
  `title:${space}${par_file_name}`,
  `uuid:${space}${await tp.user.uuid()}`,
  `aliases:${space}${par_file_alias}`,
  `task_start:${space}${task_start_link}`,
  "task_end:",
  `due_do:${space}do`,
  `pillar:${space}${pillar_yaml}`,
  `context:${space}${context_value}`,
  `goal:${space}${goal}`,
  `project:${file_value_link}`,
  `organization:${organization_yaml}`,
  `contact:${contact_yaml}`,
  `library:`,
  `status:${space}to_do`,
  `type:${space}${parent_type_value}`,
  `file_class:${space}${parent_file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  "tags:",
  hr_line,
].join(new_line);

/* ---------------- PARENT TASK FILE CONTENT ---------------- */
const par_file_content = [
  par_yaml,
  head_lvl(1) + par_full_title_name + new_line,
  par_info,
  par_sections_content,
].join(new_line);

/* ------ PARENT TASK DIRECTORY AND FILE PATH CREATION ------ */
await this.app.vault.createFolder(par_dir);
await this.app.vault.create(par_file_path, par_file_content);

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
task_start: <%* tR += task_start_link %>
task_end: <%* tR += task_end_link %>
due_do: <%* tR += due_do_value %>
pillar: <%* tR += pillar_yaml %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
organization: <%* tR += organization_yaml %>
contact: <%* tR += contact_yaml %>
library: <%* tR += book_value_link %>
status: <%* tR += status_value %>
type: <%* tR += proj_type_value %>
file_class: <%* tR += proj_file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_title_name %>

<%* tR += info %>
<%* tR += sections_content %>
