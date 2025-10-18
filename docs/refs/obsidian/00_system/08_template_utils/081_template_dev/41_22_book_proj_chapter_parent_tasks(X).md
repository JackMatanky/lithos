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
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const pillar_name_alias = "20_00_pillar_name_alias";
const do_due_date = "40_task_do_due_date";
const task_status = "40_task_status";
const parent_task_preview_review = "41_parent_task_preview_review";
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// SPECIAL CHARACTERS
//-------------------------------------------------------------------
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const percent = String.fromCodePoint(37);
const comment = percent.repeat(2);
const cmnt_ob_start = `${comment}${space}`;
const cmnt_ob_end = `${space}${comment}`;

//-------------------------------------------------------------------
// PILLAR, ORGANIZATION, CONTACT, AND GOAL OBJECT ARRAYS
//-------------------------------------------------------------------
const pillars_obj_arr = await tp.user.file_by_status({
  dir: pillars_dir,
  status: `active`,
});
const organizations_obj_arr = await tp.user.md_file_name_alias(
  organizations_dir
);
const contact_obj_arr = await tp.user.md_file_name_alias(contacts_dir);
const goals = await tp.user.md_file_name(goals_dir);

//-------------------------------------------------------------------
// DATAVIEW TASK TABLES
//-------------------------------------------------------------------
// TYPE: "parent", "child"; STATUS: "due", "done", "null"
const child_task_remaining = await tp.user.dv_proj_task("child", "due");
const child_task_completed = await tp.user.dv_proj_task("child", "done");

//-------------------------------------------------------------------
// LINKED FILES DATAVIEW TABLE
//-------------------------------------------------------------------
// RELATED TASK TABLES
// TASK TYPE: "project", "parent_task", "child_task"
const linked_projects = await tp.user.dv_linked_file("task", "project");
const linked_parent_tasks = await tp.user.dv_linked_file("task", "parent");
const linked_child_tasks = await tp.user.dv_linked_file("task", "child");

// RELATED NOTE TABLES
// PKM TYPE: "permanent", "literature", "fleeting", "info"
const linked_note_permanent = await tp.user.dv_linked_file("pkm", "perm");
const linked_note_lit = await tp.user.dv_linked_file("pkm", "lit");
const linked_note_fleet = await tp.user.dv_linked_file("pkm", "fleet");
const linked_note_info = await tp.user.dv_linked_file("pkm", "info");

// RELATED LIBRARY TABLES
const linked_lib_content = await tp.user.dv_linked_file("lib", "");

// RELATED DIRECTORY TABLES
// DIR TYPE: "contact", "organization"
const linked_dir_contact = await tp.user.dv_linked_file("dir", "contact");
const linked_dir_org = await tp.user.dv_linked_file("dir", "organization");

//-------------------------------------------------------------------
// KNOWLEDGE EXPANSION PILLAR FILE AND FULL NAME
//-------------------------------------------------------------------
const know_pillar_name = "Knowledge Expansion";
const know_pillar_value = know_pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const know_pillar_link = `[[${know_pillar_value}|${know_pillar_name}]]`;

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Education";
const context_value = context_name.toLowerCase();
const context_dir = education_proj_dir;

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = "Parent Task";
const type_value = type_name.replaceAll(/\s/g, "_").toLowerCase();
const file_class = `task_${type_value}`;

//-------------------------------------------------------------------
// SET PROJECT NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const project_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: context_dir,
  file_class: "task",
  type: "project",
});
const project_obj = await tp.system.suggester(
  (item) => item.key,
  project_obj_arr,
  false,
  "Project?"
);
const project_value = project_obj.value;
const project_name = project_obj.key;
const project_link = `${project_value}|${project_name}`;
const project_value_link = yaml_li(project_link);
const project_dir = `${context_dir}${project_value}/`;

//-------------------------------------------------------------------
// BOOK METADATA CACHE, TITLE, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const book_value = project_value.slice(6);
const book_tfile = tp.file.find_tfile(`${book_value}.md`);
const book_file_cache = await app.metadataCache.getFileCache(book_tfile);
const book_main_title = book_file_cache?.frontmatter?.title;
const book_main_title_value = book_main_title
  .replaceAll(/\s/g, "_")
  .toLowerCase();

