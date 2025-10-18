<%*  
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const library_dir = '60_library/';
const lib_books_dir = '60_library/61_books/';

//-------------------------------------------------------------------
// BOOK TYPE AND FILE CLASS
//-------------------------------------------------------------------
const type_name = 'Book';
const type_value = type_name.toLowerCase();
const file_class = `lib_${type_value}`;

//-------------------------------------------------------------------
// BOOK CHAPTER TYPE AND FILE CLASS
//-------------------------------------------------------------------
const chapter_type_name = 'Book Chapter';
const chapter_type_value = chapter_type_name
  .replaceAll(/\s/g, '_')
  .toLowerCase();
const chapter_file_class = `lib_${chapter_type_value}`;

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
const regex_snake_case_under = /(;\s)|(:\s)|(\-\s\-)|(\s)|(\-)/g;
const regex_snake_case_remove = /(,|'|:|;)/g;
const snake_case_fmt = (name) =>
  name
    .replaceAll(regex_snake_case_under, '_')
    .replaceAll(regex_snake_case_remove, '')
    .toLowerCase();
const md_ext = (file_name) => file_name + '.md';
const quote_enclose = (content) => `"${content}"`;

// CODE
const code_inline = (content) => backtick + content + backtick;
const code_block = (language = '', content = '') =>
  [three_backtick + language, content, three_backtick].join(new_line);

// COMMENTS
const cmnt_obsidian = (content) =>
  [two_percent, content, two_percent].join(space);
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

// LINKS
const regex_link = /.*\[([\w_].+)\|([\w\s].+)\].+/g;
const link_alias = (file, alias) => '[[' + [file, alias].join('|') + ']]';
const link_tbl_alias = (file, alias) => '[[' + [file, alias].join('|') + ']]';

// YAML PROPERTIES
const yaml_li = (value) => new_line + ul_yaml + `"${value}"`;
const yaml_li_link = (file, alias) =>
  new_line + ul_yaml + `"${link_alias(file, alias)}"`;

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

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith('Untitled');
let title;
if (!has_title) {
  title = await tp.system.prompt(`${type_name} Title`, null, true, false);
} else {
  title = tp.file.title;
}
title = title.trim();
title = await tp.user.title_case(title);

//-------------------------------------------------------------------
// SET AUTHOR'S CONTACT FILE NAME AND TITLE
//-------------------------------------------------------------------
const contact_name_alias = await tp.user.include_template(
  tp,
  '51_contact_name_alias'
);
const contact_value = contact_name_alias.split(';')[0];
const contact_name = contact_name_alias.split(';')[1];
const contact_value_link = contact_name_alias.split(';')[2];

const name_last_value_arr = contact_value
  .split(', ')
  .map((c) => c.split('_')[0]);
let contact_name_last_value;
if (name_last_value_arr.length >= 3) {
  contact_name_last_value = `${name_last_value_arr[0]}_et_al`;
} else if (name_last_value_arr.length >= 2) {
  contact_name_last_value = `${name_last_value_arr[0]}_${name_last_value_arr[1]}`;
} else if (name_last_value_arr.length >= 1) {
  contact_name_last_value = name_last_value_arr[0];
}

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const org_name_alias = await tp.user.include_template(
  tp,
  '52_organization_name_alias'
);
const organization_value = org_name_alias.split(';')[0];
const organization_value_link = org_name_alias.split(';')[1];

//-------------------------------------------------------------------
// SET PUBLISHED DATE
//-------------------------------------------------------------------
const year_published = await tp.system.prompt(
  'What year was the book published?'
);

//-------------------------------------------------------------------
// SET CITY
//-------------------------------------------------------------------
const bool_obj_arr = [
  { key: '✔️ YES ✔️', value: 'yes' },
  { key: '❌ NO ❌', value: 'no' },
];
let bool_obj = await tp.system.suggester(
  (item) => item.key,
  bool_obj_arr,
  false,
  'Same city as publisher?'
);

let city_value;
if (bool_obj.value == 'yes') {
  tfile = tp.file.find_tfile(`${organization_value}.md`);
  file_cache = await app.metadataCache.getFileCache(tfile);
  city_value = file_cache?.frontmatter?.city;
} else {
  const location = await tp.user.suggester_location({
    tp,
    country: false,
    city: true,
    utc_dst: false,
  });
  city_value = location.city_value;
}

//-------------------------------------------------------------------
// SET BOOK URL
//-------------------------------------------------------------------
const url = await tp.system.prompt('Book URL?', null, false, false);

//-------------------------------------------------------------------
// SET SERIES
//-------------------------------------------------------------------
let series_name = await tp.system.prompt('Book Series?', null, false, false);
let series_value = snake_case_fmt(
  series_name.replaceAll(/\//g, '-').replaceAll(/&/g, 'and')
);
if (series_name == '') {
  series_name = 'Null';
  series_value = 'null';
}
const series_value_link = yaml_li_link(series_value, series_name);

const series_url = await tp.system.prompt(
  'Book Series URL?',
  null,
  false,
  false
);

//-------------------------------------------------------------------
// SET THE BOOK'S DESCRIPTION
//-------------------------------------------------------------------
const about = await tp.system.prompt('Book Description?', null, false, true);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, '$2')
  .replaceAll(/(\s*)\n/g, '\n')
  .replaceAll(/([^\s])(\s*)$/g, '$1')
  .replaceAll(/\n{1,6}/g, '<new_line>')
  .replaceAll(/(<new_line>)(\w)/g, '\n \n $2')
  .replaceAll(/(<new_line>)(-\s|\d\.\s)/g, '\n $2');

//-------------------------------------------------------------------
// SET LIBRARY STATUS
//-------------------------------------------------------------------
const lib_status = await tp.user.include_template(tp, '60_library_status');
const status_value = lib_status.split(';')[0];
const status_name = lib_status.split(';')[1];

//-------------------------------------------------------------------
// BOOK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//-------------------------------------------------------------------
const {
  full_title_name: book_full_title_name,
  full_title_value: book_full_title_value,
  main_title: book_main_title,
  main_title_value: book_main_title_value,
  sub_title: book_subtitle,
} = await tp.user.lib_content_titles(title);

//const book_main_title_value = snake_case_fmt(book_main_title);

const file_name = [
  contact_name_last_value,
  year_published,
  book_main_title_value,
].join('_');

let file_alias = [
  book_full_title_name,
  book_full_title_value,
  book_main_title,
  book_main_title_value,
  file_name,
]
  .map((x) => yaml_li(x))
  .join('');

const file_section = file_name + hash;
const book_uuid = await tp.user.uuid();

const book_value_link = yaml_li_link(file_name, book_full_title_name);
const book_dir = lib_books_dir + file_name;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: 'Related Knowledge',
    toc_key: 'PKM',
    file: '100_70_related_pkm_sect',
  },
  {
    head_key: 'Related Library Content',
    toc_key: 'Library',
    file: '100_60_related_lib_sect',
  },
  {
    head_key: 'Related Tasks and Events',
    toc_key: 'Related Tasks',
    file: '100_40_related_task_sect_general',
  },
  {
    head_key: 'Related Directory',
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
const toc_row = call_tbl_row(section_obj_arr.map((x) => x.toc).join(tbl_pipe));

const toc = [
  call_title('toc', dv_content_link),
  call_start,
  toc_row,
  call_tbl_div(4),
].join(new_line);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content_book = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

//-------------------------------------------------------------------
// BOOK CONTENTS AND PDF/EPUB SECTION
//-------------------------------------------------------------------
const book_toc_link = link_alias(file_name + '_toc', 'Table of Contents');
const head_book_toc = head_lvl(2, book_toc_link);
const comment_book_file = cmnt_html('Insert book PDF/EPUB here');
let ol_book_toc_obj_arr = [];
let ol_book_toc_count = 0;

//-------------------------------------------------------------------
// BOOK INFO CALLOUT
//-------------------------------------------------------------------
const info_book = await tp.user.include_file('61_00_book_info_callout');

//-------------------------------------------------------------------
// SET BOOK COVER IMAGE PATH
//-------------------------------------------------------------------
const cover_path = `${file_name}_pic.webp`;
const cover_path_embed = '!' + link_alias(cover_path, 200);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = `${book_dir}/`;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path !== directory) {
  await tp.file.move(`${directory}${file_name}`);
}

