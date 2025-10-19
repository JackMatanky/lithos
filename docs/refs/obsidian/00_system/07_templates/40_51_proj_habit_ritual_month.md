<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = '20_pillars/';
const goals_dir = '30_goals/';
const habit_ritual_proj_dir = '45_habit_ritual/';
const contacts_dir = '51_contacts/';
const organizations_dir = '52_organizations/';

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
// GENERAL VARIABLES
//-------------------------------------------------------------------
const dv_yaml = 'file.frontmatter';
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

const pillar_mental_health = link_alias('mental_health', 'Mental Health');
const pillar_physical_health = link_alias('physical_health', 'Physical Health');
const pillar_mental_physical_health = [
  pillar_mental_health,
  pillar_physical_health,
]
  .map((x) => yaml_li(x))
  .join('');

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = 'Habits and Rituals';
const context_value = context_name
  .replaceAll(/s\sand\s/g, '_')
  .replaceAll(/s$/g, '')
  .toLowerCase();
const context_dir = '45_habit_ritual/';

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

//-------------------------------------------------------------------
// SET THE DATE
//-------------------------------------------------------------------
const date_obj_arr = [
  { key: 'Current Month', value: 'current' },
  { key: 'Last Month', value: 'last' },
  { key: 'Next Month', value: 'next' },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  'Which Month?'
);

const date_value = date_obj.value;

let full_date = '';

if (date_value.startsWith('current')) {
  full_date = moment();
} else if (date_value.startsWith('next')) {
  full_date = moment().add(1, 'months');
} else {
  full_date = moment().subtract(1, 'months');
}

/* ---------------------------------------------------------- */
/*                   SET START AND END DATES                  */
/* ---------------------------------------------------------- */
const task_start = moment(full_date).startOf('month').format('YYYY-MM-DD');
const task_start_link = `"[[${task_start}]]"`;
const task_end = moment(full_date).endOf('month').format('YYYY-MM-DD');
const task_end_link = `"[[${task_end}]]"`;

//-------------------------------------------------------------------
// PROJECT TITLES, ALIAS, FILE NAME, AND DIRECTORY
//-------------------------------------------------------------------
const year_month_short = moment(full_date).format('YYYY-MM');
const year_month_long = moment(full_date).format("MMMM [']YY");
const year_month_value = moment(full_date).format('MMMM_YY');

const full_title_name = `${year_month_long} ${context_name}`;
const short_title_name = `${year_month_short} ${context_name}`;
const full_title_value = `${year_month_value.toLowerCase()}_${context_value}`;
const short_title_value = `${year_month_short}_${context_value}`;

const file_alias =
  new_line +
  [full_title_name, short_title_name, full_title_value, short_title_value]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

const file_name = short_title_value;
const file_section = file_name + hash;
const file_value_link = yaml_li_link(file_name, full_title_name);

const project_dir = `${context_dir}${file_name}/`;

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  '20_03_pillar_name_alias_preset_mental'
);
const pillar_value = pillar_name_alias.split(';')[0];
const pillar_value_link = pillar_name_alias.split(';')[1];

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

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const org_name_alias = await tp.user.include_template(
  tp,
  '52_organization_name_alias'
);
const organization_value = org_name_alias.split(';')[0];
const organization_value_link = org_name_alias.split(';')[1];

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
const contact_name_alias = await tp.user.include_template(
  tp,
  '51_contact_name_alias'
);
const contact_value = contact_name_alias.split(';')[0];
const contact_name = contact_name_alias.split(';')[1];
const contact_value_link = contact_name_alias.split(';')[2];

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const due_do_value = 'do';

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, '40_task_status');
const status_value = task_status.split(';')[0];
const status_name = task_status.split(';')[1];
const status_symbol = task_status.split(';')[2];

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
    file: '140_50_related_task_sect_proj_habit_rit',
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
}
section_obj_arr.map((x) => (x.head = head_lvl(2, x.head_key)));
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
const proj_info = await tp.user.include_file('40_00_project_info_callout');

