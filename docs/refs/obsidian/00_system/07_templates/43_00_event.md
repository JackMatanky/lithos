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
const type_name = "Meeting";
const type_value = type_name.toLowerCase();
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
const regex_snake_case = /(\-\s\-)|(\s)|(\-)/g;
const snake_case_fmt = (name) =>
  name.replaceAll(regex_snake_case, "_").toLowerCase();
const md_ext = (file_name) => file_name + ".md";

const code_inline = (content) => backtick + content + backtick;
const cmnt_obsidian = (content) =>
  [two_percent, content, two_percent].join(space);
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

// LINKS
const regex_link = /.*\[([\w_].+)\|([\w\s].+)\].+/g;
const link_alias = (file, alias) => ["[[" + file, alias + "]]"].join("|");
const link_tbl_alias = (file, alias) => ["[[" + file, alias + "]]"].join("\\|");

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
const date_time = (date, time = "00:00") => moment(`${date}T${time}`);
const date_fmt = (format, date, time = "00:00") =>
  moment(`${date}T${time}`).format(format);
const date_add_minus_fmt = (format, unit, value, date, time = "00:00") =>
  value > 0
    ? moment(`${date}T${time}`).add(value, unit).format(format)
    : moment(`${date}T${time}`).subtract(Math.abs(value), unit).format(format);

// DATAVIEW - INLINE
const dv_inline = (key, value) => "[" + key + colon.repeat(2) + value + "]";
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

// TASK
const task_checkbox = (symbol) => `${ul}[${symbol}]`;
const task_time_end = (date, time, duration) =>
  date_add_minus_fmt("HH:mm", "minutes", Number(duration), date, time);
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
    date_add_minus_fmt("YYYY-MM-DD HH:mm", "minutes", -5, date, time),
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
const pillar_social = "[[friends_social_life|Friends and Social Life]]";
const pillar_brother_uncle = "[[brother_uncle|Brother and Uncle]]";
const pillar_course = [pillar_knowledge, pillar_career, pillar_data]
  .map((x) => yaml_li(x))
  .join("");