const book_link = `${book_value}|${book_main_title}`;
const book_dir = `${lib_books_dir}${book_value}/`;

//-------------------------------------------------------------------
// BOOK PUBLISHED DATE
//-------------------------------------------------------------------
const date_published = book_file_cache?.frontmatter?.date_published;

//-------------------------------------------------------------------
// BOOK CHAPTERS OBJECT ARRAY
//-------------------------------------------------------------------
const chapter_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: book_dir,
  file_class: "lib",
  type: "book_chapter",
});

//-------------------------------------------------------------------
// PARENT TASK INFO CALLOUT VARIABLES
//--------------------------------------------------------
const info_callout_title_context = `>${space}[!${type_value}${space}${type_name}${space}Details${two_space}

> ${space}
> ${space}Context::${space}${context_name}${two_space}`;

let info_callout_pillar = `>${space}Pillar::${space}`;

const info_callout_goal_proj = `>${space}Goal::${two_space}

> ${space}Project::${space}${project_link}${two_space}`;

let info_callout_org = `>${space}Organization::${space}`;

let info_callout_contact = `>${space}Contact::${space}`;

const info_callout_book = `>${space}

> ${space}Book::${space}${book_link}${two_space}`;

let info_callout_chapter = `>${space}Chapter::${space}`;

let info_callout_start_end_date = `>${space}

> ${space}|${space}Start Date${space}|${space}End Date${space}|${two_space}
> ${space}|${space}:--------:${space}|${space}:--------:${space}|${two_space}
> ${space}`;

const info_callout = `${info_callout_title_context}
${info_callout_pillar}
${info_callout_goal_proj}
${info_callout_org}
${info_callout_contact}
${info_callout_book}
${info_callout_chapter}
${info_callout_start_end_date}`;

