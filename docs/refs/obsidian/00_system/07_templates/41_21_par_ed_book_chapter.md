<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = '20_pillars/';
const goals_dir = '30_goals/';
const education_proj_dir = '42_education/';
const contacts_dir = '51_contacts/';
const organizations_dir = '52_organizations/';
const lib_book_dir = '60_library/61_books/';

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = 'Education';
const context_value = context_name.toLowerCase();
const context_dir = education_proj_dir;

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = 'Parent Task';
const type_value = snake_case_fmt(type_name);
const file_class = `task_${type_value.split('_')[0]}`;

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

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
// FORMATTING
const head_lvl = (level, heading) => [hash.repeat(level), heading].join(space);
const regex_snake_case = /(\-\s\-)|(\s)|(\-)/g;
const snake_case_fmt = (name) =>
  name.replaceAll(regex_snake_case, '_').toLowerCase();
const md_ext = (file_name) => file_name + '.md';

const code_inline = (content) => backtick + content + backtick;
const cmnt_obsidian = (content) =>
  [two_percent, content, two_percent].join(space);
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

// LINKS
const regex_link = /.*\[([\w_].+)\|([\w\s].+)\].+/g;
const link_alias = (file, alias) => ['[[' + file, alias + ']]'].join('|');
const link_tbl_alias = (file, alias) => ['[[' + file, alias + ']]'].join('\\|');

// YAML PROPERTIES
const yaml_li = (value) => `${new_line}${ul_yaml}"${value}"`;
const yaml_li_link = (file, alias) =>
  `${new_line}${ul_yaml}"${link_alias(file, alias)}"`;

// CALLOUT
const call_title = (call_type, title) =>
  [great_than, `[!${call_type}]`, title].join(space);

// CALLOUT TABLE
const call_tbl_row = (content) =>
  [
    great_than,
    String.fromCodePoint(0x7c),
    content,
    String.fromCodePoint(0x7c),
    space,
  ].join(space);
const call_tbl_div = (int) =>
  call_tbl_row(Array(int).fill(tbl_cent).join(tbl_pipe));

// DATE
const date_time = (date, time = '00:00') => moment(`${date}T${time}`);
const date_fmt = (format, date, time = '00:00') =>
  moment(`${date}T${time}`).format(format);
const date_add_sub_fmt = (format, unit, value, date, time = '00:00') =>
  value > 0
    ? moment(`${date}T${time}`).add(value, unit).format(format)
    : moment(`${date}T${time}`).subtract(Math.abs(value), unit).format(format);

// DATAVIEW - INLINE
const dv_inline = (key, value) =>
  '[' + key + colon.repeat(2) + space + value + ']';
const dv_yaml = (property) => 'file.frontmatter.' + property;
const dv_content_link = code_inline(
  [
    'dv:',
    `link(this.file.name + "#" +`,
    `this.${dv_yaml('aliases[0]')},`,
    `"Contents")`,
  ].join(space)
);

// Utility: Split a semicolon-delimited string into trimmed components
function parse_semicolon_values(input, expected_count) {
  const parts = input.split(";");
  if (expected_count !== undefined && parts.length < expected_count) {
    throw new Error(`Expected ${expected_count} values but got ${parts.length}: "${input}"`);
  }

  return parts.map((s, i) => (i === 0 ? s.trim() : s));
}