//-------------------------------------------------------------------
// BOOK CHAPTER FILES
//-------------------------------------------------------------------
// BOOK CHPATER INFO CALLOUT
const info_chapter = await tp.user.include_file(
  '61_01_book_chapter_info_callout'
);

// BOOK CHPATER CONTENT COMMENT
const comment_chapter_file = cmnt_html('Insert chapter content here');

//-------------------------------------------------------------------
// BOOK CHAPTER DETAILS
//-------------------------------------------------------------------
const chapter_details_input = (
  await tp.system.prompt('Chapter Details Object Array', null, false, true)
)
  .split(';\n')
  .map((x) => x.trim())
  .filter((x) => x.length);

// BOOK CHAPTER DETAILS REGEX
const regex_patterns = {
  number: /chapter_number:\s*"(\d{1,4})"/i,
  title: /title:\s*"([^"]+)"/i,
  page_start: /page_start:\s*"([^"]+)"/i,
  page_end: /page_end:\s*"([^"]+)"/i,
};

// BOOK CHAPTER DETAILS ISOLATED
const extract_chapter_detail = (input, regex) => {
  const match = input.match(regex);
  return match ? match[1] : '';
};

// BOOK CHAPTER DETAILS OBJECT ARRAY
// Parse input array into an array of chapter detail objects
const chapter_details_obj_arr = chapter_details_input.map((line) => ({
  number: extract_chapter_detail(line, regex_patterns.number),
  title: extract_chapter_detail(line, regex_patterns.title),
  page_start: extract_chapter_detail(line, regex_patterns.page_start),
  page_end: extract_chapter_detail(line, regex_patterns.page_end),
}));

