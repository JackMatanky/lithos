<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const task_tag = "#task";
const type_name = "Action Item";
const type_value = "action_item";
const file_class = "task_child";

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
const re_snake_under = /(;\s)|(:\s)|(\-\s\-)|(\s)|(\-)/g;
const re_snake_remove = /(,|'|:|;)/g;
const re_snake_double_under = /_{2,}/g;
const re_snake_trim_under = /^_+|_+$/g;
const snake_case_fmt = (name) =>
  name
    .replace(re_snake_under, "_")
    .replace(re_snake_remove, "")
    .replace(re_snake_double_under, "_")
    .replace(re_snake_trim_under, "")
    .toLowerCase();
const md_ext = (file_name) => file_name + ".md";
const quote_enclose = (content) => `"${content}"`;

// CODE
const code_inline = (content) => backtick + content + backtick;
const code_block = (language = "", content = "") =>
  [three_backtick + language, content, three_backtick].join(new_line);

// COMMENTS
const cmnt_obsidian = (content) =>
  [two_percent, content, two_percent].join(space);
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

// LINKS
const regex_link = /.*\[([\w_].+)\|([\w\s].+)\].+/g;
const link_alias = (file, alias) => "[[" + [file, alias].join("|") + "]]";
const link_tbl_alias = (file, alias) => "[[" + [file, alias].join("\\|") + "]]";

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
  "[" + key + colon.repeat(2) + space + value + "]";
const dv_yaml = (property) => "file.frontmatter." + property;
const dv_content_link = code_inline(
  [
    "dv:",
    `link(this.file.name + "#" +`,
    `this.${dv_yaml("aliases[0]")},`,
    `"Contents")`,
  ].join(space),
);

// Utility: Split a semicolon-delimited string into trimmed components
function parse_semicolon_values(input, expected_count) {
  const parts = input.split(";").map((s) => s.trim());
  if (expected_count !== undefined && parts.length < expected_count) {
    throw new Error(
      `Expected ${expected_count} values but got ${parts.length}: "${input}"`,
    );
  }
  return parts;
}

// OBSIDIAN API
// async function metadata_alias(file_name) {
//   const path = await app.vault
//     .getMarkdownFiles()
//     .filter((file) => file.path.endsWith(`/${md_ext(file_name)}`))
//     .map((file) => file.path)[0];
//   const abstract_file = await app.vault.getAbstractFileByPath(path);
//   const file_cache = await app.metadataCache.getFileCache(abstract_file);
//   return file_cache?.frontmatter?.aliases[0];
// }

// Resolve full path to a markdown file by name
async function resolve_file_path(file_name) {
  const file_ext = md_ext(file_name);
  return app.vault
    .getMarkdownFiles()
    .find((file) => file.path.endsWith(`/${file_ext}`))?.path;
}

// Get the first alias from the file's frontmatter
async function metadata_alias(file_name) {
  const path = await resolve_file_path(file_name);
  if (!path) return null;

  const abstract_file = await app.vault.getAbstractFileByPath(path);
  const file_cache = await app.metadataCache.getFileCache(abstract_file);
  return file_cache?.frontmatter?.aliases?.[0] || null;
}

// DATE
const date_time = (date, time = "00:00") => moment(`${date}T${time}`);
const date_fmt = (format, date, time = "00:00") =>
  moment(`${date}T${time}`).format(format);
const date_add_sub_fmt = (format, unit, value, date, time = "00:00") =>
  value > 0
    ? moment(`${date}T${time}`).add(value, unit).format(format)
    : moment(`${date}T${time}`).subtract(Math.abs(value), unit).format(format);

// TASK INFO
const task_checkbox = (symbol) => `${ul}[${symbol}]`;
const task_time_end = (date, time, duration) =>
  date_add_sub_fmt("HH:mm", "minutes", Number(duration), date, time);
const task_duration_est = (date, time, duration) => {
  const end_time = moment(`${date}T${time}`).add(Number(duration), "minutes");
  return moment.duration(end_time.diff(date_time(date, time))).as("minutes");
};
const task_inline_time = (date, time, duration) =>
  [
    dv_inline("time_start", date_fmt("HH:mm", date, time)),
    dv_inline("time_end", task_time_end(date, time, duration)),
    dv_inline("duration_est", task_duration_est(date, time, duration)),
  ].join(two_space);
const task_inline_date = (date, time, symbol) => {
  const common = [
    "⏰",
    date_add_sub_fmt("YYYY-MM-DD HH:mm", "minutes", -5, date, time),
    "➕",
    moment().format("YYYY-MM-DD"),
    "📅",
    date_fmt("YYYY-MM-DD", date, time),
  ];
  if (symbol === "x") {
    common.push("✅", date_fmt("YYYY-MM-DD", date, time));
  }
  return common.join(space);
};
const task_text = (text, type, date, time, duration, symbol) =>
  [
    [
      task_checkbox(symbol),
      task_tag,
      `${text}_${type}`,
      task_inline_time(date, time, duration),
      task_inline_date(date, time, symbol),
    ].join(space),
    new_line + hr_line,
  ].join(new_line) + new_line;

/* ---------------------------------------------------------- */
/*                      GENERAL VARIABLES                     */
/* ---------------------------------------------------------- */

/* --------------------- NULL VARIABLES --------------------- */
const null_link = link_alias("null", "Null");
const null_yaml_li = yaml_li(null_link);
const null_arr = ["", "null", null_link, null];

/* -------------------- PILLAR VARIABLES -------------------- */
const pillar_mental_health = link_alias("mental_health", "Mental Health");
const pillar_physical_health = link_alias("physical_health", "Physical Health");
const pillar_knowledge = link_alias(
  "knowledge_expansion",
  "Knowledge Expansion",
);
const pillar_career = link_alias("career_development", "Career Development");
const pillar_data = link_alias("data_analyst", "Data Analyst");
const pillar_course = [pillar_knowledge, pillar_career, pillar_data]
  .map((x) => yaml_li(x))
  .join("");

/* ------------------- FILE PATH VARIABLES ------------------ */
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split("/");
const folder_path_length = folder_path_split.length;

/* -------- FILE CREATION AND MODIFIED DATE VARIABLES ------- */
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");
const task_date_created = moment().format("YYYY-MM-DD");

/* ---------------------------------------------------------- */
/*          TITLE AND PROJECT DIRECTORY OBJECT ARRAYS         */
/* ---------------------------------------------------------- */
const personal_obj_arr = [
  { key: "Personal Task", value: "personal_input" },
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
    key: "Organize and Update Tasks for Previous Days",
    value: "organize_update_previous_days_tasks",
    due_do: "do",
    pillar: null_yaml_li,
    project: "general_tasks_and_events",
    parent_task: "general_tasks",
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
    project: "keyboard_dev",
    parent_task: "ergogen_pcb_development",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Revise ZMK Config",
    value: "revise_zmk_config",
    due_do: "do",
    pillar: null_yaml_li,
    project: "keyboard_dev",
    parent_task: "zmk_keyboard_layout_development",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Typing Practice",
    value: "typing_practice",
    due_do: "do",
    pillar: null_yaml_li,
    project: "keyboard_dev",
    parent_task: "learn_keymap_layout",
    organization: null_link,
    contact: null_link,
  },
];

