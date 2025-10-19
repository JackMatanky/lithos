<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pkm_dir = '70_pkm/';

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
const link_tbl_alias = (file, alias) => '[[' + [file, alias].join('\\|') + ']]';

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

// Utility: Split a semicolon-delimited string into trimmed components
function parse_semicolon_values(input, expected_count) {
  const parts = input.split(";");
  if (expected_count !== undefined && parts.length < expected_count) {
    throw new Error(`Expected ${expected_count} values but got ${parts.length}: "${input}"`);
  }

  return parts.map((s, i) => (i === 0 ? s.trim() : s));
}

// OBSIDIAN API
async function file_name_path(file_name) {
  const file_path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${file_name}.md`))
    .map((file) => file.path)[0];
  return file_path;
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

const file_name_dir = (file_name, file_path) =>
  file_path.replace(`${file_name}.md`, '');

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format('YYYY-MM-DD[T]HH:mm');
const date_modified = moment().format('YYYY-MM-DD[T]HH:mm');

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
const null_arr = ['', 'null', '[[null|Null]]', null];
const null_link = '[[null|Null]]';
const null_yaml_li = yaml_li(null_link);
const null_obj = {
  index: 0,
  key: 'Null',
  value: 'null',
  value_link: null_yaml_li,
};

//-------------------------------------------------------------------
// SET PROGRAMMING LANGUAGE AND FILE CLASS
//-------------------------------------------------------------------
const file_class = 'pkm_code';

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

const lang_name = 'Python';
const lang_value = 'python';
const lang_value_short = 'py';
const topic_value_link = yaml_li(link_alias(lang_value, lang_name));

//-------------------------------------------------------------------
// LANGUAGE (TOPIC) METADATA CACHE
//-------------------------------------------------------------------
const lang_file_path = await file_name_path(lang_value);
const lang_file_dir = file_name_dir(lang_value, lang_file_path);
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
const subtopic_value_link = yaml_li(link_alias(subtopic_value, subtopic_name));
const subtopic_value_ext = md_ext(subtopic_value);

let subtopic_file_path;
let subtopic_dir;
if (subtopic_value != 'null') {
  subtopic_file_path = await file_name_path(subtopic_value);
  subtopic_dir = file_name_dir(subtopic_value, subtopic_file_path);
}

const py_sublang_obj_arr = [
  { key: 'Beautiful Soup', value: 'beautiful_soup', short: 'soup' },
  { key: 'Datetime', value: 'datetime', short: 'dt' },
  { key: 'Functools', value: 'functools', short: 'ft' },
  { key: 'Itertools', value: 'itertools', short: 'it' },
  { key: 'JSON', value: 'json', short: 'json' },
  { key: 'Matplotlib', value: 'matplotlib', short: 'plt' },
  { key: 'NumPy', value: 'numpy', short: 'np' },
  { key: 'pandas', value: 'pandas', short: 'pd' },
  { key: 'Pyjanitor', value: 'pyjanitor', short: 'janitor' },
  { key: 'scikit-learn', value: 'scikit_learn', short: 'sklearn' },
  { key: 'Seaborn', value: 'seaborn', short: 'sns' },
];

const py_sublang_obj = py_sublang_obj_arr.filter((x) =>
  subtopic_value.includes(x.value)
);
const py_sublang_value = py_sublang_obj.map((x) => x.value);
const py_sublang_key = py_sublang_obj.map((x) => x.key);
const py_sublang_short = py_sublang_obj.map((x) => x.short);

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
] = pkm_tree_obj_arr.map((tree) => tree.value_link).reverse();

//-------------------------------------------------------------------
// SET TYPE
//-------------------------------------------------------------------
const type_obj_arr = [
  { key: 'Decorator', value: 'decorator', short: 'dec' },
  { key: 'Expression', value: 'expression', short: 'expr' },
  { key: 'Function', value: 'function', short: 'func' },
  { key: 'Keyword', value: 'keyword', short: 'kw' },
  { key: 'Method', value: 'method', short: 'mthd' },
  { key: 'Operator', value: 'operator', short: 'opr' },
  { key: 'Statement', value: 'statement', short: 'stmt' },
];

const type_obj = await tp.system.suggester(
  (item) => item.key,
  type_obj_arr,
  false,
  'Code note type?'
);

const type_name = type_obj.key;
const type_value = type_obj.value;
const type_value_short = type_obj.short;

//-------------------------------------------------------------------
// SET SUBTYPE
//-------------------------------------------------------------------
const subtype_obj_arr = [
  { key: 'Null', value: 'null', short: 'null' },
  { key: 'Aggregate', value: 'aggregate', short: 'agg' },
  { key: 'Boolean', value: 'boolean', short: 'bool' },
  { key: 'Class', value: 'class', short: 'class' },
  { key: 'Constructor', value: 'constructor', short: 'ctor' },
  { key: 'DataFrame', value: 'dataframe', short: 'df' },
  { key: 'Date', value: 'date', short: 'date' },
  { key: 'Dictionary', value: 'dictionary', short: 'dict' },
  { key: 'File', value: 'file', short: 'file' },
  { key: 'Filter', value: 'filter', short: 'fltr' },
  { key: 'Iterable', value: 'iterable', short: 'iter' },
  { key: 'List', value: 'list', short: 'list' },
  { key: 'Logical', value: 'logical', short: 'logic' },
  { key: 'Math', value: 'math', short: 'math' },
  { key: 'Numeric', value: 'numeric', short: 'num' },
  { key: 'Object', value: 'object', short: 'obj' },
  { key: 'Object-Oriented', value: 'object-oriented', short: 'oop' },
  { key: 'Operator', value: 'operator', short: 'opr' },
  { key: 'Regular Expression', value: 'regex', short: 'regex' },
  { key: 'Series', value: 'series', short: 'ser' },
  { key: 'Set', value: 'set', short: 'set' },
  { key: 'String', value: 'string', short: 'str' },
  { key: 'Tuple', value: 'tuple', short: 'tupl' },
  { key: 'General', value: 'general', short: 'general' },
];

const subtype_obj = await tp.system.suggester(
  (item) => item.key,
  subtype_obj_arr,
  false,
  `${type_name} subtype?`
);

const subtype_name = subtype_obj.key;
const subtype_value = subtype_obj.value;
const subtype_value_short = subtype_obj.short;

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith('Untitled');
let title;
if (!has_title) {
  title = await tp.system.prompt(
    `${lang_name} ${type_value} Title?`,
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
const data_lang_arr = ['google_sheets', 'ms_excel', 'sql', 'tableau'];
const prog_lang_arr = ['css', 'git', 'html', 'python'];
if (data_lang_arr.includes(lang_value)) {
  title.toUpperCase();
} else if (prog_lang_arr.includes(lang_value)) {
  title.toLowerCase();
}

let title_name = title;
let main_title_arr = [title];
if (type_value == 'function') {
  title_name = `${title}()`;
} else if (type_value == 'method') {
  title_name = `${subtype_value_short}.${title}()`;
} else if (type_value == 'element') {
  title_name = `<${title}>`;
}
if (title_name != title) {
  main_title_arr = [title_name, title];
}

const title_value = title.replaceAll(/\s/g, '_').toLowerCase();

let short_title_name = `${lang_name} ${title_name}`;
let short_title_value = `${lang_value}_${title_value}`;
let file_name_prefix = `${lang_value_short}_${type_value_short}`;
if (!(subtopic_value.includes('standard') || subtopic_value.includes('null'))) {
  short_title_name = `${lang_name} ${py_sublang_key} ${title_name}`;
  short_title_value = `${lang_value}_${py_sublang_value}_${title_value}`;
  file_name_prefix = `${lang_value_short}_${py_sublang_short}_${type_value_short}`;
}
let full_title_name = `${short_title_name} ${subtype_name} ${type_name}`;
let full_title_value = `${short_title_value}_${subtype_value}_${type_value}`;
let partial_title_name = `${short_title_name} ${type_name}`;
let partial_title_value = `${short_title_value}_${type_value}`;
if (type_value == 'method') {
  full_title_name = `${short_title_name} ${subtype_name} ${type_name}`;
  full_title_value = `${lang_value}_${subtype_value}_${title_value}_${type_value}`;
  partial_title_name = `${short_title_name} ${type_name}`;
  partial_title_value = `${short_title_value}_${type_value}`;
}

let file_name = `${file_name_prefix}_${subtype_value_short}_${title_value}`;
const full_title_arr = [full_title_name, full_title_value];
const partial_title_arr = [partial_title_name, partial_title_value];
const short_title_arr = [short_title_name, short_title_value];

let content_title = full_title_name;
let alias_arr;
if (subtype_value == 'null' || subtype_value == 'general') {
  file_name = `${file_name_prefix}_${title_value}`;
  alias_arr = [...main_title_arr, ...partial_title_arr, ...short_title_arr];
  alias_arr.push(file_name);
  content_title = partial_title_name;
} else {
  alias_arr = [
    ...main_title_arr,
    ...full_title_arr,
    ...partial_title_arr,
    ...short_title_arr,
  ];
}
alias_arr.push(file_name);
const file_alias = new_line + alias_arr.map((x) => yaml_li(x)).join('');

const file_section = file_name + hash;

//-------------------------------------------------------------------
// SET SYNTAX
//-------------------------------------------------------------------
let syntax = await tp.system.prompt(`${type_name} Syntax?`);
const syntax_parameters = syntax.replace(/^.+\(/g, '(');
if (type_value == 'method') {
  syntax = `${subtype_value_short}.${title}${syntax_parameters}`;
}
const syntax_value = syntax;

//-------------------------------------------------------------------
// PARAMETER TABLE
//-------------------------------------------------------------------
const parameter_char_include = /[^\s\w\d\*,_=\\-]/g;
const parameters = syntax_parameters
  .replaceAll(parameter_char_include, '')
  .replaceAll(/\*/g, '\\*');
const parameter_arr = parameters.split(',').map((x) => x.trim());

const parameter_body = [];
let ol_parameter_count = 0;
for (let i = 0; i < parameter_arr.length; i++) {
  ol_parameter_count = ol_parameter_count + 1;
  parameter_name = `${call_start}${ol_parameter_count}.${space}${code_inline(
    parameter_arr[i]
  )}`;
  parameter_section = [
    parameter_name,
    call_ul_indent + '**Type**:',
    call_ul_indent + '**Description**:',
  ].join(new_line);
  parameter_body.push(parameter_section);
}

const parameter_callout = [
  call_title('param', type_name + ' Parameters'),
  call_start,
  parameter_body.join(new_line),
].join(new_line);

/* ---------------------------------------------------------- */
/*                       SET DESCRIPTION                      */
/* ---------------------------------------------------------- */
const about = await tp.system.prompt(
  `${type_name} Description?`,
  null,
  false,
  true
);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, '$2')
  .replaceAll(/(\s*)\n/g, '\n')
  .replaceAll(/([^\s])(\s*)$/g, '$1')
  .replaceAll(/\n{1,6}/g, '<new_line>')
  .replaceAll(/(<new_line>)(\w)/g, '\n \n $2')
  .replaceAll(/(<new_line>)(\d\.\s)/g, '\n $2')
  .replaceAll(/(<new_line>)((·|\*|-)\s)/g, '\n - ');

/* ---------------------------------------------------------- */
/*                      SET REFERENCE URL                     */
/* ---------------------------------------------------------- */
const url = await tp.system.prompt(`${type_name} Reference URL?`);

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
//temp_file_path = `${sys_temp_include_dir}${note_status}.md`;
//abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
//tp_include = await tp.file.include(abstract_file);
//include_arr = tp_include.toString().split(";");

//const status_name = include_arr[0];
const status_value = 'resource';

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
    head_key: 'Code Information',
    toc_key: 'Code Info',
    file: null,
    content: null,
  },
  {
    head_key: 'Related Snippets',
    toc_key: 'Use Cases',
    file: '100_71_related_pkm_snip_sect_func_file',
    content: null,
  },
  {
    head_key: 'Related Code',
    toc_key: 'Related Code',
    file: '100_71_related_pkm_code_sect_func_file',
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
// CODE INFORMATION SECTION
//-------------------------------------------------------------------
section_obj_arr[0].content =
  [
    head_lvl(3, 'Syntax'),
    code_block(lang_value, syntax),
    head_lvl(3, 'Parameter Values'),
    parameter_callout,
    head_lvl(3, 'Additional Info'),
    cmnt_html('Add supplementary info here'),
    head_lvl(4, 'Examples'),
    code_block(lang_value, ''),
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
syntax: |
 <%* tR += syntax_value %>
about: |
 <%* tR += about_value %>
status: <%* tR += status_value %>
subtype: <%* tR += subtype_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += content_title %>

<%* tR += info %>
<%* tR += sections_content %>
