<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pkm_dir = '70_pkm/';

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = 'Snippet';
const type_value = type_name.toLowerCase();
const type_value_short = 'snip';
const file_class = 'pkm_code';

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

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
// FORMATTING
const head_lvl = (level, heading) => [hash.repeat(level), heading].join(space);
const regex_snake_case = /(\-\s\-)|(\s)|(\-)]/g;
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

// DATAVIEW - INLINE
const dv_inline = (key, value) => '[' + key + colon.repeat(2) + value + ']';
const dv_yaml = (property) => 'file.frontmatter.' + property;
const dv_content_link = code_inline(
  [
    'dv:',
    `link(this.file.name + "#" +`,
    `this.${dv_yaml('aliases[0]')},`,
    `"Contents")`,
  ].join(space)
);

// OBSIDIAN API
async function file_path_api(file_name) {
  const path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${md_ext(file_name)}`))
    .map((file) => file.path)[0];
  return path;
}

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
const null_obj = {
  index: 0,
  key: 'Null',
  value: 'null',
  value_link: null_yaml_li,
};

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const date_created = moment().format('YYYY-MM-DD[T]HH:mm');
const date_modified = moment().format('YYYY-MM-DD[T]HH:mm');

//-------------------------------------------------------------------
// SET PROGRAMMING LANGUAGE AND FILE CLASS
//-------------------------------------------------------------------
const lang_obj_arr = [
  { key: 'AutoHotkey', value: 'autohotkey', short: 'ahk' },
  { key: 'CSS', value: 'css', short: 'css' },
  { key: 'Git', value: 'git', short: 'git' },
  { key: 'Google Apps Script', value: 'google_apps_script', short: 'gas' },
  { key: 'Google Sheets', value: 'google_sheets', short: 'gs' },
  { key: 'HTML', value: 'html', short: 'html' },
  { key: 'JavaScript', value: 'javascript', short: 'js' },
  { key: 'LaTeX', value: 'latex', short: 'ltx' },
  { key: 'Microsoft Excel', value: 'ms_excel', short: 'xl' },
  { key: 'Python', value: 'python', short: 'py' },
  { key: 'SQL', value: 'sql', short: 'sql' },
  { key: 'Tableau', value: 'tableau', short: 'tbl' },
];

const lang_obj = await tp.system.suggester(
  (item) => item.key,
  lang_obj_arr,
  false,
  'Programming Language?'
);

const lang_name = lang_obj.key;
const lang_value = lang_obj.value;
const lang_value_short = lang_obj.short;
const topic_value_link = yaml_li_link(lang_value, lang_name);

//-------------------------------------------------------------------
// LANGUAGE (TOPIC) METADATA CACHE
//-------------------------------------------------------------------
const lang_file_path = await file_path_api(lang_value);
const lang_file_dir = lang_file_path.replace(md_ext(lang_value), '');
const lang_tfile = await app.vault.getAbstractFileByPath(lang_file_path);
const lang_file_cache = await app.metadataCache.getFileCache(lang_tfile);

//-------------------------------------------------------------------
// SUBLANGUAGE, LIBRARY, MODULE (SUBTOPIC) METADATA CACHE
//-------------------------------------------------------------------
const subtopic_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: lang_file_dir,
  file_class: 'pkm_tree',
  type: 'subtopic',
});

const subtopic_obj = await tp.system.suggester(
  (item) => item.key,
  subtopic_obj_arr,
  false,
  'Subtopic (e.g. Library, Module, Sublanguage)?'
);

const subtopic_value = subtopic_obj.value;
const subtopic_name = subtopic_obj.key;
const subtopic_value_link = yaml_li_link(subtopic_value, subtopic_name);

const subtopic_file_path =
  subtopic_value != 'null' ? await file_path_api(subtopic_value) : null;
const subtopic_dir =
  subtopic_value != 'null'
    ? subtopic_file_path.replace(md_ext(subtopic_value), '')
    : null;

//-------------------------------------------------------------------
// PKM TREE FILES AND NAMES
//-------------------------------------------------------------------
const pkm_tree_obj_arr = [
  { index: 4, key: 'Subject', value: 'subject' },
  { index: 3, key: 'Field', value: 'field' },
  { index: 2, key: 'Branch', value: 'branch' },
  { index: 1, key: 'Category', value: 'category' },
];

pkm_tree_obj_arr.forEach((obj) => {
  const pkm_yaml = lang_file_cache?.frontmatter?.[obj.value];
  obj.value_link =
    !null_arr.includes(pkm_yaml) && pkm_yaml !== undefined
      ? pkm_yaml.toString().split(',').map(yaml_li).join('')
      : null_yaml_li;
});

const [
  subject_value_link,
  field_value_link,
  branch_value_link,
  category_value_link,
] = pkm_tree_obj_arr.map((tree) => tree.value_link)

//-------------------------------------------------------------------
// SET SUBTYPE
//-------------------------------------------------------------------
const subtype_obj_arr = [
  { key: 'Null', value: 'null', value_short: 'null' },
  { key: 'Aggregate', value: 'aggregate', value_short: 'agg' },
  { key: 'Array', value: 'array', value_short: 'arr' },
  { key: 'Boolean', value: 'boolean', value_short: 'bool' },
  { key: 'Converter', value: 'converter', value_short: 'convert' },
  { key: 'Database', value: 'database', value_short: 'db' },
  { key: 'DataFrame', value: 'dataframe', value_short: 'df' },
  { key: 'Date', value: 'date', value_short: 'date' },
  { key: 'Dictionary', value: 'dictionary', value_short: 'dict' },
  { key: 'Engineering', value: 'engineering', value_short: 'eng' },
  { key: 'File', value: 'file', value_short: 'file' },
  { key: 'Filter', value: 'filter', value_short: 'fltr' },
  { key: 'Financial', value: 'financial', value_short: 'fin' },
  { key: 'Google', value: 'google', value_short: 'goog' },
  { key: 'Info', value: 'info', value_short: 'info' },
  { key: 'Iterable', value: 'iterable', short: 'iter' },
  { key: 'Level of Detail', value: 'lod', value_short: 'lod' },
  { key: 'List', value: 'list', value_short: 'list' },
  { key: 'Logical', value: 'logical', value_short: 'logic' },
  { key: 'Lookup', value: 'lookup', value_short: 'look' },
  { key: 'Math', value: 'math', value_short: 'math' },
  { key: 'Numeric', value: 'numeric', value_short: 'num' },
  { key: 'Object', value: 'object', value_short: 'obj' },
  { key: 'Operator', value: 'operator', value_short: 'opr' },
  { key: 'Parser', value: 'parser', value_short: 'prs' },
  {
    key: 'Pass-Through RAWSQL',
    value: 'pass_through_rawsql',
    value_short: 'rawsql',
  },
  { key: 'Regular Expression', value: 'regex', value_short: 'regex' },
  { key: 'Series', value: 'series', value_short: 'ser' },
  { key: 'Set', value: 'set', value_short: 'set' },
  { key: 'Spatial', value: 'spatial', value_short: 'space' },
  { key: 'Statistics', value: 'statistics', value_short: 'stat' },
  { key: 'String', value: 'string', value_short: 'str' },
  {
    key: 'Table Calculation',
    value: 'table_calculation',
    value_short: 'table_calc',
  },
  { key: 'Tuple', value: 'tuple', value_short: 'tupl' },
  { key: 'User', value: 'user', value_short: 'user' },
  { key: 'Web', value: 'web', value_short: 'web' },
  { key: 'General', value: 'general', value_short: 'general' },
];

const subtype_obj = await tp.system.suggester(
  (item) => item.key,
  subtype_obj_arr,
  false,
  `${type_name} subtype?`
);

const subtype_name = subtype_obj.key;
const subtype_value = subtype_obj.value;
const subtype_value_short = subtype_obj.value_short;

//-------------------------------------------------------------------
// SET FILE'S LONG AND SHORT TITLES
//-------------------------------------------------------------------
let long_title = await tp.system.prompt(
  `${lang_name} ${type_value} Long Title?`,
  null,
  true,
  false
);
long_title = await tp.user.title_case(long_title);

const has_title = !tp.file.title.startsWith('Untitled');
let title;
if (!has_title) {
  title = await tp.system.prompt(
    `${lang_name} ${type_value} Short Title?`,
    null,
    true,
    false
  );
} else {
  title = tp.file.title;
}
title = title.trim();

/* ---------------------------------------------------------- */
/*          FRONTMATTER TITLE, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
const title_name = await tp.user.title_case(title);
const title_value = snake_case_fmt(title);
const long_title_value = snake_case_fmt(long_title);

const full_title_name = `${lang_name} ${long_title} ${type_name}`;
const full_title_value = `${lang_value}_${long_title_value}_${type_value}`;
const partial_title_name = `${lang_name} ${title_name} ${type_name}`;
const partial_title_value = `${lang_value}_${title_value}_${type_value}`;
const short_title_name = `${lang_name} ${title_name}`;
const short_title_value = `${lang_value}_${title_value}`;

const file_name = `${lang_value_short}_${type_value_short}_${title_value}`;

const file_alias = [
  title_name,
  long_title,
  full_title_name,
  full_title_value,
  partial_title_name,
  partial_title_value,
  short_title_name,
  short_title_value,
  file_name,
]
  .map((x) => yaml_li(x))
  .join('');

const file_section = file_name + hash;

/* ---------------------------------------------------------- */
/*                       SET DESCRIPTION                      */
/* ---------------------------------------------------------- */
const why_prompt = `What is the ${type_name}'s purpose?`;
const about_why = await tp.system.prompt(why_prompt, null, false, true);

const how_prompt = `How does the ${type_name} work, or achieve its purpose${two_new_line}Purpose:${space}${about_why}?`;
const about_how = await tp.system.prompt(how_prompt, null, false, true);

const about = ['**Purpose**: ' + about_why, '**Process**: ' + about_how].join(
  new_line
);
const about_value = await tp.user.yaml_multiline(about);

/* ---------------------------------------------------------- */
/*                      SET REFERENCE URL                     */
/* ---------------------------------------------------------- */
const url = await tp.system.prompt(`${type_name} Reference URL?`);

//-------------------------------------------------------------------
// SET RELATED LANGUAGE FUNCTIONS
//-------------------------------------------------------------------
let lang_dir = subtopic_value == 'null' ? lang_file_dir : subtopic_dir;

const related_lang_dirs = app.vault
  .getAllLoadedFiles()
  .filter((i) => i.path.includes(lang_dir) && i.children)
  .map((folder) => folder.path);

const related_lang_file_obj_arr = [{ key: 'Null', value: 'null' }];
for (let i = 0; i < related_lang_file_obj_arr.length; i++) {
  const obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: related_lang_file_obj_arr[i],
    file_class: file_class,
    type: '',
  });
  related_lang_file_obj_arr.push(
    ...obj_arr.filter((x) => !['null', '_user_input'].includes(x.value))
  );
}