const chores_arr = [
  "Wash Laundry",
  "Hang Laundry",
  "Fold Laundry",
  "Buy Groceries",
  "Put Groceries into the Refrigerator",
  "Cook Dinner",
  "Wash Dishes",
  "Clean the Apartment",
];

const education_obj_arr = [
  { key: "Education Task", value: "education_input" },
  { key: "Learn Book Chapter", value: "learn_chapter" },
  { key: "Review Book Chapter", value: "review_chapter" },
  { key: "Learn Course Unit", value: "learn_course" },
  { key: "Watch Course Lecture", value: "watch_course" },
  { key: "Watch Educational Video", value: "watch" },
  { key: "Read Content", value: "read" },
  {
    key: "NAYA College Data Science Course Unit",
    value: "learn_naya_course",
    due_do: "do",
    pillar: pillar_course,
    project: "course_naya_college_practical_data_science",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Convert SuperMemo to Anki",
    value: "convert_supermemo_to_anki",
    due_do: "do",
    pillar: null_yaml_li,
    project: "spaced_repetition_learning",
    parent_task: "supermemo_to_anki_transition",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Revise Anki Cards",
    value: "revise_anki_cards",
    due_do: "do",
    pillar: null_yaml_li,
    project: "spaced_repetition_learning",
    parent_task: "anki_card_creation_and_revision",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Create General Anki Cards",
    value: "create_general_anki_cards",
    due_do: "do",
    pillar: null_yaml_li,
    project: "spaced_repetition_learning",
    parent_task: "anki_card_creation_and_revision",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Create Anki Cards for Course",
    value: "create_anki_cards_for_course",
    due_do: "do",
    pillar: yaml_li(pillar_knowledge),
    organization: null_link,
    contact: null_link,
  },
  {
    key: "Learn Anki Cards",
    value: "learn_anki_cards",
    due_do: "do",
    pillar: yaml_li(pillar_knowledge),
    project: "spaced_repetition_learning",
    parent_task: "spaced_repetition_learning_with_anki",
    organization: null_link,
    contact: null_link,
  },
  {
    key: "LeetCode Practice",
    value: "leetcode_practice",
    due_do: "do",
    pillar: yaml_li(pillar_knowledge),
    project: "general_education",
    parent_task: "leetcode_problem_solving",
    organization: "[[leetcode|LeetCode]]",
    contact: null_link,
  },
  {
    key: "General Hebrew University Task",
    value: "general_huji_input",
    due_do: "do",
    pillar: yaml_li(pillar_knowledge),
    project: "huji_general_tasks_and_events",
    organization:
      "[[hebrew_university_of_jerusalem|Hebrew University of Jerusalem]]",
  },
];