/* --------- PARENT TASK PREVIEW AND REVIEW SECTION --------- */
temp_file_path = `${sys_temp_include_dir}${parent_task_preview_review}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const parent_task_preview_review = include_arr;

//-------------------------------------------------------------------
// FRONTMATTER VARIABLES
//-------------------------------------------------------------------
let fmatter_title = `title:${space}`;
let fmatter_alias = `aliases:${space}`;
let fmatter_date_start = `date_start:${space}`;
let fmatter_date_end = `date_end:${space}`;
let fmatter_due_do = "due_do: do";
let fmatter_pillar = `pillar:${space}`;
let fmatter_context = `context:${space}${context_value}`;
let fmatter_goal = `goal:${space}`;
let fmatter_project = `project:${space}${project_value}`;
let fmatter_organization = `organization:${space}`;
let fmatter_contact = `contact:${space}`;
let fmatter_status = "status: to_do";
let fmatter_type = "type: parent_task";
let fmatter_file_class = `file_class:${space}${file_class}`;
let fmatter_date_created = `date_created:${space}${date_created}`;
let fmatter_date_modified = `date_modified:${space}${date_modified}`;
let fmatter_tags = `tags:${space}`;

//-------------------------------------------------------------------
// TP.CREATE_NEW VARIABLES
//-------------------------------------------------------------------
let file_name;
let file_content;
let directory;

//-------------------------------------------------------------------
// LOOP THROUGH ARRAY OF OBJECTS
//-------------------------------------------------------------------
for (var i = 1; i < chapter_obj_arr.length; i++) {
  // CHAPTER METADATA CACHE
  chapter_tfile = tp.file.find_tfile(`${chapter_obj_arr[i].value}.md`);
  chapter_file_cache = await app.metadataCache.getFileCache(chapter_tfile);
  chapter_main_title = chapter_file_cache?.frontmatter?.title;
  chapter_main_title_value = chapter_main_title
  .replaceAll(/\s/g, "_")
  .toLowerCase();

  chapter_link = `${chapter_obj_arr[i].value}|${chapter_obj_arr[i].key}`;
  info_callout_chapter = `${info_callout_chapter}${chapter_link}${two_space}`;

  // PUBLISHER ORGANIZATION FILE NAME, TITLE, AND LINK
  publisher_value = chapter_file_cache?.frontmatter?.publisher;
  publisher_tfile = tp.file.find_tfile(`${publisher_value}.md`);
  publisher_file_cache = await app.metadataCache.getFileCache(publisher_tfile);
  publisher_name = publisher_file_cache?.frontmatter?.aliases[0];
  publisher_link = `[[${publisher_value}|${publisher_name}]]`;

  // AUTHOR CONTACT FILE NAME, TITLE, AND LINK
  author_value = chapter_file_cache?.frontmatter?.author;
  author_tfile = tp.file.find_tfile(`${author_value}.md`);
  author_file_cache = await app.metadataCache.getFileCache(author_tfile);
  author_name = author_file_cache?.frontmatter?.aliases[0];
  author_name_last = author_file_cache?.frontmatter?.name_last;
  author_link = `[[${author_value}|${author_name}]]`;

  // TITLES, ALIAS, AND FILE NAME
  // Titles
  full_title_name = `Learn ${author_name_last}'s ${book_main_title}: ${chapter_main_title}`;
  short_title_name = full_title_name.toLowerCase();
  full_title_value = `Learn_${author_name_last}_${date_published}_${book_main_title}_${chapter_main_title}`;
  short_title_value = `learn_${chapter_obj_arr[i].value}`;
  // Alias
  alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}"${full_title_value}"${ul_yaml}${short_title_value}`;

  // File name
  file_name = short_title_value;

  fmatter_title = `${fmatter_title}${file_name}`;
  fmatter_alias = `${fmatter_alias}${alias_arr}`;

  // SET START AND END DATES
  date_start = await tp.user.nl_date(tp, "start");
  date_end = await tp.user.nl_date(tp, "end");
  fmatter_date_start = `${fmatter_date_start}${date_start}`;
  fmatter_date_end = `${fmatter_date_end}${date_end}`;
  info_callout_start_end_date = `${info_callout_start_end_date}|${space}[[date_start]]${space}|${space}[[date_end]]${space}|${two_space}`;

  // SET PILLAR FILE AND FULL NAME
  pillar_obj = await tp.system.suggester(
    (item) => item.key,
    pillars_obj_arr,
    false,
    `Pillar for ${full_title_name}?`
);
  pillar_value = pillar_obj.value;
  pillar_name = pillar_obj.key;
  pillar_link = `[[${pillar_value}|${pillar_name}]]`;

  if (pillar_value!= "null") {
    pillar_value = `[${pillar_value}, ${know_pillar_value}]`;
    pillar_link = `${pillar_link}, ${know_pillar_link}`;
  } else {
    pillar_value = `${know_pillar_value}`;
    pillar_link = `${know_pillar_link}`;
  }

  fmatter_pillar = `${fmatter_pillar}${pillar_value}`;
  info_callout_pillar = `${info_callout_pillar}${pillar_link}${two_space}`;

  // SET GOAL
  goal = await tp.system.suggester(
    goals,
    goals,
    false,
    `Goal for ${full_title_name}?`
);
  fmatter_goal = `${fmatter_goal}${goal}`;

  // SET ORGANIZATION FILE NAME AND TITLE
  organizations_obj = await tp.system.suggester(
    (item) => item.key,
    organizations_obj_arr,
    false,
    `Organization for ${full_title_name}?`
);

  organization_value = organizations_obj.value;
  organization_name = organizations_obj.key;

  if (organization_value.includes(`_user_input`)) {
    organization_name = await tp.system.prompt(
      `Organization for ${full_title_name}?`,
      ``,
      false,
      false
);
    organization_value = organization_name
  .replaceAll(/[,']/g, "")
  .replaceAll(/\s/g, "_")
  .replaceAll(/\//g, "-")
  .replaceAll(/&/g, "and")
  .toLowerCase();
  }
  organization_link = `[[${organization_value}|${organization_name}]]`;

  if (organization_value!= "null") {
    organization_value = `[${organization_value}, ${publisher_value}]`;
    organization_link = `${organization_link}, ${publisher_link}`;
  } else {
    organization_value = `${publisher_value}`;
    organization_link = `${publisher_link}`;
  }

  fmatter_organization = `${fmatter_organization}${organization_value}`;
  info_callout_org = `${info_callout_org}${organization_link}${two_space}`;

  // SET CONTACT FILE NAME AND TITLE
  contact_obj = await tp.system.suggester(
    (item) => item.key,
    contact_obj_arr,
    false,
    `Contact for ${full_title_name}?`
);

  contact_value = contact_obj.value;
  contact_name = contact_obj.key;
  if (contact_value.includes(`_user_input`)) {
    const contact_names = await tp.user.dirContactNames(tp);
    const full_name = contact_names.full_name;
    const last_first_name = contact_names.last_first_name;
    contact_name = full_name;
    contact_value = last_first_name
  .replaceAll(/,/g, "*")
  .replaceAll(/[^\w]/g, "*")
  .toLowerCase();
  }
  contact_link = `[[${contact_value}|${contact_name}]]`;

  if (contact_value!= "null") {
    contact_value = `[${contact_value}, ${author_value}]`;
    contact_link = `${contact_link}, ${author_link}`;
  } else {
    contact_value = `${author_value}`;
    contact_link = `${author_link}`;
  }

  fmatter_contact = `${fmatter_contact}${contact_value}`;
  info_callout_contact = `${info_callout_contact}${contact_link}`;

  // INFO, PREVIEW, AND REVIEW CALLOUTS
  info_callout = `${info_callout_title_context}