const bool_obj_arr = [
  { key: '✔️ YES ✔️', value: 'yes' },
  { key: '❌ NO ❌', value: 'no' },
];

let file_obj_arr = [];
let file_filter = [];
for (let i = 0; i < 10; i++) {
  // File Suggester
  const related_lang_file_obj = await tp.system.suggester(
    (item) => item.key,
    related_lang_file_obj_arr.filter(
      (file) => !file_filter.includes(file.value)
    ),
    false,
    'Related Language File?'
  );
  file_basename = related_lang_file_obj.value;
  file_alias_name = related_lang_file_obj.key;

  if (file_basename == 'null') {
    file_obj = { key: file_alias_name, value: file_basename };
    file_obj_arr.push(file_obj);
    break;
  }
  file_obj = { key: file_alias_name, value: file_basename };
  file_obj_arr.push(file_obj);
  file_filter.push(file_basename);

  const bool_obj = await tp.system.suggester(
    (item) => item.key,
    bool_obj_arr,
    false,
    'Another Related Language File?'
  );

  if (bool_obj.value == 'no') {
    break;
  }
}

const related_lang_file_link = file_obj_arr
  .map((file) => `${ul}${link_alias(file.value, file.key)}`)
  .join('');

/* ---------------------------------------------------------- */
/*   SET PILLAR FILE NAME AND TITLE; PRESET KNOW. EXPANSION   */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  '20_02_pillar_name_alias_preset_know'
);
const pillar_value = pillar_name_alias.split(';')[0];
const pillar_value_link = pillar_name_alias.split(';')[1];

