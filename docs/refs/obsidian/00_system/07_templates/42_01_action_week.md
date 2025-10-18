<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const projects_dir = "40_projects/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";

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
const task_date_created = moment().format("YYYY-MM-DD");

//-------------------------------------------------------------------
// SET THE WEEK
//-------------------------------------------------------------------
const date_obj_arr = [
  { key: "Current Week", value: "current" },
  { key: "Last Week", value: "last" },
  { key: "Next Week", value: "next" },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  "Which Week?"
);

const date_value = date_obj.value;

let full_date = "";
if (date_value.startsWith("current")) {
  full_date = moment();
} else if (date_value.startsWith("next")) {
  full_date = moment().add(1, "week");
} else {
  full_date = moment().subtract(1, "week");
}
const moment_day = (day_int) =>
  moment(full_date).day(day_int).format("YYYY-MM-DD");

//-------------------------------------------------------------------
// GENERAL VARIABLES AND FUNCTIONS
//-------------------------------------------------------------------
const null_link = link_alias("null", "Null");
const null_yaml_li = yaml_li(null_link);
const null_arr = ["", "null", null_yaml_li, null];

const dv_yaml = "file.frontmatter";
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

async function metadata_alias(file_name_value) {
  const name_ext = file_name_value + ".md";
  const path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${name_ext}`))
    .map((file) => file.path)[0];
  const abstract_file = await app.vault.getAbstractFileByPath(path);
  const file_cache = await app.metadataCache.getFileCache(abstract_file);
  return file_cache?.frontmatter?.aliases[0];
}

const pillar_mental_health = "[[mental_health|Mental Health]]";
const pillar_physical_health = "[[physical_health|Physical Health]]";
const pillar_knowledge = "[[knowledge_expansion|Knowledge Expansion]]";
const pillar_career = "[[career_development|Career Development]]";
const pillar_data = "[[data_analyst|Data Analyst]]";
const pillar_course = [pillar_knowledge, pillar_career, pillar_data]
  .map((x) => yaml_li(x))
  .join("");

//-------------------------------------------------------------------
// ACTION ITEM TAG, TYPE, AND FILE CLASS
//-------------------------------------------------------------------
const task_tag = "#task";
const type_name = "Action Item";
const type_value = type_name.replaceAll(/\s/g, "_").toLowerCase();
const file_class = "task_child";

/* ---------------------------------------------------------- */
/*          TITLE AND PROJECT DIRECTORY OBJECT ARRAYS         */
/* ---------------------------------------------------------- */
const personal_obj_arr = [
  {
    key: "Coaching Assignments",
    value: "coaching_assignments_input",
    due_do: "do",
    pillar: yaml_li(pillar_mental_health),
    project: "coaching_with_nir_zer",
    parent_task: "coaching_assignments",
    organization: null_link,
    contact: "[[zer_nir|Nir Zer]]",
  },
  {
    key: "Collect Prescriptions",
    value: "collect_prescriptions",
    due_do: "do",
    pillar: yaml_li(pillar_physical_health),
    project: "medical",
    parent_task: "medical_prescriptions",
    organization: "[[clalit_health_services|Clalit Health Services]]",
    contact: null_link,
  },
  {
    key: "Convert SuperMemo to Anki",
    value: "convert_supermemo_to_anki",
    due_do: "do",
    pillar: null_yaml_li,
    project: "obsidian_workspace_development",
    parent_task: "setup_obsidian_spaced_repetition_system",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Create Template",
    value: "create_template",
    due_do: "do",
    pillar: null_yaml_li,
    project: "obsidian_workspace_development",
    parent_task: "template_development",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Revise Template",
    value: "revise_template",
    due_do: "do",
    pillar: null_yaml_li,
    project: "obsidian_workspace_development",
    parent_task: "template_development",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "House Chores",
    value: "house_chores",
    due_do: "do",
    pillar: yaml_li("[[partner|Partner]]"),
    project: "general_tasks_and_events",
    parent_task: "house_chores",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Pick Up Mail",
    value: "pick_up_mail",
    due_do: "do",
    pillar: null_yaml_li,
    project: "general_tasks_and_events",
    parent_task: "pick_up_mail",
    organization: "[[israel_post|Israel Post]]",
    contact: null_link,
  },
  {
    key: "Revise Ergogen Config",
    value: "revise_ergogen_config",
    due_do: "do",
    pillar: null_yaml_li,
    project: "keyboard_development",
    parent_task: "ergogen_pcb_development",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Revise ZMK Config",
    value: "revise_zmk_config",
    due_do: "do",
    pillar: null_yaml_li,
    project: "keyboard_development",
    parent_task: "zmk_keyboard_layout_development",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Typing Practice",
    value: "typing_practice",
    due_do: "do",
    pillar: null_yaml_li,
    project: "keyboard_development",
    parent_task: "learn_keymap_layout",
    organization: null_link,
    contact: null_link,
  },
  { key: "Personal Task", value: "personal_input" },
];

const chores_arr = [
  "Fold Laundry",
  "Wash Laundry",
  "Buy Groceries",
  "Cook Dinner",
  "Wash Dishes",
  "Clean the Apartment",
];

const education_obj_arr = [
  { key: "Learn Book Chapter", value: "learn_chapter" },
  { key: "Review Book Chapter", value: "review_chapter" },
  { key: "Learn Course Unit", value: "learn_course" },
  {
    key: "NAYA College Data Science Course Unit",
    value: "learn_naya_course",
    due_do: "do",
    pillar: pillar_course,
    project: "course_naya_college_practical_data_science",
    organization: null_link,
    contact: null_link,
  },
  { key: "Watch Course Lecture", value: "watch_course" },
  { key: "Watch Educational Video", value: "watch" },
  { key: "Read Content", value: "read" },
  { key: "Education Task", value: "education_input" },
];

const professional_obj_arr = [
  {
    key: "First Job Application Assignment",
    value: "first_job_assignment",
    due_do: "due",
    project: "job_hunting_2023",
  },
  {
    key: "Second Job Application Assignment",
    value: "second_job_assignment",
    due_do: "due",
    project: "job_hunting_2023",
  },
  {
    key: "Third Job Application Assignment",
    value: "third_job_assignment",
    due_do: "due",
    project: "job_hunting_2023",
  },
  {
    key: "Interview Preparation",
    value: "interview_prep",
    due_do: "due",
    project: "job_hunting_2023",
  },
  {
    key: "Daily Job Search",
    value: "daily_job_search",
    due_do: "do",
    pillar: yaml_li("[[career_development|Career Development]]"),
    project: "job_hunting_2023",
    parent_task: "daily_job_search_2023",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Networking Meeting Preparation",
    value: "network_meeting_prep",
    due_do: "do",
    project: "networking",
  },
  { key: "Professional Task", value: "professional_input" },
];

const work_obj_arr = [{ key: "Work Task", value: "work_input" }];

const project_setup_obj_arr = [
  {
    key: "Project Setup: Write Objective",
    value: "proj_setup_objective",
    due_do: "do",
    pillar: null_yaml_li,
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Project Setup: Create Parent Tasks",
    value: "proj_setup_parent_tasks",
    due_do: "do",
    pillar: null_yaml_li,
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Project Setup: Gather Resources",
    value: "proj_setup_resources",
    due_do: "do",
    pillar: null_yaml_li,
    organization: null_link,
    contact: null_link,
  },
];

const task_obj_arr = [
  ...personal_obj_arr,
  ...education_obj_arr,
  ...professional_obj_arr,
  ...work_obj_arr,
  ...project_setup_obj_arr,
];

// Sort title object array
task_obj_arr.sort((a, b) => {
  let fa = a.key.toLowerCase(),
    fb = b.key.toLowerCase();

  if (fa < fb) {
    return -1;
  }
  if (fa > fb) {
    return 1;
  }
  return 0;
});

const title_obj_arr = [
  { key: "User Input", value: "_user_input" },
  ...task_obj_arr,
];

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
let title_obj;
let title;
let title_value = "";

const has_title = !tp.file.title.startsWith("Untitled");
if (!has_title) {
  title_obj = await tp.system.suggester(
    (item) => item.key,
    title_obj_arr,
    false,
    `${type_name} Title?`
  );
  title = title_obj.key;
  title_value = title_obj.value;
} else {
  title = tp.file.title;
  title_value = title.trim();
  title = await tp.user.title_case(title_value);
}

if (title_obj.value == "house_chores") {
  title = await tp.system.suggester(
    (item) => item,
    chores_arr,
    false,
    "House Chore?"
  );
} else if (title_value.endsWith("_template")) {
  template_title = await tp.system.prompt(
    `Template to ${title_value.split("_")[0]}?`,
    null,
    true,
    false
  );
  template_title.trim();
  title_split = title.split(" ");
  title = [title_split[0], template_title, title_split[1]].join(space);
} else if (title_value.endsWith("_input")) {
  title = await tp.system.prompt(`${type_name} Title?`, null, true, false);
  title = title.trim();
  title = await tp.user.title_case(title);
}

const title_filter = (filter_value) => {
  let filter = title_obj_arr
    .filter((x) => x.value == title_obj.value)
    .filter((x) => x?.[filter_value]);
  return filter.length == 0 ? null : filter;
};
const title_map = (map_value) =>
  title_obj_arr
    .filter((x) => x.value == title_obj.value)
    .map((x) => x?.[map_value])
    .toString();

//-------------------------------------------------------------------
// WEEKDAY AND DEFAULT TIME VARIABLES
//-------------------------------------------------------------------
const day_obj_arr = [
  { key: "Sunday", value: 0 },
  { key: "Monday", value: 1 },
  { key: "Tuesday", value: 2 },
  { key: "Wednesday", value: 3 },
  { key: "Thursday", value: 4 },
  { key: "All Weekdays", value: [0, 1, 2, 3, 4] },
  { key: "All Weekdays, Minus Sunday", value: [1, 2, 3, 4] },
  { key: "Null", value: null },
];
let day_arr = [];
for (let i = 0; i < 10; i++) {
  const day_suggest_obj = await tp.system.suggester(
    (item) => item.key,
    day_obj_arr.filter((day_int) => !day_arr.includes(day_int.value)),
    false,
    `Days of ${date_obj.key} for Action Item (${title})?`
  );
  const day_key = day_suggest_obj.key;
  const day_value = day_suggest_obj.value;

  if (null_arr.includes(day_value)) {
    if (day_arr) {
      break;
    }
    day_arr.push(day_value);
    break;
  }
  if (day_key.startsWith("All")) {
    day_arr = day_value;
    break;
  }
  day_arr.push(day_value);

  const bool_obj = await tp.system.suggester(
    (item) => item.key,
    bool_obj_arr,
    false,
    "Another Day?"
  );

  if (bool_obj.value == "no") {
    break;
  }
}

const action_day_arr = [...new Set(day_arr)].map((x) => moment_day(x));

/* ---------------------------------------------------------- */
/*                 SET TASK TIME AND DURATION                 */
/* ---------------------------------------------------------- */
const time = await tp.user.nl_time(tp, `${type_name} Start Time?`);
const duration_min = await tp.user.durationMin(tp);

//-------------------------------------------------------------------
// PROJECT AND FILE PATH VARIABLES
//-------------------------------------------------------------------
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split("/");
const folder_path_length = folder_path_split.length;

const project_path_obj_arr = [
  { key: "personal", dir: "41_personal/" },
  { key: "education", dir: "42_education/" },
  { key: "professional", dir: "43_professional/" },
  { key: "work", dir: "44_work/" },
];

let context_value;
const context_include = (obj_arr) =>
  obj_arr.map((el) => el.value).includes(title_obj.value);
if (context_include(personal_obj_arr)) {
  context_value = "personal";
} else if (context_include(education_obj_arr)) {
  context_value = "education";
} else if (context_include(professional_obj_arr)) {
  context_value = "professional";
} else if (context_include(work_obj_arr)) {
  context_value = "work";
} else {
  context_value = null;
}

let project_dir_path;
if (context_value) {
  project_dir_path = project_path_obj_arr
    .filter((path) => path.key == context_value)
    .map((path) => path.dir);
} else {
  project_dir_path = projects_dir;
}

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
let pillar_value_link;
if (title_filter("pillar")) {
  pillar_value_link = title_map("pillar");
} else {
  let pillar_name_alias_file = "20_00_pillar_name_alias";
  if (context_value == "education") {
    pillar_name_alias_file = "20_02_pillar_name_alias_preset_know";
  } else if (context_value == "professional") {
    pillar_name_alias_file = "20_01_pillar_name_alias_preset_career";
  }
  const pillar_name_alias = await tp.user.include_template(
    tp,
    pillar_name_alias_file
  );
  pillar_value = pillar_name_alias.split(";")[0];
  pillar_value_link = pillar_name_alias.split(";")[1];
}

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `${type_name} Goal?`
);

/* ---------------------------------------------------------- */
/*      SET CONTEXT AND PROJECT BY FILE PATH OR SUGGESTER     */
/* ---------------------------------------------------------- */
let project_value;
let project_name;
let project_dir;
if (title_filter("project")) {
  project_value = title_map("project");
  project_name = await metadata_alias(project_value);
  project_dir = `${project_dir_path}${project_value}/`;
} else if (
  projects_dir == `${folder_path_split[0]}/` &&
  folder_path_length >= 3
) {
  project_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[2],
    file_class: "task",
    type: "project",
  });
  project_value = project_obj[1].value;
  project_name = project_obj[1].key;
  project_dir = `${projects_dir}${folder_path_split[1]}/${folder_path_split[2]}/`;
} else {
  project_obj_arr = await tp.user.file_name_alias_by_class_type_status({
    dir: project_dir_path,
    file_class: "task",
    type: "project",
    status: "active",
  });
  project_obj = await tp.system.suggester(
    (item) => item.key,
    project_obj_arr,
    false,
    "Project?"
  );
  project_value = project_obj.value;
  project_name = project_obj.key;
  project_name_ext = `${project_value}.md`;
  project_dir = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${project_name_ext}`))
    .map((file) => file.path)[0]
    .replace(project_name_ext, "");
}
const project_file_path = `${project_dir}${project_value}.md`;
const project_value_link = yaml_li(`[[${project_value}|${project_name}]]`);

