<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const habit_ritual_proj_dir = "45_habit_ritual/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";

//-------------------------------------------------------------------
// CONTEXT NAME, VALUE, DIRECTORY, AND FILE CLASS
//-------------------------------------------------------------------
const context_name = "Habits and Rituals";
const context_value = "habit_ritual";
const context_dir = habit_ritual_proj_dir;
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
const remove_trail_s = (name) => name.replace(/s$/, "");
const concat_space_fmt = (value_1, value_2) => [value_1, value_2].join(space);
const concat_underscore_fmt = (value_1, value_2) =>
  [value_1, value_2].join("_");

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
  [`${call_start}[!${call_type}]`, title].join(space);

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
const code_inline = (content) => backtick + content + backtick;

// DATE
const date_fmt = (format, date, time = "00:00") =>
  moment(`${date}T${time}`).format(format);
const date_add_sub_fmt = (format, unit, value, date, time = "00:00") =>
  value > 0
    ? moment(`${date}T${time}`).add(value, unit).format(format)
    : moment(`${date}T${time}`).subtract(-value, unit).format(format);

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");
const task_date_created = moment().format("YYYY-MM-DD");

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
const null_link = link_alias("null", "Null");
const null_yaml_li = yaml_li(null_link);
const null_arr = ["", "null", null_yaml_li, null];

const dv_yaml = (property) => "file.frontmatter." + property;
const dv_content_link = code_inline(
  [
    "dv:",
    `link(this.file.name + "#" +`,
    `this.${dv_yaml("aliases[0]")},`,
    `"Contents")`,
  ].join(space)
);

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
const join_non_empty = (arr, joiner) => arr.filter(Boolean).join(joiner);

// Journal
const generate_journal_alias = (key) => "Daily " + key;
const generate_journal_full_value = (value) => "daily_" + value;
const generate_journal_button = (value) =>
  code_inline(["button", value, "daily", "preset"].join("-"));
const generate_journal_file_section_name = (name, full_value) =>
  name ? full_value + name : full_value;

// Habit and Ritual Object Array
const generate_hab_rit_name = (key) =>
  key == "Habits" ? key : key + " Rituals";
const generate_hab_rit_type = (value) => value.replace(/s$/, "");

const generate_hab_rit_file_name = (date, type, value, full_value) =>
  concat_underscore_fmt(date, type == "habit" ? full_value : value);
const generate_hab_rit_file_alias_arr = (
  date,
  name,
  full_name,
  value,
  full_value
) => [
  full_name,
  full_name.toLowerCase(),
  full_value,
  concat_space_fmt(date_fmt("YYYY-MM-DD", date), full_name),
  concat_underscore_fmt(date_fmt("YY_MM_DD", date), full_value),
  name,
  name.toLowerCase(),
  value,
  concat_space_fmt(date_fmt("YYYY-MM-DD", date), name),
  concat_underscore_fmt(date_fmt("YY_MM_DD", date), value),
];
const generate_hab_rit_parent_value = (date, type, ord, key, value) =>
  [date, `0${ord}`, type == "habit" ? key.toLowerCase() : value].join("_");
const generate_hab_rit_parent_name = (date, type, key, name) =>
  concat_space_fmt(date, type == "habit" ? key : name);
const generate_hab_rit_path = (project, parent, file_name) =>
  [context_dir + project, parent, file_name + ".md"].join("/");

// Habit and Ritual Tasks Object Array
const generate_obj_heading = (level, heading) =>
  !heading ? heading : head_lvl(level, heading) + two_new_line;

const set_start_time = (default_time, prompt_time) =>
  null_arr.includes(prompt_time) ? default_time : prompt_time;
const set_content = (obj_arr, key, value, content) => {
  const obj = obj_arr.find((obj) => obj[key] === value);
  if (obj) {
    obj.content = new_line + content;
  }
};

const set_task_info = (obj_arr, date, get_start_time) => {
  obj_arr.forEach((obj, index) => {
    obj.start = get_start_time(obj_arr, index);
    obj.end = date_add_sub_fmt(
      "HH:mm",
      "minutes",
      obj.duration,
      date,
      obj.start
    );
    obj.reminder = obj.remind
      ? date_add_sub_fmt(
          "YYYY-MM-DD HH:mm",
          "minutes",
          -obj.remind,
          date,
          obj.start
        )
      : obj.remind;
    obj.task_checkbox = !obj.task
      ? obj.task
      : task_text(
          obj.task,
          obj.type,
          date,
          obj.start,
          obj.end,
          obj.duration,
          obj.reminder
        ) + new_line;
  });
};