/* ---------------------------------------------------------- */
/*                       SET NOTE STATUS                      */
/* ---------------------------------------------------------- */
const note_status = await tp.user.include_template(tp, '80_note_status');
const status_name = note_status.split(';')[0];
const status_value = note_status.split(';')[1];

//-------------------------------------------------------------------
// PKM TREE HIERARCHY CALLOUT
//-------------------------------------------------------------------
const pkm_tree_hierarchy =
  (await tp.user.include_file('70_pkm_tree_hierarchy_callout')) + two_new_line;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: 'Snippet Information',
    toc_key: 'Snippet Info',
    file: null,
    content: null,
  },
  {
    head_key: 'Related Snippets',
    toc_key: 'Use Cases',
    file: '100_71_related_pkm_snip_sect_snip_file',
    content: null,
  },
  {
    head_key: 'Related Code',
    toc_key: 'Related Code',
    file: '100_71_related_pkm_code_sect_snip_file',
    content: null,
  },
  {
    head_key: 'Related Knowledge',
    toc_key: 'PKM',
    file: '100_70_related_pkm_sect',
    content: pkm_tree_hierarchy,
  },
  {
    head_key: 'Related Library Content',
    toc_key: 'Library',
    file: '100_61_related_lib_sect_related_file',
    content: null,
  },
];