const professional_obj_arr = [
  { key: "Professional Task", value: "professional_input" },
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
    key: "Daily LinkedIn Job Search",
    value: "daily_linkedin_job_search",
    due_do: "do",
    pillar: yaml_li("[[career_development|Career Development]]"),
    project: "job_hunting_2023",
    parent_task: "daily_job_search_2023",
    organization: "[[linkedin|LinkedIn]]",
    contact: null_link,
  },
  {
    key: "Networking Meeting Preparation",
    value: "network_meeting_prep",
    due_do: "do",
    project: "networking",
  },
];

const work_obj_arr = [
  { key: "Work Task", value: "work_input" },
  {
    key: "Task for Hive Urban",
    value: "hive_urban_input",
    pillar: yaml_li(pillar_data),
    project: "hive_research_assistant",
    organization: "[[hive|Hive]]",
  },
];

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
  } else if (fa > fb) {
    return 1;
  } else {
    return 0;
  }
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
    `${type_name} Title?`,
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
    "House Chore?",
  );
} else if (title_value.endsWith("_template")) {
  template_title = await tp.system.prompt(
    `Template to ${title_value.split("_")[0]}?`,
    null,
    true,
    false,
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

/* ---------------------------------------------------------- */
/*           PROJECT DIRECTORY AND CONTEXT VARIABLES          */
/* ---------------------------------------------------------- */
const project_path_obj_arr = [
  { key: "personal", dir: "41_personal/" },
  { key: "education", dir: "42_education/" },
  { key: "professional", dir: "43_professional/" },
  { key: "work", dir: "44_work/" },
];
const project_path_arr = project_path_obj_arr.map((x) => x.dir);

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
}

let project_dir_path;
if (context_value) {
  project_dir_path = project_path_obj_arr
    .filter((path) => path.key == context_value)
    .map((path) => path.dir);
}
// // Unified context map: object arrays and project paths per context
// const context_map = {
//   "personal": {
//     obj_arr: personal_obj_arr,
//     project_path: "41_personal/",
//   },
//   "education": {
//     obj_arr: education_obj_arr,
//     project_path: "42_education/",
//   },
//   "professional": {
//     obj_arr: professional_obj_arr,
//     project_path: "43_professional/",
//   },
//   "work": {
//     obj_arr: work_obj_arr,
//     project_path: "44_work/",
//   },
// };
//
// // resolve context name from matching title_obj in obj_arr
// const resolve_context_from_title = (title_obj, context_map) =>
//   Object.entries(context_map).find(([_, ctx]) =>
//     ctx.obj_arr.some((el) => el.value === title_obj.value),
//   )?.[0] || null;
//
// // Resolve context and associated project directory path
// const context_value = resolve_context_from_title(title_obj, context_map);
//
// const project_dir_path = context_value
//   ? context_map[context_value].project_path
//   : null;
//
// // Optional debug/warning
// if (!context_value) {
//   console.warn(`Could not resolve context for: ${title_obj.value}`);
// }

/* ---------------------------------------------------------- */
/*                      SET DATE AND TIME                     */
/* ---------------------------------------------------------- */
const date = await tp.user.nl_date(tp, "start");
const date_link = `"[[${date}]]"`;
const short_date = moment(date).format("YY-MM-DD");
const short_date_value = moment(date).format("YY_MM_DD");

/* ---------------------------------------------------------- */
/*                 SET TASK TIME AND DURATION                 */
/* ---------------------------------------------------------- */
const time = await tp.user.nl_time(tp, `${type_name} Start Time?`);
const duration_min = await tp.user.durationMin(tp);

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
let pillar_value;
let pillar_yaml;

if (title_filter("pillar")) {
  // Use pillar from static title map
  pillar_yaml = title_map("pillar");
} else {
  const pillar_context = ["education", "professional"].includes(context_value)
    ? context_value
    : null;

  const { value: parsed_pillar_value, link: parsed_pillar_link } =
    await tp.user.multi_suggester({
      tp,
      items: await tp.user.file_by_status({
        dir: pillars_dir,
        status: "active",
      }),
      type: "pillar",
      context: pillar_context,
    });

  pillar_value = parsed_pillar_value;
  pillar_yaml = parsed_pillar_link;
}

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `${type_name} Goal?`,
);