//-------------------------------------------------------------------
// DAILY JOURNALS HEADING AND BUTTON
//-------------------------------------------------------------------
const head_morn_journal = head_lvl(3, "Morning Journals");
const daily_journal_button =
  [
    `${three_backtick}button`,
    "name 🕯️Daily Journals",
    "type note(Untitled, split) template",
    "action 90_01_daily_journals_preset",
    "color purple",
    three_backtick,
  ].join(new_line) + two_new_line;

//-------------------------------------------------------------------
// REFLECTION, GRATITUDE, AND DETACHMENT JOURNAL VARIABLES
//-------------------------------------------------------------------
const journal_obj_arr = [
  {
    key: "Reflection",
    value: "reflection",
    name: "#What Happened _date_prev_ Yesterday?",
  },
  {
    key: "Wins and Losses",
    value: "reflection",
    name: "#What Unplanned Occurrences Happened?",
  },
  { key: "Gratitude", value: "gratitude", name: "#I Am Grateful For…" },
  { key: "Detachment", value: "detachment", name: null },
];

journal_obj_arr.forEach((obj) => {
  obj.alias = generate_journal_alias(obj.key);
  obj.full_value = generate_journal_full_value(obj.value);
  obj.button = generate_journal_button(obj.value);
  obj.file_section_name = generate_journal_file_section_name(
    obj.name,
    obj.full_value
  );
  obj.call_title = call_title(obj.value, obj.alias + " Journal");
});

//-------------------------------------------------------------------
// GUIDED MEDITATION
//-------------------------------------------------------------------
const meditation_content = [
  head_lvl(4, "Mindfulness Bell Meditation"),
  "![[1_Five Minute Mindfulness Bell Meditation.mp3]]",
  head_lvl(4, "Positive Mind Meditation"),
  "![[morn_1_Positive Mind in 5 Minutes Meditation_Jason Stephenson.mp3]]",
].join(two_new_line);

//-------------------------------------------------------------------
// MENTAL AND PHYSICAL HEALTH PILLAR NAME AND LINK
//-------------------------------------------------------------------
const pillar_mental_yaml = yaml_li(
  link_alias("mental_health", "Mental Health")
);
const pillar_physical_yaml = yaml_li(
  link_alias("physical_health", "Physical Health")
);

//-------------------------------------------------------------------
// PILLARS, GOALS, ORGANIZATIONS, AND CONTACTS
//-------------------------------------------------------------------
const pillars_obj_arr = await tp.user.file_by_status({
  dir: pillars_dir,
  status: "active",
});
//const goals = await tp.user.md_file_name(goals_dir);
const org_obj_arr = await tp.user.md_file_name_alias(organizations_dir);
const contact_obj_arr = await tp.user.md_file_name_alias(contacts_dir);

// Boolean Arrays of Objects
const bool_obj_arr = [
  { key: "✔️ YES ✔️", value: "yes" },
  { key: "❌ NO ❌", value: "no" },
];