// Content, heading, and table of contents link
for (let i = 0; i < section_obj_arr.length; i++) {
  if (!section_obj_arr[i].file) {
    continue;
  }
  if (!section_obj_arr[i].content) {
    section_obj_arr[i].content = await tp.user.include_template(
      tp,
      section_obj_arr[i].file
    );
  } else {
    section_obj_arr[i].content += await tp.user.include_template(
      tp,
      section_obj_arr[i].file
    );
  }
  section_obj_arr[i].head = head_lvl(2, section_obj_arr[i].head_key);
  section_obj_arr[i].toc = link_tbl_alias(
    file_section + section_obj_arr[i].head_key,
    section_obj_arr[i].toc_key
  );
}

//-------------------------------------------------------------------
// SNIPPET INFORMATION SECTION
//-------------------------------------------------------------------
const data_lang_arr = ['google_sheets', 'ms_excel', 'sql', 'tableau'];
let head_info_io;
let call_info_io;
if (data_lang_arr.includes(lang_value)) {
  head_info_io = 'Data';
  call_info_io = await tp.user.include_file('81_code_snip_input_data_table');
} else {
  head_info_io = 'Input and Output';
  call_info_io = await tp.user.include_file(
    '81_code_snip_input_output_callout'
  );
}

section_obj_arr[0].content =
  [
    head_lvl(3, head_info_io),
    call_info_io,
    head_lvl(3, 'Snippet'),
    head_lvl(4, 'Snippet Explanation'),
    cmnt_html('Include explanatory comments'),
    three_backtick + lang_value,
    three_backtick,
    head_lvl(4, 'Clean Snippet'),
    cmnt_html('Exclude explanatory comments'),
    three_backtick + lang_value,
    three_backtick,
    head_lvl(3, 'Language Reference'),
    cmnt_html('Recreate the code with links'),
    related_lang_file_link,
    hr_line,
  ].join(two_new_line) + new_line;

//-------------------------------------------------------------------
// FILE CONTENTS CALLOUT
//-------------------------------------------------------------------
const toc_title = call_title('toc', dv_content_link);

const toc_body_high = call_title(
  section_obj_arr.map((x) => x.toc).join(tbl_pipe)
);

const toc = [toc_title, call_start, toc_body_high, call_tbl_div(5)].join(
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
const info_title = call_title(type_value, type_name + ' Details');
const info_body = await tp.user.include_file('81_pkm_code_info_callout');

const info = [info_title, call_start, info_body].join(new_line);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
let directory = lang_file_dir + type_value + 's';
if (type_value == 'method' && subtype_value != 'null') {
  directory += '_' + subtype_value;
}
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}/${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
pillar: <%* tR += pillar_value_link %>
category: <%* tR += category_value_link %>
branch: <%* tR += branch_value_link %>
field: <%* tR += field_value_link %>
subject: <%* tR += subject_value_link %>
topic: <%* tR += topic_value_link %>
subtopic: <%* tR += subtopic_value_link %>
library:
url: <%* tR += url %>
about: |-
 <%* tR += about_value %>
status: <%* tR += status_value %>
subtype: <%* tR += subtype_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_title_name %>

<%* tR += info %>
<%* tR += sections_content %>