let temp_file_path;

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
  { key: "Personal Meeting", value: "personal_input" },
  {
    key: "Coaching Session",
    value: "coaching_session",
    subtype: "Meeting",
    due_do: "due",
    pillar: yaml_li(pillar_mental_health),
    project: "coaching_with_nir_zer",
    parent_task: "coaching_sessions",
    organization: null_link,
    contact: "[[zer_nir|Nir Zer]]",
  },
  {
    key: "Hangout with a Friend or Group",
    value: "hangout",
    subtype: "Hangout",
    due_do: "do",
    pillar: yaml_li(pillar_social),
    project: "general_tasks_and_events",
    parent_task: "general_events",
  },
  {
    key: "Haircut",
    value: "haircut",
    subtype: "appointment",
    due_do: "due",
    pillar: null_yaml_li,
    project: "general_tasks_and_events",
    parent_task: "general_events",
    contact: "[[vachnash_yossi|Yossi Vachnash]]",
  },
  {
    key: "Phone Call with Gene Matanky",
    value: "phone_call_gene",
    subtype: "phone_call",
    due_do: "do",
    pillar: yaml_li(pillar_brother_uncle),
    project: "general_tasks_and_events",
    parent_task: "general_events",
    organization: null_link,
    contact: "[[matanky_gene|Gene Matanky]]",
  },
];
const medical_obj_arr = [
  {
    key: "Dental Checkup",
    value: "dental_checkup",
    subtype: "Appointment",
    due_do: "due",
    project: "medical",
    parent_task: "medical_appointments",
  },
  {
    key: "Dental Hygienist",
    value: "dental_hygienist",
    subtype: "Appointment",
    due_do: "due",
    project: "medical",
    parent_task: "medical_appointments",
  },
  {
    key: "Dental Procedure",
    value: "dental_procedure",
    subtype: "Appointment",
    due_do: "due",
    project: "medical",
    parent_task: "medical_appointments",
  },
  {
    key: "Dermatologist",
    value: "dermatologist",
    subtype: "Appointment",
    due_do: "due",
    project: "medical",
    parent_task: "medical_appointments",
  },
  {
    key: "Family Doctor",
    value: "family_doctor",
    subtype: "Appointment",
    due_do: "due",
    project: "medical",
    parent_task: "medical_appointments",
  },
  {
    key: "Neurologist",
    value: "neurologist",
    subtype: "Appointment",
    due_do: "due",
    project: "medical",
    parent_task: "medical_appointments",
  },
  {
    key: "Therapy",
    value: "therapy",
    subtype: "Appointment",
    due_do: "due",
    pillar: yaml_li(pillar_mental_health),
    project: "medical",
    parent_task: "betterhelp_psychotherapy",
    organization: "[[betterhelp|BetterHelp]]",
    contact: "[[roy_anita|Anita Roy]]",
  },
];
const education_obj_arr = [
  { key: "Education Meeting", value: "education_input" },
  { key: "Course Lecture", value: "lecture", subtype: "Lecture" },
  {
    key: "NAYA College Data Science Lecture",
    value: "practical_data_science_lecture",
    subtype: "Lecture",
    due_do: "due",
    pillar: pillar_course,
    project: "course_naya_college_practical_data_science",
    organization: "[[naya_college|NAYA College]]",
    contact: "[[geva_dror|Dror Geva]]",
  },
  {
    key: "Tutoring Session with Yohan Gross",
    value: "tutoring_session_yohan_gross",
    subtype: "video_call",
    due_do: "due",
    pillar: yaml_li(pillar_knowledge),
    project: "general_education",
    parent_task: "tutoring_with_yohan_gross",
    organization: null_link,
    contact: "[[gross_yohan|Yohan Gross]]",
  },
];
const professional_obj_arr = [
  { key: "Professional Meeting", value: "professional_input" },
  {
    key: "In-Person Interview",
    value: "interview_in_person",
    subtype: "Interview",
    due_do: "due",
    project: "job_hunting_2023",
  },
  {
    key: "Phone Call Interview",
    value: "interview_phone_call",
    subtype: "Interview",
    due_do: "due",
    project: "job_hunting_2023",
  },
  {
    key: "Video Call Interview",
    value: "interview_video_call",
    subtype: "Interview",
    due_do: "due",
    project: "job_hunting_2023",
  },
  {
    key: "Networking Meeting",
    value: "networking_meeting",
    due_do: "due",
    project: "networking",
    parent_task: "networking_meetings",
  },
];
const work_obj_arr = [
  { key: "Work Meeting", value: "work_input" },
  {
    key: "Hive Urban Meeting",
    value: "hive_urban_input",
    pillar: yaml_li(pillar_data),
    project: "hive_research_assistant",
    organization: "[[hive|Hive]]",
  },
  {
    key: "Hive Urban Weekly Team Meeting",
    value: "hive_urban_weekly_team_meeting",
    pillar: yaml_li(pillar_data),
    due_do: "due",
    project: "hive_research_assistant",
    parent_task: "hive_weekly_meetings",
    organization: "[[hive|Hive]]",
  },
  {
    key: "Hive Urban Weekly Geo Team Meeting",
    value: "hive_urban_weekly_geo_meeting",
    pillar: yaml_li(pillar_data),
    due_do: "due",
    project: "hive_research_assistant",
    parent_task: "hive_weekly_meetings",
    organization: "[[hive|Hive]]",
  },
];

const task_obj_arr = [
  ...personal_obj_arr,
  ...medical_obj_arr,
  ...education_obj_arr,
  ...professional_obj_arr,
  ...work_obj_arr,
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
    `${type_name} Title?`,
  );
  title = title_obj.key;
  title_value = title_obj.value;
} else {
  title = tp.file.title;
  title_value = title.trim();
  title = await tp.user.title_case(title_value);
}