${info_callout_pillar}
${info_callout_goal_proj}
${info_callout_org}
${info_callout_contact}
${info_callout_book}
${info_callout_chapter}
${info_callout_start_end_date}`;

  // FILE CONTENT
  file_content = `---
${fmatter_title}
${fmatter_alias}
${fmatter_date_start}
${fmatter_date_end}
${fmatter_due_do}
${fmatter_pillar}
${fmatter_context}
${fmatter_goal}
${fmatter_project}
${fmatter_organization}
${fmatter_contact}
${fmatter_status}
${fmatter_type}
${fmatter_file_class}
${fmatter_date_created}
${fmatter_date_modified}
${fmatter_tags}
---\n
tags::\n
---\n

# ${full_title_name}\n

${info_callout}\n
---\n

## Prepare and Reflect\n

${parent_task_preview_review}\n
---\n

## Tasks and Events\n

### Remaining Tasks\n

${child_task_remaining}\n

### Completed Tasks\n

${child_task_completed}\n
---\n

## Related Tasks and Events\n

### Outgoing Task and Events Links\n

${cmnt_html_start}Link related tasks and events here${cmnt_html_end}\n

### Projects\n

${linked_projects}\n

### Parent Tasks\n

${linked_parent_tasks}\n

### Child Tasks\n

${linked_child_tasks}\n
---\n

## Related Notes\n

### Outgoing Note Links\n

${cmnt_html_start}Link related notes here${cmnt_html_end}\n

### Permanent\n

${linked_note_permanent}\n

### Literature\n

${linked_note_lit}\n

### Fleeting\n

${linked_note_fleet}\n

### General\n

${linked_note_info}\n
---\n

## Related Directory\n

### Outgoing Contact Links\n

${cmnt_html_start}Link related contacts here${cmnt_html_end}\n

### Contacts\n

${linked_dir_contact}\n

### Outgoing Organization Links\n

${cmnt_html_start}Link related organizations here${cmnt_html_end}\n

### Organizations\n

${linked_dir_org}\n
---\n

## Related Resources\n

### Outgoing Resource Links\n

${cmnt_html_start}Link related resources here${cmnt_html_end}\n

### Library\n

${linked_lib_content}\n
\n`;

  // PARENT TASK DIRECTORY PATH AND CREATION
  directory = `${project_dir}${file_name}`;
  await this.app.vault.createFolder(directory);

  // Create subdirectory file
  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(directory)
);
}
%>