// OBSIDIAN API
async function metadata_alias(file_name) {
  const path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${md_ext(file_name)}`))
    .map((file) => file.path)[0];
  const abstract_file = await app.vault.getAbstractFileByPath(path);
  const file_cache = await app.metadataCache.getFileCache(abstract_file);
  return file_cache?.frontmatter?.aliases[0];
}

/* ---------------------------------------------------------- */
/*                      GENERAL VARIABLES                     */
/* ---------------------------------------------------------- */

/* --------------------- NULL VARIABLES --------------------- */
const null_link = link_alias('null', 'Null');
const null_yaml_li = yaml_li(null_link);
const null_arr = ['', 'null', null_link, null];

/* -------- FILE CREATION AND MODIFIED DATE VARIABLES ------- */
const date_created = moment().format('YYYY-MM-DD[T]HH:mm');
const date_modified = moment().format('YYYY-MM-DD[T]HH:mm');

/* ------------------- FILE PATH VARIABLES ------------------ */
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split(`/`);
const folder_path_length = folder_path_split.length;

//-------------------------------------------------------------------
// SET PROJECT BY FILE PATH OR SUGGESTER
//-------------------------------------------------------------------
let project_value;
let project_name;
if (context_dir == `${folder_path_split[0]}/` && folder_path_length >= 2) {
  const project_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[1],
    file_class: 'task',
    type: 'project',
  });
  project_value = project_obj[1].value;
  project_name = project_obj[1].key;
} else {
  const project_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: context_dir,
    file_class: 'task',
    type: 'project',
  });
  const project_obj = await tp.system.suggester(
    (item) => item.key,
    project_obj_arr,
    false,
    'Project?'
  );
  project_value = project_obj.value;
  project_name = project_obj.key;
}
const project_yaml = yaml_li_link(project_value, project_name);
const project_name_ext = md_ext(project_value);
const project_file_path = await app.vault
  .getMarkdownFiles()
  .filter((file) => file.path.endsWith(`/${project_name_ext}`))
  .map((file) => file.path)[0];
const project_dir = `${context_dir}${project_value}/`;

//-------------------------------------------------------------------
// BOOK METADATA CACHE, TITLE, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const proj_abstract_file = await app.vault.getAbstractFileByPath(
  project_file_path
);
const proj_file_cache = await app.metadataCache.getFileCache(
  proj_abstract_file
);
const proj_library_yaml = proj_file_cache?.frontmatter?.library[0];
const book_value = proj_library_yaml.split('|')[0].slice(2);
const book_file_path = `${lib_book_dir}${book_value}.md`;
const book_tfile = await app.vault.getAbstractFileByPath(book_file_path);
const book_file_cache = await app.metadataCache.getFileCache(book_tfile);
const book_main_title = book_file_cache?.frontmatter?.title;
const book_main_title_value = book_main_title
  .replaceAll(/\s/g, '_')
  .toLowerCase();

const book_dir = `${lib_book_dir}${book_value}/`;

//-------------------------------------------------------------------
// BOOK PUBLISHED DATE
//-------------------------------------------------------------------
const date_published = book_file_cache?.frontmatter?.year_published;

//-------------------------------------------------------------------
// SET BOOK CHAPTER
//-------------------------------------------------------------------
const chapter_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: book_dir,
  file_class: 'lib',
  type: 'book_chapter',
});
const chapter_obj = await tp.system.suggester(
  (item) => item.key,
  chapter_obj_arr,
  false,
  'Chapter?'
);
const chapter_value = chapter_obj.value;
const chapter_name = chapter_obj.key;
const lib_chapter_yaml = yaml_li_link(chapter_value, chapter_name);

//-------------------------------------------------------------------
// BOOK CHAPTER METADATA CACHE
//-------------------------------------------------------------------
const chapter_file_path = await app.vault
  .getMarkdownFiles()
  .filter((file) => file.path.endsWith(`/${chapter_value}.md`))
  .map((file) => file.path)[0];
const chapter_tfile = await app.vault.getAbstractFileByPath(chapter_file_path);
const chapter_file_cache = await app.metadataCache.getFileCache(chapter_tfile);
const chapter_main_title = chapter_file_cache?.frontmatter?.main_title;
const chapter_main_title_value = chapter_main_title
  .replaceAll(/\s/g, '_')
  .toLowerCase();

/* ---------------------------------------------------------- */
/*                   SET START AND END DATES                  */
/* ---------------------------------------------------------- */
const task_start = await tp.user.nl_date(tp, 'start');
const task_start_link = "[[" + task_start + "]]";
const task_end = await tp.user.nl_date(tp, 'end');
const task_end_link = "[[" + task_end + "]]";

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  '20_02_pillar_name_alias_preset_know'
);
const pillar_value = pillar_name_alias.split(';')[0];
const pillar_yaml = pillar_name_alias.split(';')[1];

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

//-------------------------------------------------------------------
// PUBLISHER ORGANIZATION FILE NAME, TITLE, AND LINK
//-------------------------------------------------------------------
const publisher_yaml = chapter_file_cache?.frontmatter?.publisher;

let publisher_value_link = null_yaml_li;
if (
  !null_arr.includes(publisher_yaml) &&
  typeof publisher_yaml != 'undefined'
) {
  publisher_arr = publisher_yaml.toString().split(',');
  publisher_value_link = publisher_arr
    .map((publisher) => yaml_li(publisher))
    .join('');
}

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const org_name_alias = await tp.user.include_template(
  tp,
  '52_organization_name_alias'
);
let organization_value = org_name_alias.split(';')[0];
let organization_yaml =
  organization_value != 'null'
    ? org_name_alias.split(';')[1] + publisher_value_link
    : publisher_value_link;

//-------------------------------------------------------------------
// AUTHOR CONTACT FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const author_yaml = chapter_file_cache?.frontmatter?.author;

let author_value_link = null_yaml_li;
if (!null_arr.includes(author_yaml) && typeof author_yaml != 'undefined') {
  author_arr = author_yaml.toString().split(',');
  author_value_link = author_arr.map((author) => yaml_li(author)).join('');
}

const author_file_name_arr = author_yaml.map((file) =>
  file.toString().replaceAll(regex_link_to_value, '')
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
  .replace(/'s\sand/g, '')
  .replaceAll(/\s/g, '_');

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
const contact_name_alias = await tp.user.include_template(
  tp,
  '51_contact_name_alias'
);
let contact_value = contact_name_alias.split(';')[0];
let contact_yaml =
  contact_value != 'null'
    ? contact_name_alias.split(';')[1] + author_value_link
    : author_value_link;

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const do_due_date = await tp.user.include_template(tp, '40_task_do_due_date');
const due_do_value = do_due_date.split(';')[0];
const due_do_name = do_due_date.split(';')[1];

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, '40_task_status');
const status_value = task_status.split(';')[0];
const status_name = task_status.split(';')[1];
const status_symbol = task_status.split(';')[2];

//-------------------------------------------------------------------
// PARENT TASK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//-------------------------------------------------------------------
const full_title_name = `Learn ${author_last_name}'s ${book_main_title}: ${chapter_main_title}`;
const short_title_name = full_title_name.toLowerCase();
const full_title_value = `Learn_${author_last_name}_${date_published}_${book_main_title}_${chapter_main_title}`;
const short_title_value = `learn_${chapter_value}`;

const file_alias = [
  full_title_name,
  short_title_name,
  full_title_value,
  short_title_value,
]
  .map((x) => yaml_li(x))
  .join('');

const file_name = short_title_value;
const file_section = file_name + hash;

const parent_task_dir = `${project_dir}${file_name}/`;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: 'Prepare and Reflect',
    toc_level: 1,
    toc_key: 'Insight',
    file: '41_parent_task_preview_review',
  },
  {
    head_key: 'Tasks and Events',
    toc_level: 1,
    toc_key: 'Tasks and Events',
    file: '141_00_related_task_sect_parent',
  },
  {
    head_key: 'Related Tasks and Events',
    toc_level: 1,
    toc_key: 'Related Tasks',
    file: '100_42_related_task_sect_task_file',
  },
  {
    head_key: 'Related Knowledge',
    toc_level: 2,
    toc_key: 'PKM',
    file: '100_70_related_pkm_sect',
  },
  {
    head_key: 'Related Library Content',
    toc_level: 2,
    toc_key: 'Library',
    file: '100_60_related_lib_sect',
  },
  {
    head_key: 'Related Directory',
    toc_level: 2,
    toc_key: 'Directory',
    file: '100_50_related_dir_sect',
  },
];

// Content, heading, and table of contents link
for (let i = 0; i < section_obj_arr.length; i++) {
  section_obj_arr[i].content = await tp.user.include_template(
    tp,
    section_obj_arr[i].file
  );
  section_obj_arr[i].head = head_lvl(2, section_obj_arr[i].head_key);
  section_obj_arr[i].toc = link_tbl_alias(
    file_section + section_obj_arr[i].head_key,
    section_obj_arr[i].toc_key
  );
}

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_lvl = (int) =>
  call_tbl_row(
    section_obj_arr
      .filter((x) => x.toc_level == int)
      .map((x) => x.toc)
      .join(tbl_pipe)
  );

const toc = [
  call_title('toc', dv_content_link),
  call_start,
  toc_lvl(1),
  call_tbl_div(3),
  toc_lvl(2),
].join(new_line);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info = await tp.user.include_file('41_21_par_ed_book_ch_info_callout');

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = parent_task_dir;

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
pillar: <%* tR += pillar_yaml %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
project: <%* tR += project_yaml %>
organization: <%* tR += organization_yaml %>
contact: <%* tR += contact_yaml %>
library: <%* tR += lib_chapter_yaml %>
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