/* ---------------------------------------------------------- */
/*      SET CONTEXT AND PROJECT BY FILE PATH OR SUGGESTER     */
/* ---------------------------------------------------------- */
let project_value;
let project_name;
if (title_filter("project")) {
  project_value = title_map("project");
  console.log(project_value);
  project_name = await metadata_alias(project_value);
  console.log(project_name);
} else if (
  project_path_arr.includes(`${folder_path_split[0]}/`) &&
  folder_path_length >= 2
) {
  const project_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[1],
    file_class: "task",
    type: "project",
  });
  project_value = project_obj[1].value;
  project_name = project_obj[1].key;
} else {
  let project_obj_arr;
  if (!project_dir_path) {
    project_obj_arr = [{ key: "Null", value: "null" }];
    for (let i = 0; i < project_path_arr.length; i++) {
      const obj_arr = await tp.user.file_name_alias_by_class_type({
        dir: project_path_arr[i],
        file_class: "task",
        type: "project",
      });
      project_obj_arr.push(
        ...obj_arr.filter((x) => !["null", "_user_input"].includes(x.value)),
      );
    }
  } else {
    project_obj_arr = await tp.user.file_name_alias_by_class_type_status({
      dir: project_dir_path,
      file_class: "task",
      type: "project",
      status: "active",
    });
  }
  project_obj = await tp.system.suggester(
    (item) => item.key,
    project_obj_arr,
    false,
    "Project?",
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
// await tp.user.resolve_file_path(project_value)
const project_dir = project_file_path.replace(project_name_ext, "");

if (!context_value) {
  context_value = project_dir.split("/")[0].replace(/^\d\d_/g, "");
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
  project_path_arr.includes(`${folder_path_split[0]}/`) &&
  folder_path_length >= 3
) {
  parent_task_obj = await tp.user.file_name_alias_by_class_type({
    dir: folder_path_split[2],
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
    "Parent Task?",
  );
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
}
const parent_yaml = yaml_li(link_alias(parent_task_value, parent_task_name));
const parent_task_dir = project_dir + parent_task_value;
const parent_task_file_path = `${parent_task_dir}/${parent_task_value}.md`;

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
let organization_value;
let organization_yaml;

if (title_filter("organization")) {
  // Case 1: Directly mapped via title metadata
  organization_yaml = yaml_li(title_map("organization"));
} else if (
  title_value.endsWith("job_assignment") ||
  title_obj.value === "interview_prep"
) {
  // Case 2: Pull from parent task frontmatter
  const abstract_file = await app.vault.getAbstractFileByPath(
    parent_task_file_path,
  );
  const file_cache = await app.metadataCache.getFileCache(abstract_file);
  const organization_link = file_cache?.frontmatter?.organization;
  organization_yaml = yaml_li(organization_link);
} else {
  // Case 3: Manual selection via multi_suggester
  const { value: parsed_org_value, link: parsed_org_link } =
    await tp.user.multi_suggester({
      tp,
      items: await tp.user.md_file_name_alias(organizations_dir),
      type: "organization",
    });

  organization_value = parsed_org_value;
  organization_yaml = parsed_org_link;
}

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
let contact_value;
let contact_yaml;

if (title_filter("contact")) {
  // Case 1: Use value from metadata title mapping
  const contact_link = title_map("contact");
  contact_value = contact_link.replaceAll(regex_link, "$1");
  contact_yaml = yaml_li(contact_link);
} else {
  // Case 2: Fallback to manual selection via multi_suggester
  const { value: parsed_contact_value, link: parsed_contact_link } =
    await tp.user.multi_suggester({
      tp,
      items: await tp.user.md_file_name_alias(contacts_dir),
      type: "contact",
    });

  contact_value = parsed_contact_value;
  contact_yaml = parsed_contact_link;
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

let library_yaml = yaml_li(link_alias(library_value, library_name));

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
const task_status = await tp.user.include_template(
  tp,
  "42_00_child_task_status",
);

const [status_value, status_name, status_symbol] =
  parse_semicolon_values(task_status);

/* ---------------------------------------------------------- */
/*          FRONTMATTER TITLE, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
/* -------------------- Title Helpers ------------------- */
// Functions for transforming and formatting titles.

function extract_number_prefix(library_value) {
  // Formats a 3–4 digit prefix as decimal (e.g., "1234" → "1.234")
  const match = library_value.match(/^(\d{1,4})/);
  if (!match) return "";
  const raw_number = match[1];
  return raw_number.length >= 3
    ? `${raw_number[0]}.${raw_number.slice(1)}`
    : raw_number;
}

function extract_project_setup(title, title_value) {
  // Removes "Project Setup" labels from both display and value strings
  const action_name = title.replace("Project Setup: ", "");
  const action_value = title_value.replace("proj_", "");
  return { action_name, action_value };
}

function build_title_with_context({
  base_title,
  base_value,
  context_name,
  context_value,
}) {
  // Assembles a contextual title and short ID from a task and its reference
  const title = [base_title, "for", context_name].join(space);
  const short_title_value = `${snake_case_fmt(base_value)}_${context_value}`;
  return { title, short_title_value };
}

/* ---------------- Library Title Builder --------------- */
// Handles chapters, courses, and lectures from library-type resources.

function get_title_prefix_info(title, title_value) {
  // Extracts the action prefix and chooses "ch" or default suffix
  const action_name = title.split(" ")[0];
  const title_prefix_value = title_value.includes("chap")
    ? `${action_name.toLowerCase()}_ch`
    : `${action_name.toLowerCase()}_`;
  return { action_name, title_prefix_value };
}

async function get_content_type_and_lesson(title_value, tp) {
  // Determines whether content is a Chapter, Lecture, or Unit and optionally prompts for lesson title
  let content_type = "Chapter";
  let unit_lesson_title = "";

  if (title_value === "watch_course") {
    content_type = "Lecture";
  } else if (title_value.endsWith("_course")) {
    content_type = "Unit";
    unit_lesson_title = await tp.system.prompt(`${content_type} Lesson Title?`);
  }

  return { content_type, unit_lesson_title };
}

async function handle_library_title({
  title,
  title_value,
  library_value,
  library_name,
  tp,
}) {
  // Builds full and short titles for library items including optional lesson name
  const number_prefix = extract_number_prefix(library_value);
  const { action_name, title_prefix_value } = get_title_prefix_info(
    title,
    title_value,
  );
  const { content_type, unit_lesson_title } = await get_content_type_and_lesson(
    title_value,
    tp,
  );

  let full_title = [
    action_name,
    content_type,
    number_prefix,
    library_name,
  ].join(space);
  let short_title_value = `${title_prefix_value}${library_value}`;

  if (unit_lesson_title) {
    const formatted = await tp.user.title_case(unit_lesson_title);
    full_title += `, ${formatted}`;
    short_title_value += `_${snake_case_fmt(formatted)}`;
  }

  return { title: full_title, short_title_value };
}

/* --------------- Title Logic Dispatcher --------------- */
// Decides which title strategy to apply based on value pattern.

let short_title_value = snake_case_fmt(title);

if (
  title_value.endsWith("job_assignment") ||
  title_obj.value === "interview_prep" ||
  title_value.startsWith("proj_setup_")
) {
  let base_title, base_value, context_name, context_value;

  if (title_value.startsWith("proj_setup_")) {
    const { action_name, action_value } = extract_project_setup(
      title,
      title_value,
    );
    base_title = action_name;
    base_value = action_value;
    context_name = project_name;
    context_value = project_value;
  } else {
    base_title = title;
    base_value = title_value;
    context_name = parent_task_name;
    context_value = parent_task_value;
  }

  const { title: new_title, short_title_value: new_short_title } =
    build_title_with_context({
      base_title,
      base_value,
      context_name,
      context_value,
    });

  title = new_title;
  short_title_value = new_short_title;
} else if (title_value === "create_anki_cards_for_course") {
  // Replaces "Course" with the actual project name in both title and short value
  title = title.replace("Course", project_name);
  short_title_value = title_value.replace("course", project_value);
} else if (
  title_value.endsWith("_chapter") ||
  title_value.endsWith("_course") ||
  title_value === "watch_course"
) {
  const result = await handle_library_title({
    title,
    title_value,
    library_value,
    library_name,
    tp,
  });

  title = result.title;
  short_title_value = result.short_title_value;
}

/* ------------- File Metadata Construction ------------- */
// Finalizes all filename and alias-related fields.

const full_title_name = `${short_date} ${title}`;
const short_title_name = title.toLowerCase();
const full_title_value = `${short_date_value}_${short_title_value}`;

const file_alias = [
  title,
  full_title_name,
  short_title_name,
  short_title_value,
  full_title_value,
]
  .map(yaml_li)
  .join("");

const file_name = full_title_value;
const file_section = file_name + hash;

/* ---------------------------------------------------------- */
/*            ACTION ITEM PREVIEW, PLAN, AND REVIEW           */
/* ---------------------------------------------------------- */
const preview_review_file_map = {
  default: "42_00_action_item_preview_review",
  interview_prep: "42_01_act_pre_interview_preview_review",
  typing_practice: "42_02_act_typing_preview_review",
  daily_linkedin_job_search: "42_03_act_job_search_preview_review",
  _chapter: "42_21_act_preview_review_ed_book_chapter",
};

// Match by exact value first
let preview_review_file = preview_review_file_map[title_obj.value];

// If no exact match, check suffix match
if (!preview_review_file && title_value.endsWith("_chapter")) {
  preview_review_file = preview_review_file_map["_chapter"];
}

// Fallback to default
preview_review_file ||= preview_review_file_map.default;

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
  section_obj_arr[i].content =
    i === 0
      ? task_text(title, type_value, date, time, duration_min, status_symbol)
      : await tp.user.include_template(tp, section_obj_arr[i].file);
  section_obj_arr[i].head = head_lvl(2, section_obj_arr[i].head_key);
  section_obj_arr[i].toc = link_tbl_alias(
    file_section + section_obj_arr[i].head_key,
    section_obj_arr[i].toc_key,
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
      .join(tbl_pipe),
  );

const toc = [
  call_title("toc", dv_content_link),
  call_start,
  toc_lvl(1),
  call_tbl_div(3),
  toc_lvl(2),
].join(new_line);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) =>
    (s.file ? [s.head, toc, s.content] : [s.head, s.content]).join(
      two_new_line,
    ),
  )
  .join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_file_map = {
  default: "42_00_action_info_callout",
  _chapter: "42_21_act_ed_book_ch_info_callout",
  _course: "42_22_act_ed_course_lect_info_callout",
};

// Look for the first suffix that matches title_value
const matched_suffix = Object.keys(info_file_map).find(
  (suffix) => suffix !== "default" && title_value.endsWith(suffix),
);

// Resolve the file name from the map
const info_file = info_file_map[matched_suffix] || info_file_map.default;

const info = await tp.user.include_file(info_file);

//const linkedin_link = "[LinkedIn Jobs](https://www.linkedin.com/jobs/)";
//const info_linkedin = `${call_start}${linkedin_link}${two_space}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory =
  parent_task_value === "null"
    ? `${project_dir}/`
    : `${project_dir}/${parent_task_value}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_link %>
due_do: <%* tR += due_do_value %>
pillar: <%* tR += pillar_yaml %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
project: <%* tR += project_yaml %>
parent_task: <%* tR += parent_yaml %>
organization: <%* tR += organization_yaml %>
contact: <%* tR += contact_yaml %>
library: <%* tR += library_yaml %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title %>

<%* tR += info %>
<%* tR += sections_content %>