// BOOK CHAPTER FRONTMATTER
const yaml_bottom = [
  'doi:',
  `url:${space}${url}`,
  `library:${space}${book_value_link}`,
  'cssclasses:',
  'status: undetermined',
  `type:${space}${chapter_type_value}`,
  `file_class:${space}${chapter_file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  'tags:',
  hr_line,
].join(new_line);

//-------------------------------------------------------------------
// CREATE NEW BOOK CHAPTER FILES
//-------------------------------------------------------------------
for (let i = 0; i < chapter_details_obj_arr.length; i++) {
  const chapter_uuid = await tp.user.uuid();
  const chapter = chapter_details_obj_arr[i];

  // Normalize chapter number
  const raw_chapter_number = chapter.number;
  let chapter_number = '';
  if (raw_chapter_number.length >= 3) {
    const part = raw_chapter_number[0];
    const section = raw_chapter_number.slice(1).replace(/^0{1,2}/g, '');
    const has_section = !raw_chapter_number.match(/^\d00/g);
    chapter_number = has_section ? `${part}.${section}` : part;
  } else {
    chapter_number = raw_chapter_number.replace(/^0/g, '');
  }

  // Format title and get normalized title values
  const chapter_title = await tp.user.title_case(chapter.title);
  const {
    full_title_name: chapter_full_title_name,
    full_title_value: chapter_full_title_value,
    main_title: chapter_main_title,
    main_title_value: chapter_main_title_value,
    sub_title: chapter_subtitle,
  } = await tp.user.lib_content_titles(chapter_title);

  const chapter_title_num_name = `${chapter_number}.${space}${chapter_full_title_name}`;
  const book_chapter_title_name = `${book_main_title}:${space}${chapter_main_title}`;
  const book_chapter_title_value = `${book_main_title_value}_${chapter_main_title_value}`;

  const chapter_file_name = [
    chapter.number,
    chapter_main_title_value,
    book_main_title_value,
  ].join('_');

  const chapter_file_alias = [
    book_chapter_title_name,
    chapter_full_title_name,
    chapter_title_num_name,
    chapter_main_title,
    chapter_main_title_value,
    chapter_full_title_value,
    book_chapter_title_value,
    chapter_file_name,
  ]
    .map((x) => yaml_li(x))
    .join('');

  const page_start = chapter.page_start;
  const page_end = chapter.page_end;

  // TOC CALLOUT
  const toc_chap = toc.replaceAll(file_name, chapter_file_name);

  // BOOK CHAPTER FILE SECTIONS
  const sections_content_chap = section_obj_arr
    .map((s) => [s.head, toc_chap, s.content].join(two_new_line))
    .join(new_line);

  // BOOK FILE TABLE OF CONTENTS SECTION
  const chapter_link = link_alias(chapter_file_name, chapter_full_title_name);
  ol_book_toc_count = ol_book_toc_count + 1;
  ol_book_toc_obj = { key: ol_book_toc_count, value: chapter_link };
  ol_book_toc_obj_arr.push(ol_book_toc_obj);

  const yaml_chap = [
    hr_line,
    `title:${space}${chapter_file_name}`,
    `uuid:${space}${chapter_uuid}`,
    `aliases:${space}${chapter_file_alias}`,
    `main_title:${space}${chapter_main_title}`,
    `subtitle:${space}${chapter_subtitle}`,
    `author:${space}${contact_value_link}`,
    'editor:',
    'translator:',
    `year_published:${space}${year_published}`,
    `publisher:${space}${organization_value_link}`,
    `page_start:${space}${page_start}`,
    `page_end:${space}${page_end}`,
    yaml_bottom,
  ].join(new_line);

  // FILE CONTENT
  const file_content = [
    yaml_chap,
    head_lvl(1, chapter_title_num_name) + new_line,
    info_chapter,
    comment_chapter_file + new_line,
    hr_line + new_line,
    sections_content_chap,
  ].join(new_line);

  // BOOK CHAPTER DIRECTORY AND FILE PATH AND CREATION
  const book_directory = book_dir;
  //await this.app.vault.createFolder(book_directory);

  const file_path = `${book_directory}/${chapter_file_name}.md`;
  await this.app.vault.create(file_path, file_content);
}

const ol_book_toc = ol_book_toc_obj_arr
  .map((obj) => `${obj.key}.${space}${obj.value}`)
  .join(new_line);
const book_toc =
  [head_book_toc, toc, ol_book_toc, comment_book_file, hr_line].join(
    two_new_line
  ) + new_line

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += book_uuid %>
aliases: <%* tR += file_alias %>
main_title: <%* tR += book_main_title %>
subtitle: <%* tR += book_subtitle %>
author: <%* tR += contact_value_link %>
editor:
translator:
year_published: <%* tR += year_published %>
publisher: <%* tR += organization_value_link %>
city: <%* tR += city_value %>
edition:
volume:
series: <%* tR += series_value_link %>
series_url: <%* tR += series_url %>
isbn10:
isbn13:
doi:
url: <%* tR += url %>
cover_url:
cover_path: <%* tR += cover_path %>
about: |-
 <%* tR += about_value %>
cssclasses: null
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += book_full_title_name %>

<%* tR += info_book %>

<%* tR += cover_path_embed %>

---

<%* tR += book_toc %>
<%* tR += sections_content_book %>