if (!context_value) {
  context_value = project_dir.split("/")[1].replace(/^\d\d_/g, "");
}

/* ---------------------------------------------------------- */
/*          SET PARENT TASK BY FILE PATH OR SUGGESTER         */
/* ---------------------------------------------------------- */
let parent_task_value;
let parent_task_name;
if (title_filter("parent_task")) {
  parent_task_value = title_map("parent_task");
  parent_task_name = await metadata_alias(parent_task_value);
} else if (
  projects_dir == `${folder_path_split[0]}/` &&
  folder_path_length >= 4
) {
  parent_task_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[3],
    file_class: "task",
    type: "parent_task",
  });
  parent_task_value = parent_task_obj[1].value;
  parent_task_name = parent_task_obj[1].key;
} else {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_value,
    file_class: "task",
    type: "parent_task",
  });
  parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    "Parent Task?"
  );
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
}
const parent_value_link = yaml_li(
  `[[${parent_task_value}|${parent_task_name}]]`
);
const parent_task_dir = `${project_dir}${parent_task_value}`;
const parent_task_file_path = `${parent_task_dir}/${parent_task_value}.md`;

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
let organization_value_link;
if (title_filter("organization")) {
  organization_value_link = yaml_li(title_map("organization"));
} else if (
  title_value.endsWith("job_assignment") ||
  title_obj.value == "interview_prep"
) {
  abstract_file = await app.vault.getAbstractFileByPath(parent_task_file_path);
  file_cache = await app.metadataCache.getFileCache(abstract_file);
  organization_link = file_cache?.frontmatter?.organization;
  organization_value_link = yaml_li(organization_link);
} else {
  const org_name_alias = await tp.user.include_template(
    tp,
    "52_organization_name_alias"
  );
  organization_value = org_name_alias.split(";")[0];
  organization_value_link = org_name_alias.split(";")[1];
}

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
let contact_value;
let contact_value_link;
if (title_filter("contact")) {
  contact_link = title_map("contact");
  contact_value = contact_link.replaceAll(/.+([\w_].+)\|.+/g, "$1");
  contact_value_link = yaml_li(contact_link);
} else {
  const contact_name_alias = await tp.user.include_template(
    tp,
    "51_contact_name_alias"
  );
  contact_value = contact_name_alias.split(";")[0];
  contact_name = contact_name_alias.split(";")[1];
  contact_value_link = contact_name_alias.split(";")[2];
}