async function multi_suggester(day_name, hab_rit_name, type, obj_arr) {
  let file_obj_arr = [];
  let file_filter = [];
  for (let i = 0; i < 10; i++) {
    // File Suggester
    const file_suggest_obj = await tp.system.suggester(
      (item) => item.key,
      obj_arr.filter((file) => !file_filter.includes(file.value)),
      false,
      `${type} for ${day_name} ${hab_rit_name}?`
    );
    const file_obj = {
      key: file_suggest_obj.key,
      value: file_suggest_obj.value,
    };
    if (file_obj.value == "_user_input") {
      if (obj_arr == org_obj_arr) {
        file_obj.key = await tp.system.prompt(
          `Organization for ${weekday} ${hab_rit_name}?`,
          "",
          false,
          false
        );
        file_obj.value = file_obj.key
          .replaceAll(/[,']/g, "")
          .replaceAll(/\s/g, "_")
          .replaceAll(/\//g, "-")
          .replaceAll(/&/g, "and")
          .toLowerCase();
      } else if (obj_arr == contact_obj_arr) {
        contact_names = await tp.user.dirContactNames(tp);
        full_name = contact_names.fullName;
        last_first_name = contact_names.lastFirstName;
        file_obj.key = full_name;
        file_obj.value = last_first_name
          .replaceAll(/,/g, "")
          .replaceAll(/[^\w]/g, "*")
          .toLowerCase();
      }
    }
    if (null_arr.includes(file_obj.value)) {
      if (file_obj_arr) {
        break;
      }
      file_obj_arr.push(file_obj);
      break;
    }
    file_obj_arr.push(file_obj);
    file_filter.push(file_obj.value);

    const bool_obj = await tp.system.suggester(
      (item) => item.key,
      bool_obj_arr,
      false,
      `Another ${type}?`
    );

    if (bool_obj.value == "no") {
      break;
    }
  }
  return file_obj_arr
    .map((file) => yaml_li(link_alias(file.value, file.key)))
    .join("");
}

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
  full_date = moment().startOf("week");
} else if (date_value.startsWith("next")) {
  full_date = moment().add(1, "week");
} else {
  full_date = moment().subtract(1, "week");
}

//-------------------------------------------------------------------
// WEEKDAY AND DEFAULT TIME VARIABLES
//-------------------------------------------------------------------
const moment_day = (int) => moment(full_date).day(int).format("YYYY-MM-DD");
const weekday_arr = [0, 1, 2, 3, 4].map((x) => moment_day(x));

const default_time_morning = "08:00";
const default_time_habit_early = "13:00";
const default_time_habit_late = "16:00";
const default_time_evening = "18:50";

//-------------------------------------------------------------------
// HABIT AND RITUAL OBJECT ARRAY
//-------------------------------------------------------------------
const hab_rit_obj_arr = [
  { ord: 1, rate: "Bi-Daily", key: "Habits", button: "habit" },
  { ord: 2, rate: "Daily", key: "Morning", button: "morn-rit" },
  { ord: 3, rate: "Daily", key: "Workday Startup", button: "work-start" },
  { ord: 4, rate: "Daily", key: "Workday Shutdown", button: "work-shut" },
  { ord: 5, rate: "Daily", key: "Evening", button: "eve-rit" },
];

hab_rit_obj_arr.forEach((obj) => {
  obj.name = generate_hab_rit_name(obj.key);
  obj.name_low = obj.name.toLowerCase();
  obj.value = snake_case_fmt(obj.name);
  obj.type = generate_hab_rit_type(obj.value);
  obj.full_name = concat_space_fmt(obj.rate, obj.name);
  obj.full_name_low = obj.full_name.toLowerCase();
  obj.full_value = snake_case_fmt(obj.full_name);
  obj.main_head = head_lvl(1, obj.name);
  obj.sub_head = head_lvl(2, obj.key == "Habits" ? obj.key : "Rituals");
});

//-------------------------------------------------------------------
// TODAY AND TOMORROW HABIT AND RITUAL BUTTONS AND LINK TABLE
//-------------------------------------------------------------------
const hab_rit_button_table_title = (day) =>
  call_title(context_value, `${day}'s ${context_name}`);
const hab_rit_button_table_row = (day) =>
  hab_rit_obj_arr
    .map((x) => code_inline(["button", x.button, day.toLowerCase()].join("-")))
    .join(tbl_pipe);

const hab_rit_button_table = (day) =>
  [
    hab_rit_button_table_title(day),
    call_start,
    call_tbl_row(hab_rit_button_table_row(day)),
    call_tbl_div(5),
  ].join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_body = await tp.user.include_file("42_00_child_task_info_callout");
hab_rit_obj_arr.map(
  (x) =>
    (x.info = [
      x.main_head,
      call_title(x.type, `${x.name} Details`),
      call_start,
      info_body,
      x.sub_head,
    ].join(new_line))
);

//-------------------------------------------------------------------
// HABITS AND RITUALS TASK OBJECT ARRAYS
//-------------------------------------------------------------------
const morn_start_obj_arr = [
  {
    type: "morning_ritual",
    head_level: 3,
    head: "Morning Self Affirmations",
    task: "Self Affirmations",
    duration: 3,
    remind: 5,
  },
  {
    type: "morning_ritual",
    head_level: 3,
    head: "Plan the Day",
    task: "Daily Schedule Preview",
    duration: 10,
    remind: null,
  },
  {
    type: "morning_ritual",
    head_level: 4,
    head: "Recount Yesterday",
    task: "Daily Reflection",
    duration: 15,
    remind: null,
  },
  {
    type: "morning_ritual",
    head_level: 4,
    head: "Yesterday's Wins and Losses",
    task: "Daily Achievements and Blind Spots",
    duration: 5,
    remind: null,
  },
  {
    type: "morning_ritual",
    head_level: 4,
    head: "Give Thanks",
    task: "Daily Gratitude",
    duration: 3,
    remind: null,
  },
  {
    type: "workday_startup_ritual",
    head_level: 3,
    head: "Email Review",
    task: "Morning Email Review",
    duration: 6,
    remind: 5,
  },
  {
    type: "workday_startup_ritual",
    head_level: 3,
    head: "WhatsApp Review",
    task: "Morning WhatsApp Review",
    duration: 6,
    remind: null,
  },
  {
    type: "morning_ritual",
    head_level: 3,
    head: "Start the Day With a Clear Mind",
    task: "Morning Meditation",
    duration: 10,
    remind: 5,
  },
];

const shut_eve_obj_arr = [
  {
    type: "workday_shutdown_ritual",
    head: "Knowledge Review",
    task: "Evening PKM Review",
    duration: 10,
    remind: 5,
  },
  {
    type: "workday_shutdown_ritual",
    head: "Email Review",
    task: "Evening Email Review",
    duration: 6,
    remind: null,
  },
  {
    type: "workday_shutdown_ritual",
    head: "WhatsApp Review",
    task: "Evening WhatsApp Review",
    duration: 6,
    remind: null,
  },
  {
    type: "evening_ritual",
    head: "Review the Day",
    task: "Daily Schedule Review",
    duration: 5,
    remind: 5,
  },
  {
    type: "evening_ritual",
    head: "Get a Jump on Tomorrow",
    task: "Preview Tomorrow's Schedule",
    duration: 10,
    remind: 5,
  },
];

const hab_head_obj_arr = [
  {
    type: "habit",
    key: "srs",
    head: "Spaced Repetition",
  },
  {
    type: "habit",
    key: "detach",
    head: "Detachment Practice",
  },
  {
    type: "habit",
    key: "gratitude",
    head: "Thank Yourself",
  },
  {
    type: "habit",
    key: "fit",
    head: "Movement",
  },
  {
    type: "habit",
    key: "meditation",
    head: "Let Go and Release",
  },
];
const hab_srs_obj_arr = [
  {
    type: "habit",
    key: "srs",
    head: null,
    task: "First SRS Review",
    duration: 20,
    remind: 5,
  },
  {
    type: "habit",
    key: "srs",
    head: null,
    task: "Second SRS Review",
    duration: 10,
    remind: 5,
  },
];
const hab_early_obj_arr = [
  {
    type: "habit",
    key: "detach",
    head: null,
    task: "Early Afternoon Detachment",
    duration: 5,
    remind: 5,
  },
  {
    type: "habit",
    key: "gratitude",
    head: null,
    task: "Early Afternoon Self Gratitude",
    duration: 3,
    remind: null,
  },
  {
    type: "habit",
    key: "fit",
    head: "Core Strength",
    task: "Early Afternoon Movement",
    duration: 10,
    remind: null,
  },
];
const hab_late_obj_arr = [
  {
    type: "habit",
    key: "detach",
    head: null,
    task: "Late Afternoon Detachment",
    duration: 5,
    remind: 5,
  },
  {
    type: "habit",
    key: "gratitude",
    head: null,
    task: "Late Afternoon Self Gratitude",
    duration: 3,
    remind: null,
  },
  {
    type: "habit",
    key: "fit",
    head: "Dynamic Flexibility",
    task: "Late Afternoon Movement",
    duration: 10,
    remind: null,
  },
  {
    type: "habit",
    key: "meditation",
    head: null,
    task: "Late Afternoon Meditation",
    duration: 10,
    remind: null,
  },
];

const hab_obj_arr_filter = (filter) =>
  [...hab_head_obj_arr, ...hab_early_obj_arr, ...hab_late_obj_arr].filter(
    (x) => x.key == filter
  );

morn_start_obj_arr.map(
  (x) => (x.heading = generate_obj_heading(x.head_level, x.head))
);
shut_eve_obj_arr.map((x) => (x.heading = generate_obj_heading(3, x.head)));
hab_head_obj_arr.map((x) => (x.heading = generate_obj_heading(3, x.head)));
hab_early_obj_arr.map((x) => (x.heading = generate_obj_heading(4, x.head)));
hab_late_obj_arr.map((x) => (x.heading = generate_obj_heading(4, x.head)));

hab_early_obj_arr[2].content =
  new_line +
  (await tp.user.include_file("45_01_movement_early_afternoon_callout"));
hab_late_obj_arr[2].content =
  new_line +
  (await tp.user.include_file("45_02_movement_late_afternoon_callout"));

//-------------------------------------------------------------------
// TASK STATUS AND SYMBOL
//-------------------------------------------------------------------
const task_tag = "#task";
const status_symbol = " ";
const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}`;

const task_inline_time = (start, end, duration) =>
  [
    `[time_start${dv_colon}${start}]`,
    `[time_end${dv_colon}${end}]`,
    `[duration_est${dv_colon}${duration}]`,
  ].join(two_space);
const task_inline_remind = (remind) =>
  remind ? ["⏰", remind].join(space) : null;
const task_inline_date = (date_due) =>
  ["➕", task_date_created, "📅", date_due].join(space);

const task_text = (text, type, date_due, start, end, duration, remind) =>
  [
    checkbox_task_tag,
    concat_underscore_fmt(text, type),
    task_inline_time(start, end, duration),
    remind ? task_inline_remind(remind) : null,
    task_inline_date(date_due),
  ]
    .filter(Boolean)
    .join(space);

//-------------------------------------------------------------------
// YAML FRONTMATTER
//-------------------------------------------------------------------
const yaml_bottom = [
  concat_space_fmt("file_class:", file_class),
  concat_space_fmt("date_created:", date_created),
  concat_space_fmt("date_modified:", date_modified),
  "tags:",
  hr_line,
].join(new_line);

const generate_yaml_properties = (obj, date_link, project) =>
  [
    hr_line,
    concat_space_fmt("title:", obj.file_name),
    concat_space_fmt("uuid:", await tp.user.uuid()),
    concat_space_fmt("aliases:", obj.file_alias),
    concat_space_fmt("date:", date_link),
    concat_space_fmt("due_do:", "do"),
    concat_space_fmt("pillar:", obj.pillar),
    concat_space_fmt("context:", context_value),
    concat_space_fmt("goal:", "null"),
    concat_space_fmt("project:", project),
    concat_space_fmt("parent_task:", obj.parent_yaml),
    concat_space_fmt("organization:", obj.organization),
    concat_space_fmt("contact:", obj.contact),
    concat_space_fmt("library:", null_yaml_li),
    concat_space_fmt("type:", obj.type),
    yaml_bottom,
  ].join(new_line);

//-------------------------------------------------------------------
// CREATE HABITS AND RITUALS FILES
//-------------------------------------------------------------------
let last_file_content = "";
let last_file_path = "";
for (let i = 0; i < weekday_arr.length; i++) {
  // SET DATE, TITLE, ALIASES, PROJECT AND PARENT TASK
  const date = weekday_arr[i];
  console.log(date);
  const date_link = `"[[${date}]]"`;
  const date_value = moment(date).format("YY_MM_DD");
  const date_next = moment(date).add(1, "days").format("YYYY-MM-DD");
  const date_next_value = moment(date).add(1, "days").format("YY_MM_DD");
  const date_prev = moment(date).subtract(1, "days").format("YYYY-MM-DD");
  const weekday = moment(date).format("dddd");
  const year_month_short = moment(date).format("YYYY-MM");
  console.log(year_month_short);
  const year_month_long = moment(date).format("MMMM [']YY");
  console.log(year_month_long);

  // PROJECT FILE NAME, ALIAS AND LINK
  const project_value = concat_underscore_fmt(year_month_short, context_value);
  const project_name = concat_space_fmt(year_month_long, context_name);
  const project_yaml = yaml_li(link_alias(project_value, project_name));

  // FILE CALLOUT TITLES
  const file_today_task_due_link = link_alias(
    date + "_task_event#Due Today",
    "Today's Schedule"
  );
  const file_today_task_due = call_title("task_plan", file_today_task_due_link);

  const file_today_task_done_link = link_alias(
    date + "_task_event#Completed Today",
    "Today's Completed Tasks"
  );
  const file_today_task_done = call_title(
    "task_review",
    file_today_task_done_link
  );

  const file_tomorrow_task_link = link_alias(
    date_next + "_task_event#Due Today",
    "Tomorrow's Tasks and Events"
  );
  const file_tomorrow_task = call_title(
    "task_preview",
    file_tomorrow_task_link
  );

  const file_today_pkm_link = link_alias(date + "_pkm", "Knowledge Review");
  const file_today_pkm = call_title("pkm", file_today_pkm_link);

  // DAILY JOURNALS CALLOUT TABLES
  journal_obj_arr
    .filter((x) => x.key === "Reflection")
    .map(
      (x) => (x.title_name = x.full_value.replace("_date_prev_", date_prev))
    );

  const journal_button_call_tbl_row = (button, file_section_name, alias) =>
    call_tbl_row(
      [
        button,
        link_tbl_alias(`${date_value}_${file_section_name}`, alias),
      ].join(tbl_pipe)
    );
  const journal_button_call_tbl = (obj_key) =>
    journal_obj_arr
      .filter((x) => x.key === obj_key)
      .map(
        (x) =>
          [
            x.call_title,
            call_start,
            journal_button_call_tbl_row(x.button, x.file_section_name, x.alias),
            call_tbl_div(2),
          ].join(new_line) + new_line
      );

  const reflection_callout = journal_button_call_tbl("Reflection");
  const win_loss_callout = journal_button_call_tbl("Wins and Losses");
  const gratitude_callout = journal_button_call_tbl("Gratitude");
  const detach_callout = journal_button_call_tbl("Detachment");

  hab_rit_obj_arr.forEach((obj) => {
    // FILE NAME AND ALIAS ARRAY
    obj.file_name = generate_hab_rit_file_name(
      date_value,
      obj.type,
      obj.value,
      obj.full_value
    );
    obj.file_alias = generate_hab_rit_file_alias_arr(
      date,
      obj.name,
      obj.full_name,
      obj.value,
      obj.full_value
    )
      .map((alias) => yaml_li(alias))
      .join("");

    // TODAY AND TOMORROW LINKS AND TABLE LINKS
    obj.link_today = link_alias(obj.file_name, obj.key);
    obj.link_tbl_today = link_tbl_alias(obj.file_name, obj.key);
    obj.link_tomorrow = obj.link_today.replace(date_value, date_next_value);
    obj.link_tbl_tomorrow = obj.link_tbl_today.replace(
      date_value,
      date_next_value
    );

    // PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
    obj.parent_value = generate_hab_rit_parent_value(
      year_month_short,
      obj.type,
      obj.ord,
      obj.key,
      obj.value
    );
    obj.parent_name = generate_hab_rit_parent_name(
      year_month_long,
      obj.type,
      obj.key,
      obj.name
    );
    obj.parent_yaml = yaml_li(link_alias(obj.parent_value, obj.parent_name));
    obj.file_path = generate_hab_rit_path(
      project_value,
      obj.parent_value,
      obj.file_name
    );
  });

  // SPECIFIC HABIT AND RITUALS BUTTON-LINK CALLOUTS
  const call_hab_rit_link = (hab_rit_type) =>
    [
      hr_line + new_line,
      hab_rit_obj_arr
        .filter((x) => x.type == hab_rit_type)
        .map((x) =>
          [
            call_title(x.type, `Today's ${link_alias(x.file_name, x.name)}`),
            call_start,
            call_start + code_inline(["button", x.button, "today"].join("-")),
          ].join(new_line)
        ),
      new_line + hr_line,
    ].join(new_line) + new_line;

  // CALLOUT LINKS
  const hab_rit_table = (day_relative) =>
    [
      hab_rit_button_table(day_relative),
      call_tbl_row(
        hab_rit_obj_arr
          .map((x) =>
            day_relative == "Today" ? x.link_tbl_today : x.link_tbl_tomorrow
          )
          .join(tbl_pipe)
      ),
    ].join(new_line);

  // SET OBJECT CONTENT
  set_content(hab_late_obj_arr, "key", "detach", detach_callout);
  set_content(hab_late_obj_arr, "key", "gratitude", gratitude_callout);
  set_content(
    morn_start_obj_arr,
    "head",
    "Plan the Day",
    [file_today_task_due, head_morn_journal, daily_journal_button].join(
      two_new_line
    )
  );
  set_content(
    morn_start_obj_arr,
    "head",
    "Recount Yesterday",
    reflection_callout
  );
  set_content(
    morn_start_obj_arr,
    "head",
    "Yesterday's Wins and Losses",
    win_loss_callout
  );
  set_content(
    morn_start_obj_arr,
    "head",
    "Give Thanks",
    [gratitude_callout, call_hab_rit_link("workday_startup_ritual")].join(
      two_new_line
    )
  );
  set_content(shut_eve_obj_arr, "head", "Knowledge Review", file_today_pkm);
  set_content(shut_eve_obj_arr, "head", "Review the Day", file_today_task_done);
  set_content(
    shut_eve_obj_arr,
    "head",
    "Get a Jump on Tomorrow",
    file_tomorrow_task
  );

  // SET BASE TASK START TIMES, SUGGESTER VALUES, AND YAML
  const time_prompt = (prompt, time_in) =>
    [weekday, prompt, "Start Time (esc default:", time_in].join(space) + "?)";

  // Habits and Rituals Start Times
  const time_morning_prompt = time_prompt(
    "SRS Review and Day Starting Rituals",
    default_time_morning
  );
  let time_morning = await tp.user.nl_time(tp, time_morning_prompt);
  time_morning = set_start_time(default_time_morning, time_morning);

  const time_habit_early_prompt = time_prompt(
    "Early Afternoon Habits",
    default_time_habit_early
  );
  let time_habit_early = await tp.user.nl_time(tp, time_habit_early_prompt);
  time_habit_early = set_start_time(default_time_habit_early, time_habit_early);

  const time_habit_late_prompt = time_prompt(
    "Late Afternoon Habits",
    default_time_habit_late
  );
  let time_habit_late = await tp.user.nl_time(tp, time_habit_late_prompt);
  time_habit_late = set_start_time(default_time_habit_late, time_habit_late);

  const time_evening_prompt = time_prompt(
    "SRS Review and Day Ending Rituals",
    default_time_evening
  );
  let time_evening = await tp.user.nl_time(tp, time_evening_prompt);
  time_evening = set_start_time(default_time_evening, time_evening);

  // TASK TIMES FOR HABITS
  set_task_info(hab_srs_obj_arr, date, (arr, index) =>
    index === 0
      ? date_fmt("HH:mm", date, time_morning)
      : date_fmt("HH:mm", date, time_evening)
  );
  set_task_info(hab_early_obj_arr, date, (arr, index) =>
    index === 0
      ? date_fmt("HH:mm", date, time_habit_early)
      : date_add_sub_fmt("HH:mm", "minutes", 1, date, arr[index - 1].end)
  );
  set_task_info(hab_late_obj_arr, date, (arr, index) =>
    index === 0
      ? date_fmt("HH:mm", date, time_habit_late)
      : date_add_sub_fmt("HH:mm", "minutes", 1, date, arr[index - 1].end)
  );
  set_task_info(morn_start_obj_arr, date, (arr, index) =>
    index === 0
      ? date_add_sub_fmt("HH:mm", "minutes", 1, date, hab_srs_obj_arr[0].end)
      : date_add_sub_fmt("HH:mm", "minutes", 1, date, arr[index - 1].end)
  );
  set_task_info(shut_eve_obj_arr, date, (arr, index) =>
    index === 0
      ? date_add_sub_fmt("HH:mm", "minutes", 1, date, hab_srs_obj_arr[1].end)
      : date_add_sub_fmt("HH:mm", "minutes", 1, date, arr[index - 1].end)
  );

  // SEPARATE AND ORGANIZE OBJECT ARRAYS BY TYPE
  const hab_head_srs_obj_arr = [
    ...hab_head_obj_arr.filter((x) => x.key == "srs"),
    ...hab_srs_obj_arr,
  ];
  const hab_meditation_obj_arr = [
    ...hab_head_obj_arr,
    ...hab_late_obj_arr,
  ].filter((x) => x.key == "meditation");

  const hab_obj_arr = [
    ...hab_head_srs_obj_arr,
    ...hab_obj_arr_filter("detach"),
    ...hab_obj_arr_filter("gratitude"),
    ...hab_obj_arr_filter("fit"),
    ...hab_meditation_obj_arr,
  ];
  const morn_obj_arr = morn_start_obj_arr.filter(
    (x) => x.type == "morning_ritual"
  );
  const work_start_obj_arr = morn_start_obj_arr.filter(
    (x) => x.type == "workday_startup_ritual"
  );
  const work_shut_obj_arr = shut_eve_obj_arr.filter(
    (x) => x.type == "workday_shutdown_ritual"
  );
  const eve_obj_arr = shut_eve_obj_arr.filter(
    (x) => x.type == "evening_ritual"
  );

  for (let j = 0; j < hab_rit_obj_arr.length; j++) {
    hab_rit_obj_arr[j].pillar = null_yaml_li;
    hab_rit_obj_arr[j].organization = null_yaml_li;
    hab_rit_obj_arr[j].contact = null_yaml_li;
    if (hab_rit_obj_arr[j].type == "habit") {
      hab_rit_obj_arr[j].pillar = pillar_mental_yaml + pillar_physical_yaml;
      hab_rit_obj_arr[j].organization = null_yaml_li;
      hab_rit_obj_arr[j].contact = null_yaml_li;
    } else if (
      hab_rit_obj_arr[j].type == "morning_ritual" ||
      hab_rit_obj_arr[j].type == "evening_ritual"
    ) {
      hab_rit_obj_arr[j].pillar = pillar_mental_yaml;
      hab_rit_obj_arr[j].organization = null_yaml_li;
      hab_rit_obj_arr[j].contact = null_yaml_li;
    } //  else {
    //     pillar = await multi_suggester(
    //       weekday,
    //       hab_rit_obj_arr[j].name,
    //       "Pillar",
    //       pillars_obj_arr
    //     );
    //     organization = await multi_suggester(
    //       weekday,
    //       hab_rit_obj_arr[j].name,
    //       "Organization",
    //       org_obj_arr
    //     );
    //     contact = await multi_suggester(
    //       weekday,
    //       hab_rit_obj_arr[j].name,
    //       "Contact",
    //       contact_obj_arr
    //     );
    // }
  }

  // PAGE CONTENT
  hab_rit_obj_arr.forEach((obj) => {
    // YAML PROPERTIES
    obj.yaml_properties = generate_yaml_properties(
      obj,
      date_link,
      project_yaml
    );

    // TOP PAGE CONTENT
    obj.content_top = obj.info;
    if (obj.type === "morning_ritual") {
      obj.content_top += new_line + hab_rit_table("Today") + new_line;
    } else if (obj.type === "evening_ritual") {
      obj.content_top += new_line + hab_rit_table("Tomorrow") + new_line;
    }

    // BOTTOM PAGE CONTENT
    if (obj.type === "habit") {
      obj.content_bottom = hab_obj_arr
        .map(
          (x) =>
            (x.heading ? new_line + x.heading : "") +
            (x.task_checkbox ? x.task_checkbox : "") +
            (x.content ? new_line + x.content + new_line : "")
        )
        .join("");
    } else if (obj.type === "morning_ritual") {
      obj.content_bottom = morn_obj_arr
        .map(
          (x) =>
            (x.heading ? x.heading : "") +
            (x.task_checkbox ? x.task_checkbox : "") +
            (x.content ? new_line + x.content + new_line : "")
        )
        .join(new_line);
    } else if (obj.type === "workday_startup_ritual") {
      obj.content_bottom =
        work_start_obj_arr
          .map((x) => x.heading + x.task_checkbox)
          .join(new_line) +
        new_line +
        call_hab_rit_link("morning_ritual");
    } else if (obj.type === "workday_shutdown_ritual") {
      obj.content_bottom =
        work_shut_obj_arr
          .map(
            (x) =>
              [x.heading, x.task_checkbox].join(new_line) +
              (x.content ? two_new_line + x.content : "")
          )
          .join(two_new_line) +
        two_new_line +
        call_hab_rit_link("evening_ritual");
    } else if (obj.type === "evening_ritual") {
      obj.content_bottom =
        eve_obj_arr
          .map((x) =>
            [new_line, x.heading, x.task_checkbox, x.content].join(new_line)
          )
          .join(two_new_line) +
        two_new_line +
        call_hab_rit_link("workday_shutdown_ritual");
    }
    obj.file_content = [
      obj.yaml_properties,
      obj.content_top,
      obj.content_bottom,
    ].join(new_line);
  });

  for (let j = 0; j < hab_rit_obj_arr.length; j++) {
    //if (!(i === weekday_arr.length - 1 && j === hab_rit_obj_arr[j].ord === 5)) {
      await app.vault.create(
        hab_rit_obj_arr[j].file_path,
        hab_rit_obj_arr[j].file_content
      );
    //} else {
    //  last_file_content = hab_rit_obj_arr[j].file_content;
    //  last_file_path = hab_rit_obj_arr[j].file_path;
    //  last_file_path = last_file_path.replace(/\.md$/g, "");
    //  console.log(last_file_path);
    //}
  }
}
//await tp.file.move(`${last_file_path}`);
//tR += last_file_content;
%>
