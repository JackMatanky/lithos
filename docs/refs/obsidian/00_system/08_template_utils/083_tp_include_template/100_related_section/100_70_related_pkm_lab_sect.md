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
// RELATED NOTES BUTTON AND BUTTONS CALLOUT
//-------------------------------------------------------------------
const button_start = [
  cmnt_html('Adjust replace lines'),
  three_backtick + 'button',
].join(two_new_line);
const button_end = three_backtick;

const button = [
  button_start,
  'name 🗃️Related Notes',
  'type append template',
  'action 100_71_dvmd_related_note_sect',
  'replace [1, 2]',
  'color purple',
  button_end,
].join(new_line);

const button_callout = await tp.user.include_file(
  '00_80_buttons_callout_notes'
);

//-------------------------------------------------------------------
// SET RELATED PKM FILE NAME AND ALIAS
//-------------------------------------------------------------------
// Projects directory
const pkm_path = '70_pkm/';

const pkm_file_class_arr = ['pkm_zettel', 'pkm_info'];

// Boolean Arrays of Objects
const bool_obj_arr = [
  { key: '✔️ YES ✔️', value: 'yes' },
  { key: '❌ NO ❌', value: 'no' },
];

// Filter array to only include projects in the Projects Directory
let file_name_alias_obj_arr = [{ key: 'Null', value: 'null' }];
for (let i = 0; i < pkm_file_class_arr.length; i++) {
  const obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: pkm_path,
    file_class: pkm_file_class_arr[i],
    type: '',
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
    'Related Related PKM Note File?'
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
    'Another Related PKM Note?'
  );

  if (bool_obj.value == 'no') {
    break;
  }
}

const pkm_lab_link = file_obj_arr
  .map((file) => new_line + ul + link_alias(file.value, file.key))
  .join('');

/* ---------------------------------------------------------- */
/*                    RELATED NOTES SECTION                   */
/* ---------------------------------------------------------- */

/* --------------------- OUTLINK SECTION -------------------- */
// let outlink_section = [
//   head_lvl(3, 'Outgoing Note Links'),
//   cmnt_html('Link related notes here'),
// ].join(two_new_line);
// if (!pkm_lab_link.endsWith(link_alias('null', 'Null'))) {
//   outlink_section = [outlink_section, pkm_lab_link].join(two_new_line);
// }

const outlink_section = [
  head_lvl(3, 'Outgoing Note Links'),
  cmnt_html('Link related notes here'),
  !pkm_lab_link.endsWith(link_alias('null', 'Null')) && pkm_lab_link,
]
  .filter(Boolean)
  .join(two_new_line);

/* ------------ DYNAMIC DATAVIEW QUERIES SECTION ------------ */
const head_query_arr = ['Permanent', 'Literature', 'Fleeting', 'Info'];

const query_sections = await Promise.all(
  head_query_arr.map(async (title) => {
    const head = head_lvl(3, title);
    const query = await tp.user.dv_pkm_linked({
      type: title.toLowerCase(),
      status: '',
      relation: 'link',
      md: 'false',
    });
    return [head, query].join(two_new_line);
  })
);

/* --------------------- FINAL ASSEMBLY --------------------- */
const final_output =
  [
    button_callout,
    button,
    outlink_section,
    query_sections.join(two_new_line),
    hr_line,
  ].join(two_new_line) + new_line;

tR += final_output;
%>