/* ---------------------------------------------------------- */
/*               SET LIBRARY FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
let library_value = "null";
let library_name = "Null";

if (title_value.endsWith("_chapter") || title_value.endsWith("_course")) {
  let temp_file_path = project_file_path;
  if (parent_task_value != "null") {
    temp_file_path = parent_task_file_path;
  }
  abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
  file_cache = await app.metadataCache.getFileCache(abstract_file);
  if (file_cache?.frontmatter?.library) {
    library = file_cache?.frontmatter?.library[0];
    library_value = library.split("|")[0].slice(2);
    library_name = library.split("|")[1].slice(0, -2);
  }
}
// ADD ELSE IF FOR VIDEO AND CONTENT

let library_value_link = yaml_li(`[[${library_value}|${library_name}]]`);

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
let due_do_value;
if (title_filter("due_do")) {
  due_do_value = title_map("due_do");
} else {
  const due_do_date = await tp.user.include_template(tp, "40_task_do_due_date");
  due_do_value = due_do_date.split(";")[0];
}

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, "42_00_child_task_status");
const status_value = task_status.split(";")[0];
const status_name = task_status.split(";")[1];
const status_symbol = task_status.split(";")[2];

//-------------------------------------------------------------------
// ACTION ITEM PREVIEW, PLAN, AND REVIEW
//-------------------------------------------------------------------
let preview_review_file = "42_00_action_item_preview_review";
if (title_obj.value == "interview_prep") {
  preview_review_file = "42_01_act_pre_interview_preview_review";
} else if (title_obj.value == "typing_practice") {
  preview_review_file = "42_02_act_typing_preview_review";
} else if (title_obj.value == "daily_linkedin_job_search") {
  preview_review_file = "42_03_act_job_search_preview_review";
}

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: "Tasks and Events",
    toc_level: 1,
    toc_key: "Tasks and Events",
    file: null,
  },
  {
    head_key: "Prepare and Reflect",
    toc_level: 1,
    toc_key: "Insight",
    file: preview_review_file,
  },
  {
    head_key: "Related Tasks and Events",
    toc_level: 1,
    toc_key: "Related Tasks",
    file: "142_00_related_sect_task_child",
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
  if (!section_obj_arr[i].file) {
    continue;
  }
  section_obj_arr[i].content = await tp.user.include_template(
    tp,
    section_obj_arr[i].file
  );
}
section_obj_arr.map((x) => (x.head = head_lvl(2) + x.head_key));

//-------------------------------------------------------------------
// TASK CHECKBOX
//-------------------------------------------------------------------
const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}${space}`;

function task_checkbox(text, date_due) {
  const task_title = checkbox_task_tag + text + "_" + type_value;
  const datetime_start = moment(`${date_due}T${time}`);
  const start = moment(datetime_start).format("HH:mm");
  const remind = moment(datetime_start)
    .subtract(5, "minutes")
    .format("YYYY-MM-DD HH:mm");
  const datetime_end = moment(datetime_start).add(
    Number(duration_min),
    "minutes"
  );
  const end = moment(datetime_end).format("HH:mm");
  const duration_est = moment
    .duration(datetime_end.diff(datetime_start))
    .as("minutes");
  const inline_time = [
    `[time_start${dv_colon}${start}]`,
    `[time_end${dv_colon}${end}]`,
    `[duration_est${dv_colon}${duration_est}]`,
  ].join(two_space);
  const inline_date = [
    "⏰",
    remind,
    "➕",
    task_date_created,
    "📅",
    date_due,
  ].join(space);
  let task = [task_title, inline_time, inline_date].join(space);
  if (status_value == "done") {
    task = [task, "✅", date_due].join(space);
  }
  return [task, new_line, hr_line].join(new_line) + new_line;
}

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

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
let info;
if (title_value.endsWith("_chapter")) {
  info = await tp.user.include_file("42_21_act_ed_book_ch_info_callout");
} else if (title_value.endsWith("_course")) {
  info = await tp.user.include_file("42_22_act_ed_course_lect_info_callout");
} else {
  info = await tp.user.include_file("42_00_action_info_callout");
}

//-------------------------------------------------------------------
// BOTTOM FRONTMATTER YAML
//-------------------------------------------------------------------
const yaml_bottom = [
  `due_do:${space}${due_do_value}`,
  `pillar:${pillar_value_link}`,
  `context:${space}${context_value}`,
  `goal:${space}${goal}`,
  `project:${project_value_link}`,
  `parent_task:${parent_value_link}`,
  `organization:${organization_value_link}`,
  `contact:${contact_value_link}`,
  `library:${library_value_link}`,
  `type:${space}${type_value}`,
  `file_class:${space}${file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  "tags:",
  hr_line,
].join(new_line);

//-------------------------------------------------------------------
// LOOP THROUGH THE CHOSEN DAYS
//-------------------------------------------------------------------
for (let i = 0; i < action_day_arr.length; i++) {
  const date = action_day_arr[i];
  const date_link = `"[[${date}]]"`;
  const short_date = moment(date).format("YY-MM-DD");
  const short_date_value = moment(date).format("YY_MM_DD");

  // TASK TITLES, ALIAS, AND FILE NAME
  const short_title_name = title.toLowerCase();
  let short_title_value = short_title_name
    .replaceAll(/[\s-]/g, "_")
    .replaceAll(/[,']/g, "");

  const number_prefix_regex = /(\d{1,4}).+/g;
  if (
    title_value.endsWith("job_assignment") ||
    title_obj.value == "interview_prep"
  ) {
    title = `${title}${space}for${space}${parent_task_name}`;
    short_title_value = `${title_value}_${parent_task_value}`;
  } else if (title_value.startsWith("proj_setup_")) {
    const action_name = title.replace("Project Setup: ", "");
    const action_value = title_value.replace("proj_", "");
    title = `${action_name}${space}for${space}${project_name}`;
    short_title_value = `${action_value}_${project_value}`;
  } else if (
    title_value.endsWith("_chapter") ||
    title_value.endsWith("_course")
  ) {
    let number_prefix = library_value.replace(number_prefix_regex, "$1");
    if (number_prefix.length >= 3) {
      const part_number = number_prefix[0];
      const part_section_number = number_prefix.slice(1);
      number_prefix = `${part_number}.${part_section_number}`;
    }
    const action_name = title.split(" ")[0];
    const action_value = action_name.toLowerCase();
    let title_prefix = `${action_name} Chapter ${number_prefix}`;
    let title_prefix_value = `${action_value}_ch`;
    if (title_value == "learn_course") {
      title_prefix = `${action_name} Unit ${number_prefix}`;
      title_prefix_value = `${action_value}_`;
    } else if (title_value == "watch_course") {
      title_prefix = `${action_name} Lecture ${number_prefix}`;
      title_prefix_value = `${action_value}_`;
    }
    title = `${title_prefix} ${library_name}`;
    short_title_value = `${title_prefix_value}${library_value}`;
  }

  const full_title_name = `${short_date} ${title}`;
  const full_title_value = `${short_date_value}_${short_title_value}`;

  const file_alias =
    new_line +
    [
      title,
      full_title_name,
      short_title_name,
      short_title_value,
      full_title_value,
    ]
      .map((x) => `${ul_yaml}"${x}"`)
      .join(new_line);

  const file_name = full_title_value;
  const file_section = file_name + hash;

  // BOTTOM FRONTMATTER YAML
  const yaml_top = [
    hr_line,
    `title:${space}${file_name}`,
    `aliases:${file_alias}`,
    `date:${space}${date_link}`,
  ].join(new_line);

  section_obj_arr.map(
    (x) => (x.toc = `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
  );

  // TASK AND EVENTS SECTION
  section_obj_arr[0].content = task_checkbox(title, date);

  // TABLE OF CONTENTS CALLOUT
  const toc = [
    toc_title,
    call_start,
    toc_lvl(1),
    toc_body_div,
    toc_lvl(2),
  ].join(new_line);

  // FILE SECTIONS
  const sections_content = section_obj_arr
    .map((s) =>
      (s.file ? [s.head, toc, s.content] : [s.head, s.content]).join(
        two_new_line
      )
    )
    .join(new_line);

  // FILE CONTENT
  const file_content = [
    yaml_top,
    yaml_bottom,
    head_lvl(1) + title + new_line,
    info,
    sections_content,
  ].join(new_line);

  // FILE DIRECTORY AND PATH
  let directory;
  if (parent_task_value == "null") {
    directory = `${project_dir}/`;
  } else {
    directory = `${project_dir}/${parent_task_value}/`;
  }
  const file_path = `${directory}${file_name}.md`;

  // CREATE FILE
  await app.vault.create(file_path, file_content);
}
%>