if (title_value.endsWith("_input")) {
  title = await tp.system.prompt(`${type_name} Title?`, null, true, false);
  title = title.trim();
  title = await tp.user.title_case(title);
}

const title_filter = (filter_value) => {
  let filter = title_obj_arr
    .filter((x) => x.value == title_value)
    .filter((x) => x?.[filter_value]);
  return filter.length == 0 ? null : filter;
};
const title_map = (map_value) =>
  title_obj_arr
    .filter((x) => x.value == title_value)
    .map((x) => x?.[map_value])
    .toString();

//-------------------------------------------------------------------
// SET MEETING SUBTYPE
//-------------------------------------------------------------------
const meeting_obj_arr = [
  { key: "Meeting", value: "meeting" },
  { key: "Phone Call", value: "phone_call" },
  { key: "Video Call", value: "video_call" },
  { key: "Interview", value: "interview" },
  { key: "Appointment", value: "appointment" },
  { key: "Lecture", value: "lecture" },
  { key: "Tutorial", value: "tutorial" },
  { key: "Event", value: "event" },
  { key: "Gathering", value: "gathering" },
  { key: "Hangout", value: "hangout" },
];

let subtype_name;
let subtype_value;
if (title_filter("subtype")) {
  subtype_name = title_map("subtype");
  subtype_value = subtype_name.replaceAll(/\s/g, "_").toLowerCase();
} else {
  const meeting_obj = await tp.system.suggester(
    (item) => item.key,
    meeting_obj_arr,
    false,
    "Meeting Type?",
  );
  subtype_name = meeting_obj.key;
  subtype_value = meeting_obj.value;
}

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
if (context_include(personal_obj_arr) || context_include(medical_obj_arr)) {
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
//   personal: {
//     obj_arr: [...personal_obj_arr, ...medical_obj_arr],
//     project_path: "41_personal/",
//   },
//   education: {
//     obj_arr: education_obj_arr,
//     project_path: "42_education/",
//   },
//   professional: {
//     obj_arr: professional_obj_arr,
//     project_path: "43_professional/",
//   },
//   work: {
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
  `${subtype_name} Goal?`,
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
const project_name_ext = `${project_value}.md`;
const project_file_path = await app.vault
  .getMarkdownFiles()
  .filter((file) => file.path.endsWith(`/${project_name_ext}`))
  .map((file) => file.path)[0];
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
const parent_task_dir = `${project_dir}${parent_task_value}`;
const parent_task_file_path = `${parent_task_dir}/${parent_task_value}.md`;

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
let organization_value;
let organization_yaml;

if (title_filter("organization")) {
  // Case 1: Directly mapped via title metadata
  organization_yaml = yaml_li(title_map("organization"));
} else if (title_value.startsWith("interview")) {
  // Case 2: Pull from parent task frontmatter
  const abstract_file = await app.vault.getAbstractFileByPath(
    parent_task_file_path,
  );
  const file_cache = await app.metadataCache.getFileCache(abstract_file);
  const organization_link = file_cache?.frontmatter?.organization;
  organization_yaml = yaml_li(organization_link);

  if (!organization_link) {
    console.warn(
      `Could not locate organization in parent task frontmatter: ${parent_task_value}`,
    );
  }
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
  // Case 1: Use values from metadata title mapping (as a list of links)
  //const contact_links = title_map("contact").match(/\[\[.*?\]\]/g) || [];
  //contact_values = contact_links.map(link => link.replaceAll(regex_link, "$1"));
  //contact_yaml = contact_links.map(link => yaml_li(link)).join("\n");

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

if (title_value.includes("lecture")) {
  temp_file_path = project_file_path;
  if (parent_task_value != "null") {
    temp_file_path = parent_task_file_path;
  }
  console.log(temp_file_path);
  abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
  console.log(abstract_file);
  file_cache = await app.metadataCache.getFileCache(abstract_file);
  if (file_cache?.frontmatter?.library) {
    library = file_cache?.frontmatter?.library[0];
    library_value = library.split("|")[0].slice(2);
    library_name = library.split("|")[1].slice(0, -2);
  }
}

let library_yaml = yaml_li_link(library_value, library_name);

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
/*         FRONTMATTER TITLES, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
let short_title_value = title
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/[,']/g, "")
  .toLowerCase();

const number_prefix_regex = /(\d{1,4}).+/g;
if (title_obj.value == "therapy") {
  title = "Therapy Appointment";
  short_title_value = "therapy_appointment";
} else if (medical_obj_arr.map((el) => el.value).includes(title_obj.value)) {
  short_title_value = title_value;
} else if (title_value.startsWith("interview")) {
  title = "Interview for " + parent_task_name;
  short_title_value = `${title_obj.value}_${parent_task_value}`;
} else if (title_value.includes("lecture")) {
  const course_name =
    parent_task_value != "null" ? parent_task_name : project_name;
  const course_value =
    parent_task_value != "null" ? parent_task_value : project_value;
  title = ["Lecture", "for", course_name].join(space);
  short_title_value = `${title_obj.value}_${course_value}`;
  if (library_value != "null") {
    let number_prefix = library_value.replace(number_prefix_regex, "$1");
    if (number_prefix.length >= 3) {
      const part_number = number_prefix[0];
      const part_section_number = number_prefix.slice(1);
      number_prefix = `${part_number}.${part_section_number}`;
    }
    title = ["Lecture", number_prefix, library_name].join(space);
    short_title_value = `lecture_${library_value}`;
  }
} else if (title_obj.value == "networking_meeting") {
  title = `Networking Meeting with ${contact_name}`;
  short_title_value = `networking_meeting_${contact_value}`;
} else if (
  ["hangout", "networking_meeting", "phone_call_gene"].includes(title_obj.value)
) {
  let contact_value_arr = contact_value.split(",");
  let contact_name_arr = contact_name.split(",");
  const title_prefix_obj_arr = [
    { key: "hangout", prefix_name: "Hangout", prefix_value: "hangout" },
    {
      key: "networking_meeting",
      prefix_name: "Networking Meeting",
      prefix_value: "networking_meeting",
    },
    {
      key: "phone_call_gene",
      prefix_name: "Phone Call",
      prefix_value: "phone_call",
    },
  ];
  const prefix_name = title_prefix_obj_arr
    .filter((x) => title_obj.value == x.key)
    .map((x) => x.prefix_name);
  const prefix_value = title_prefix_obj_arr
    .filter((x) => title_obj.value == x.key)
    .map((x) => x.prefix_value);

  if (contact_value_arr.length < 2) {
    contact_value_arr = contact_value;
    contact_name_arr = contact_name;
  } else if (contact_value_arr.length == 2) {
    contact_value_arr.join("_and_");
    contact_name_arr.join(" and ");
  }

  title = [prefix_name, "with", contact_name_arr].join(space);
  short_title_value = `${prefix_value}_${contact_value_arr}`;
}
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
  .map((x) => yaml_li(x))
  .join("");

const file_name = full_title_value;
const file_section = file_name + hash;

//-------------------------------------------------------------------
// MEETING PREVIEW, PLAN, AND REVIEW
//-------------------------------------------------------------------
let meeting_preview_review = "43_00_meeting_preview_review";
if (title_value == "coaching_session") {
  meeting_preview_review = "43_01_coaching_session_preview_review";
} else if (title_value.startsWith("interview")) {
  meeting_preview_review = "43_02_interview_preview_review";
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
    file: meeting_preview_review,
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
      ? task_text(title, subtype_value, date, time, duration_min, status_symbol)
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
const info_title = call_title(type_value, `${type_name}${space}Details`);
const info_body = await tp.user.include_file("42_00_child_task_info_callout");
const info = [info_title, call_start, info_body].join(new_line);

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
type: <%* tR += subtype_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title %>

<%* tR += info %>
<%* tR += sections_content %>