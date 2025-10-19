<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = '20_pillars/';
const goals_dir = '30_goals/';
const education_proj_dir = '42_education/';
const contacts_dir = '51_contacts/';
const organizations_dir = '52_organizations/';
const lib_books_dir = '60_library/61_books/';

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = 'Education';
const context_value = context_name.toLowerCase();
const context_dir = education_proj_dir;

//-------------------------------------------------------------------
// PROJECT TASK TYPE AND FILE CLASS
//-------------------------------------------------------------------
const proj_type_name = 'Project';
const proj_type_value = proj_type_name.toLowerCase();
const proj_file_class = `task_${proj_type_value}`;

/* ---------------------------------------------------------- */
/*                  PROJECT SETUP PARENT TASK                 */
/* ---------------------------------------------------------- */

/* ------------------- FILE TYPE AND CLASS ------------------ */
const parent_type_name = 'Parent Task';
const parent_type_value = parent_type_name.replace(/\s/g, '_').toLowerCase();
const parent_file_class = `task_${parent_type_value.split('_')[0]}`;

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
const regex_snake_case_remove = /(,|'|:|;)/g;
const regex_snake_case_under = /(\-\s\-)|(\s)|(\-)/g;
const snake_case_fmt = (name) =>
  name
    .replaceAll(regex_snake_case_remove, '')
    .replaceAll(regex_snake_case_under, '_')
    .toLowerCase();
const md_ext = (file_name) => file_name + '.md';
const quote_enclose = (content) => `"${content}"`;

const code_inline = (content) => backtick + content + backtick;
const cmnt_obsidian = (content) =>
  [two_percent, content, two_percent].join(space);
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

// LINKS
const regex_link = /.*\[([\w_].+)\|([\w\s].+)\].+/g;
const regex_link_to_value = /^(\[\[)|(\|.+)/g;
const link_alias = (file, alias) => '[[' + [file, alias].join('|') + ']]';
const link_tbl_alias = (file, alias) => '[[' + [file, alias].join('\\|') + ']]';

// YAML PROPERTIES
const yaml_li = (value) => new_line + ul_yaml + `"${value}"`;
const yaml_li_link = (file, alias) =>
  new_line + ul_yaml + `"${link_alias(file, alias)}"`;

// CALLOUT
const call_title = (call_type, title) =>
  [great_than, `[!${call_type}]`, title].join(space);
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
async function file_path(file_name) {
  const path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${md_ext(file_name)}`))
    .map((file) => file.path)[0];
  return path;
}

async function metadata_alias(file_name) {
  const path = await file_path(file_name);
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

//-------------------------------------------------------------------
// SET BOOK FILE NAME, ALIAS, AND DIRECTORY
//-------------------------------------------------------------------
const book_name_alias = await tp.user.include_template(
  tp,
  '61_book_name_alias'
);
const book_value = book_name_alias.split(';')[0];
const book_name = book_name_alias.split(';')[1];
const book_dir = `${lib_books_dir}${book_value}/`;

//-------------------------------------------------------------------
// BOOK METADATA CACHE, TITLE, AND LINK
//-------------------------------------------------------------------
const book_file_path = md_ext(book_dir + book_value);
const book_tfile = await app.vault.getAbstractFileByPath(book_file_path);
const book_file_cache = await app.metadataCache.getFileCache(book_tfile);
const book_main_title = book_file_cache?.frontmatter?.main_title;

const book_value_link = yaml_li_link(book_value, book_main_title);

//-------------------------------------------------------------------
// BOOK PUBLISHED DATE
//-------------------------------------------------------------------
const date_published = book_file_cache?.frontmatter?.year_published;

/* ---------------------------------------------------------- */
/*                   SET START AND END DATES                  */
/* ---------------------------------------------------------- */
const task_start = await tp.user.nl_date(tp, 'start');
const task_start_link = `"[[${task_start}]]"`;
const task_end = await tp.user.nl_date(tp, 'end');
const task_end_link = `"[[${task_end}]]"`;

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  '20_02_pillar_name_alias_preset_know'
);
const [pillar_value, pillar_value_link] = pillar_name_alias.split(';');

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${context_name} ${proj_type_name}?`
);

//-------------------------------------------------------------------
// PUBLISHER ORGANIZATION FILE NAME, TITLE, AND LINK
//-------------------------------------------------------------------
const publisher_yaml = book_file_cache?.frontmatter?.publisher;

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
let [organization_value, organization_value_link] = org_name_alias.split(';');

if (organization_value != 'null') {
  organization_value_link = `${organization_value_link}${publisher_value_link}`;
} else {
  organization_value_link = publisher_value_link;
}

//-------------------------------------------------------------------
// AUTHOR CONTACT FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
let author_last_name = '';
let author_last_name_value = '';

const author_yaml = book_file_cache?.frontmatter?.author;
let author_value_link = null_yaml_li;

if (!null_arr.includes(author_yaml) && typeof author_yaml !== 'undefined') {
  const author_arr = author_yaml.toString().split(',');
  author_value_link = author_arr.map((author) => yaml_li(author)).join('');

  const author_file_names = author_arr.map((a) =>
    a.toString().replaceAll(regex_link_to_value, '').trim()
  );

  const author_last_names_arr = [];

  for (const file_name of author_file_names) {
    const author_file_path = md_ext(contacts_dir + file_name);
    const author_tfile = await app.vault.getAbstractFileByPath(
      author_file_path
    );
    const author_file_cache = await app.metadataCache.getFileCache(
      author_tfile
    );
    const author_name_last = author_file_cache?.frontmatter?.name_last;
    if (author_name_last) author_last_names_arr.push(author_name_last);
  }

  if (author_last_names_arr.length === 1) {
    author_last_name = author_last_names_arr[0];
  } else if (author_last_names_arr.length === 2) {
    author_last_name = `${author_last_names_arr[0]} and ${author_last_names_arr[1]}`;
  } else if (author_last_names_arr.length >= 3) {
    author_last_name = `${author_last_names_arr[0]} et al`;
  }

  author_last_name_value = author_last_name
    .replace(/'s\sand/g, '')
    .replaceAll(/\s/g, '_');
}

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
const contact_name_alias = await tp.user.include_template(
  tp,
  '51_contact_name_alias'
);
let [contact_value, contact_value_link] = contact_name_alias.split(';');

if (contact_value != 'null') {
  contact_value_link = `${contact_value_link}${author_value_link}`;
} else {
  contact_value_link = author_value_link;
}

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const do_due_date = await tp.user.include_template(tp, '40_task_do_due_date');
const [due_do_value, due_do_name] = do_due_date.split(';');

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, '40_task_status');
const [status_value, status_name, status_symbol] = task_status.split(';');

//-------------------------------------------------------------------
// PROJECT TITLES, ALIAS, FILE NAME, AND DIRECTORY
//-------------------------------------------------------------------
const full_title_name = `Read ${author_last_name}'s ${book_main_title}`;
const full_title_value = [
  'Read',
  author_last_name_value,
  date_published,
  book_main_title,
]
  .join('_')
  .replaceAll(' ', '_');
const short_title_name = full_title_name.toLowerCase();
const short_title_value = `read_${book_value}`;

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
const project_value_link = yaml_li_link(file_name, full_title_name);

const project_dir = `${context_dir}${file_name}/`;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: 'Prepare and Reflect',
    toc_level: 1,
    toc_key: 'Insight',
    file: '40_project_preview_review',
  },
  {
    head_key: 'Tasks and Events',
    toc_level: 1,
    toc_key: 'Tasks and Events',
    file: '140_00_related_task_sect_proj',
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
const proj_sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const proj_info = await tp.user.include_file('40_21_proj_ed_book_info_callout');

//-------------------------------------------------------------------
// PROJECT FRONTMATTER YAML PROPERTIES AND CONTENT
//-------------------------------------------------------------------
const proj_uuid = await tp.user.uuid();

const yaml_proj = [
  hr_line,
  `title:${space}${file_name}`,
  `uuid:${space}${proj_uuid}`,
  `aliases:${space}${file_alias}`,
  `task_start:${space}${task_start_link}`,
  `task_end:${space}${task_end_link}`,
  `due_do:${space}${due_do_value}`,
  `pillar:${space}${pillar_value_link}`,
  `context:${space}${context_value}`,
  `goal:${space}${goal}`,
  `organization:${organization_value_link}`,
  `contact:${contact_value_link}`,
  `library:${book_value_link}`,
  `status:${space}${status_value}`,
  `type:${space}${proj_type_value}`,
  `file_class:${space}${proj_file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  'tags:',
  hr_line,
].join(new_line);

// FILE CONTENT
const proj_file_content = [
  yaml_proj,
  head_lvl(1, full_title_name) + new_line,
  proj_info,
  proj_sections_content,
].join(new_line);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = project_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

/* ------------ PARENT TASK FILE DETAILS CALLOUT ------------ */
const parent_info = await tp.user.include_file(
  '41_21_par_ed_book_ch_info_callout'
);

/* --------- PARENT TASK PREVIEW AND REVIEW SECTION --------- */
section_obj_arr[0].content = await tp.user.include_template(
  tp,
  '41_parent_task_preview_review'
);

/* ---------- PARENT TASK TASKS AND EVENTS SECTION ---------- */
section_obj_arr[1].content = await tp.user.include_template(
  tp,
  '141_00_related_task_sect_parent'
);

//-------------------------------------------------------------------
// BOOK CHAPTERS OBJECT ARRAY
//-------------------------------------------------------------------
const chapter_obj_arr = (
  await tp.user.file_name_alias_by_class_type({
    dir: book_dir,
    file_class: 'lib',
    type: 'book_chapter',
  })
).filter((file) => file.value != 'null' && file.value != '_user_input');

//-------------------------------------------------------------------
// LOOP THROUGH ARRAY OF OBJECTS
//-------------------------------------------------------------------
for (let i = 0; i < chapter_obj_arr.length; i++) {
  const ch_name = chapter_obj_arr[i].key;
  const ch_value = chapter_obj_arr[i].value;
  // CHAPTER METADATA CACHE
  const ch_file_path = md_ext(book_dir + ch_value);
  const ch_tfile = await app.vault.getAbstractFileByPath(ch_file_path);
  const ch_file_cache = await app.metadataCache.getFileCache(ch_tfile);
  const ch_main_title = ch_file_cache?.frontmatter?.main_title;
  const ch_main_title_value = snake_case_fmt(ch_main_title);

  const ch_value_link = yaml_li_link(ch_value, ch_name);

  // TITLES, ALIAS, AND FILE NAME
  // Titles
  const par_full_title_name = `${full_title_name}:${space}${ch_main_title}`;
  const par_short_title_name = par_full_title_name.toLowerCase();
  const par_full_title_value = [full_title_value, ch_main_title_value]
    .join('_')
    .replaceAll(' ', '_')
    .toLowerCase();
  const par_short_title_value = `read_${ch_value}`;

  // Alias
  const par_file_alias = [
    par_full_title_name,
    par_short_title_name,
    par_full_title_value,
    par_short_title_value,
  ]
    .map((x) => yaml_li(x))
    .join('');

  // File name
  const par_file_name = par_short_title_value;

  // TOC CALLOUT
  const par_toc = toc.replaceAll(file_name, par_file_name);

  // PARENT TASK FILE SECTIONS
  const par_sections_content = section_obj_arr
    .map((s) => [s.head + two_new_line, par_toc, s.content].join(two_new_line))
    .join(new_line);

  const parent_uuid = await tp.user.uuid();

  // PARENT TASK FRONTMATTER YAML PROPERTIES
  const yaml_parent = [
    hr_line,
    `title:${space}${par_file_name}`,
    `uuid:${space}${parent_uuid}`,
    `aliases:${space}${par_file_alias}`,
    'task_start:',
    'task_end:',
    `due_do:${space}do`,
    `pillar:${space}${pillar_value_link}`,
    `context:${space}${context_value}`,
    `goal:${space}${goal}`,
    `project:${project_value_link}`,
    `organization:${organization_value_link}`,
    `contact:${contact_value_link}`,
    `library:${ch_value_link}`,
    `status:${space}schedule`,
    `type:${space}${parent_type_value}`,
    `file_class:${space}${parent_file_class}`,
    `date_created:${space}${date_created}`,
    `date_modified:${space}${date_modified}`,
    'tags:',
    hr_line,
  ].join(new_line);

  // FILE CONTENT
  const par_file_content = [
    yaml_parent,
    head_lvl(1, par_full_title_name) + new_line,
    parent_info,
    par_sections_content,
  ].join(new_line);

  // PARENT TASK DIRECTORY AND FILE PATH CREATION
  const parent_directory = `${project_dir}${par_file_name}`;
  await this.app.vault.createFolder(parent_directory);

  const par_file_path = `${parent_directory}/${par_file_name}.md`;
  await this.app.vault.create(par_file_path, par_file_content);
}


tR += yaml_proj;
%>
# <%* tR += full_title_name %>

<%* tR += proj_info %>
<%* tR += proj_sections_content %>