//-------------------------------------------------------------------
// PROJECT FRONTMATTER YAML PROPERTIES
//-------------------------------------------------------------------
const yaml_proj = [
  hr_line,
  `title:${space}${file_name}`,
  `uuid:${space}${await tp.user.uuid()}`,
  `aliases:${space}${file_alias}`,
  `task_start:${space}${task_start_link}`,
  `task_end:${space}${task_end_link}`,
  `due_do:${space}${due_do_value}`,
  `pillar:${pillar_value_link}`,
  `context:${space}${context_value}`,
  `goal:${space}${goal}`,
  `organization:${organization_value_link}`,
  `contact:${contact_value_link}`,
  'library:',
  `status:${space}${status_value}`,
  `type:${space}${proj_type_value}`,
  `file_class:${space}${proj_file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  'tags:',
  hr_line,
].join(new_line);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = project_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

//-------------------------------------------------------------------
// HABITS AND RITUALS PARENT TASKS ARRAY OF OBJECTS
//-------------------------------------------------------------------
const parent_obj_arr = [
  {
    order: '01',
    name: 'Habits',
    type: 'habit',
    pillar: pillar_mental_physical_health,
  },
  {
    order: '02',
    name: 'Morning Rituals',
    type: 'morning_ritual',
    pillar: yaml_li(pillar_mental_health),
  },
  {
    order: '03',
    name: 'Workday Startup Rituals',
    type: 'workday_startup_ritual',
    pillar: yaml_li(null_link),
  },
  {
    order: '04',
    name: 'Workday Shutdown Rituals',
    type: 'workday_shutdown_ritual',
    pillar: yaml_li(null_link),
  },
  {
    order: '05',
    name: 'Evening Rituals',
    type: 'evening_ritual',
    pillar: yaml_li(pillar_mental_health),
  },
];

//-------------------------------------------------------------------
// PILLAR, ORGANIZATION, AND CONTACT VARIABLES
//-------------------------------------------------------------------
const pillars_obj_arr = await tp.user.file_by_status({
  dir: pillars_dir,
  status: 'active',
});
const organizations_obj_arr = await tp.user.md_file_name_alias(
  organizations_dir
);
const contact_obj_arr = await tp.user.md_file_name_alias(contacts_dir);

/* ------------ PARENT TASK FILE DETAILS CALLOUT ------------ */
const parent_info = await tp.user.include_file(
  '41_00_parent_task_info_callout'
);

/* --------- PARENT TASK PREVIEW AND REVIEW SECTION --------- */
section_obj_arr[0].content = await tp.user.include_template(
  tp,
  '41_50_parent_habit_rit_preview_review'
);

//-------------------------------------------------------------------
// LOOP THROUGH ARRAY OF OBJECTS
//-------------------------------------------------------------------
for (let i = 0; i < parent_obj_arr.length; i++) {
  // TITLES, ALIAS, AND FILE NAME
  // Titles
  const par_name = parent_obj_arr[i].name;
  const par_value = par_name.replaceAll(/\s/g, '_').toLowerCase();
  const par_order = parent_obj_arr[i].order;

  const par_full_title_name = `${year_month_long} ${par_name}`;
  const par_short_title_name = `${year_month_short} ${par_name}`;
  const par_full_title_value = `${year_month_value.toLowerCase()}_${par_value}`;
  const par_short_title_value = `${year_month_short}_${par_order}_${par_value}`;

  // Alias
  const par_alias =
    new_line +
    [
      par_full_title_name,
      par_short_title_name,
      par_full_title_value,
      par_short_title_value,
    ]
      .map((x) => `${ul_yaml}"${x}"`)
      .join(new_line);

  // File name
  const par_file_name = par_short_title_value;

  // SET PILLAR FILE AND FULL NAME
  const par_pillar_link = parent_obj_arr[i].pillar;

  // SET GOAL
  const par_goal = await tp.system.suggester(
    goals,
    goals,
    false,
    `Goal for Monthly ${par_name}?`
  );

  // SET ORGANIZATION FILE NAME AND TITLE
  const organizations_obj = await tp.system.suggester(
    (item) => item.key,
    organizations_obj_arr,
    false,
    `Organization for Monthly ${par_name}?`
  );
  let par_organization_value = organizations_obj.value;
  let par_organization_name = organizations_obj.key;

  if (par_organization_value.includes('_user_input')) {
    par_organization_name = await tp.system.prompt(
      `Organization for Monthly ${par_name}?`,
      '',
      false,
      false
    );
    par_organization_value = parent_organization_name
      .replaceAll(/[,']/g, '')
      .replaceAll(/\s/g, '_')
      .replaceAll(/\//g, '-')
      .replaceAll(/&/g, 'and')
      .toLowerCase();
  }
  const par_organization_value_link = yaml_li_link(
    par_organization_value,
    par_organization_name
  );

  // SET CONTACT FILE NAME AND TITLE
  const contact_obj = await tp.system.suggester(
    (item) => item.key,
    contact_obj_arr,
    false,
    `Contact for Monthly ${par_name}?`
  );

  let par_contact_value = contact_obj.value;
  let par_contact_name = contact_obj.key;
  if (par_contact_value.includes('_user_input')) {
    const contact_names = await tp.user.dirContactNames(tp);
    const full_name = contact_names.fullName;
    const last_first_name = contact_names.lastFirstName;
    par_contact_value = full_name;
    par_contact_value = last_first_name.replaceAll(/[^\w]/g, '*').toLowerCase();
  }
  const par_contact_value_link = yaml_li_link(
    par_contact_value,
    par_contact_name
  );

  // DATAVIEW TASK TABLES
  // TYPES: "parent", "child"
  // STATUS: "due", "done", "null"
  heading = head_lvl(3, 'Project');
  query = await tp.user.dv_task_linked({
    type: 'parent',
    status: '',
    relation: 'in_parent',
    md: 'false',
  });
  const project = `${heading}${two_new_line}${query}${two_new_line}`;

  heading = head_lvl(3, `Remaining${space}${par_name}`);
  query = await tp.user.dv_task_linked({
    type: 'parent',
    status: 'due',
    relation: 'in_hab_rit',
    md: 'false',
  });
  const remaining = `${heading}${two_new_line}${query}${two_new_line}`;

  heading = head_lvl(3, `Completed${space}${par_name}`);
  query = await tp.user.dv_task_linked({
    type: 'parent',
    status: '',
    relation: 'in_hab_rit',
    md: 'false',
  });
  const completed = `${heading}${two_new_line}${query}`;
  const related_parent_task_section =
    [project, remaining, completed, hr_line].join(two_new_line) + new_line;
  section_obj_arr[1].content = related_parent_task_section;

  // TOC CALLOUT
  const toc_parent = toc.replaceAll(file_name, par_file_name);

  // PARENT TASK FILE SECTIONS
  const sections_content_par = section_obj_arr
    .map((s) => [s.head, toc_parent, s.content].join(two_new_line))
    .join(new_line);

  // PARENT TASK FRONTMATTER YAML PROPERTIES
  const yaml_parent = [
    hr_line,
    `title:${space}${par_file_name}`,
    `uuid:${space}${await tp.user.uuid()}`,
    `aliases:${space}${par_alias}`,
    `task_start:${space}${task_start_link}`,
    `task_end:${space}${task_end_link}`,
    `due_do:${space}${due_do_value}`,
    `pillar:${space}${par_pillar_link}`,
    `context:${space}${context_value}`,
    `goal:${space}${par_goal}`,
    `project:${file_value_link}`,
    `organization:${par_organization_value_link}`,
    `contact:${par_contact_value_link}`,
    'library:',
    `status:${space}${status_value}`,
    `type:${space}${parent_obj_arr[i].type}`,
    `file_class:${space}${parent_file_class}`,
    `date_created:${space}${date_created}`,
    `date_modified:${space}${date_modified}`,
    'tags:',
    hr_line,
  ].join(new_line);

  // FILE CONTENT
  const file_content = [
    yaml_parent,
    head_lvl(1, par_full_title_name) + new_line,
    parent_info,
    sections_content_par,
  ].join(new_line);

  // PARENT TASK DIRECTORY AND FILE PATH AND CREATION
  const parent_directory = `${project_dir}${par_file_name}`;
  await this.app.vault.createFolder(parent_directory);

  const file_path = `${parent_directory}/${par_file_name}.md`;
  await this.app.vault.create(file_path, file_content);
}

tR += yaml_proj;
%>
# <%* tR += full_title_name %>

<%* tR += proj_info %>
<%* tR += sections_content %>
