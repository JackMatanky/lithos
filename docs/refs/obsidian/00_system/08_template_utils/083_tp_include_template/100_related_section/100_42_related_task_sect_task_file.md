<%*
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

const code_inline = (content) => backtick + content + backtick;
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

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS BUTTON AND BUTTON CALLOUT
//-------------------------------------------------------------------
const button_start = [
  cmnt_html('Adjust replace lines'),
  three_backtick + 'button',
].join(two_new_line);
const button_end = three_backtick;

const button = [
  button_start,
  'name ✅Related Tasks and Events',
  'type append template',
  'action 100_50_dvmd_related_task_sect',
  'replace [1, 2]',
  'color blue',
  button_end,
].join(new_line);

//-------------------------------------------------------------------
// SET RELATED PROJECT FILE NAME AND ALIAS
//-------------------------------------------------------------------
// Projects directory
const project_path_arr = [
  '41_personal/',
  '42_education/',
  '43_professional/',
  '44_work/',
  '45_habit_ritual/',
];

// Boolean Arrays of Objects
const bool_obj_arr = [
  { key: '✔️ YES ✔️', value: 'yes' },
  { key: '❌ NO ❌', value: 'no' },
];

// Filter array to only include projects in the Projects Directory
let file_name_alias_obj_arr = [{ key: 'Null', value: 'null' }];
for (let i = 0; i < project_path_arr.length; i++) {
  const obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_path_arr[i],
    file_class: 'task',
    type: 'project',
  });
  file_name_alias_obj_arr.push(
    ...obj_arr.filter((x) => !['null', '_user_input'].includes(x.value))
  );
}

let file_obj_arr = [];
let file_filter_arr = [];
for (let i = 0; i < 10; i++) {
  // File Suggester
  const file_name_alias_obj = await tp.system.suggester(
    (item) => item.key,
    file_name_alias_obj_arr.filter(
      (file) => !file_filter_arr.includes(file.value)
    ),
    false,
    'Related Project? (Related Tasks and Events Section)'
  );

  if (file_name_alias_obj.value == 'null' && file_filter_arr.length == 0) {
    file_obj_arr.push(file_name_alias_obj);
    break;
  } else if (file_name_alias_obj.value == 'null') {
    break;
  }
  file_obj_arr.push(file_name_alias_obj);
  file_filter_arr.push(file_name_alias_obj.value);

  const bool_obj = await tp.system.suggester(
    (item) => item.key,
    bool_obj_arr,
    false,
    'Another Related Project?'
  );

  if (bool_obj.value == 'no') {
    break;
  }
}

const project_link = file_obj_arr
  .map((file) => new_line + ul + link_alias(file.value, file.key))
  .join('');

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION
//-------------------------------------------------------------------
let outlink = [
  head_lvl(3, 'Outgoing Task and Events Links'),
  cmnt_html('Link related tasks and events here'),
].join(two_new_line);
if (!project_link.endsWith(link_alias('null', 'Null'))) {
  outlink = [outlink, project_link].join(two_new_line);
}

const head_query_arr = ['Projects', 'Parent Tasks', 'Child Tasks'];
const query_section = [];
for (let i = 0; i < head_query_arr.length; i++) {
  const head = head_lvl(3, head_query_arr[i]);
  const query_type = head_query_arr[i]
    .toLowerCase()
    .replace(/s$/g, '')
    .replace(/\s/g, '_');
  const query = await tp.user.dv_task_linked({
    type: query_type,
    status: '',
    relation: 'link',
    md: 'false',
  });
  const section = [head, query].join(two_new_line);
  query_section.push(section);
}

const section =
  [button, outlink, query_section.join(two_new_line), hr_line].join(
    two_new_line
  ) + new_line;

tR += section;
%>
