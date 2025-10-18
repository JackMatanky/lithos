<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const habit_ritual_proj_dir = "45_habit_ritual/";
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

const call_tbl_div = (int) =>
  call_tbl_start + Array(int).fill(tbl_cent).join(tbl_pipe) + call_tbl_end;

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");
const task_date_created = moment().format("YYYY-MM-DD");

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
const null_arr = ["", "null", "[[null|Null]]", null];
const null_link = "[[null|Null]]";
const null_yaml_li = yaml_li(null_link);

const dv_yaml = "file.frontmatter";
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

//-------------------------------------------------------------------
// DAILY JOURNALS HEADING AND BUTTON
//-------------------------------------------------------------------
const head_morn_journal = head_lvl(3) + "Morning Journals";
const daily_journal_button =
  [
    `${three_backtick}button`,
    "name 🕯️Daily Journals",
    "type note(Untitled, split) template",
    "action 90_01_daily_journals_preset",
    "color purple",
    `${three_backtick}`,
  ].join(new_line) + two_new_line;

//-------------------------------------------------------------------
// REFLECTION, GRATITUDE, AND DETACHMENT JOURNAL VARIABLES
//-------------------------------------------------------------------
const reflect_button = `${backtick}button-reflection-daily-preset${backtick}`;
const reflect_alias = "Daily Reflection";
const reflect_value = reflect_alias.replaceAll(/\s/g, "_").toLowerCase();
const reflect_title = `${call_start}[!reflection]${space}${reflect_alias}${space}Journal`;

const win_loss_alias = "Daily Wins and Losses";
const win_loss_title = `${call_start}[!reflection]${space}${win_loss_alias}${space}Journal`;

const gratitude_button = `${backtick}button-gratitude-daily-preset${backtick}`;
const gratitude_alias = "Daily Gratitude";
const gratitude_value = gratitude_alias.replaceAll(/\s/g, "_").toLowerCase();
const gratitude_title = `${call_start}[!gratitude]${space}${gratitude_alias}${space}Journal`;

const detach_button = `${backtick}button-detachment-daily-preset${backtick}`;
const detach_alias = "Daily Detachment";
const detach_value = detach_alias.replaceAll(/\s/g, "_").toLowerCase();
const detach_title = `${call_start}[!detachment]${space}${detach_alias}${space}Journal`;

//-------------------------------------------------------------------
// GUIDED MEDITATION
//-------------------------------------------------------------------
const head_meditation_bell = head_lvl(4) + "Mindfulness Bell Meditation";
const embed_meditation_bell =
  "![[1_Five Minute Mindfulness Bell Meditation.mp3]]";

const head_meditation_positive_mind = head_lvl(4) + "Positive Mind Meditation";
const embed_meditation_positive_mind =
  "![[morn_1_Positive Mind in 5 Minutes Meditation_Jason Stephenson.mp3]]";

const meditation_content = [
  head_meditation_bell,
  embed_meditation_bell,
  head_meditation_positive_mind,
  embed_meditation_positive_mind,
].join(two_new_line);

//-------------------------------------------------------------------
// MENTAL AND PHYSICAL HEALTH PILLAR NAME AND LINK
//-------------------------------------------------------------------
const pillar_mental_value_link = yaml_li("[[mental_health|Mental Health]]");
const pillar_physical_value_link = yaml_li(
  "[[physical_health|Physical Health]]"
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
        full_name = contact_names.full_name;
        last_first_name = contact_names.last_first_name;
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
    .map((file) => `${new_line}${ul_yaml}"[[${file.value}|${file.key}]]"`)
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
  full_date = moment();
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
// CONTEXT NAME, VALUE, DIRECTORY, AND FILE CLASS
//-------------------------------------------------------------------
const context_name = "Habits and Rituals";
const context_value = "habit_ritual";
const context_dir = habit_ritual_proj_dir;
const file_class = "task_child";

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

// Name
hab_rit_obj_arr.map(
  (x) => (x.name = x.key == "Habits" ? x.key : x.key + " Rituals")
);
// Value: Snake-Case Name
hab_rit_obj_arr.map(
  (x) => (x.value = x.name.replaceAll(/[\s]/g, "_").toLowerCase())
);
// Type
hab_rit_obj_arr.map((x) => (x.type = x.value.replace(/s$/, "")));
// Full Name: Rate and Name
hab_rit_obj_arr.map((x) => (x.full_name = x.rate + " " + x.name));
// Full Name Value: Snake Case Full Name
hab_rit_obj_arr.map(
  (x) => (x.full_value = x.full_name.replaceAll(/[\s-]/g, "_").toLowerCase())
);

//-------------------------------------------------------------------
// TODAY AND TOMORROW HABIT AND RITUAL BUTTONS AND LINK TABLE
//-------------------------------------------------------------------
function hab_rit_button_table(day) {
  const hab_rit_button_link_title = `${call_start}[!${context_value}]${space}${day}'s${space}${context_name}`;
  const hab_rit_button_row =
    call_tbl_start +
    hab_rit_obj_arr
      .map(
        (x) =>
          backtick + "button-" + x.button + `-${day.toLowerCase()}` + backtick
      )
      .join(tbl_pipe) +
    call_tbl_end;
  return [
    hab_rit_button_link_title,
    call_start,
    hab_rit_button_row,
    call_tbl_div(5),
  ].join(new_line);
}

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_body = await tp.user.include_file("42_00_child_task_info_callout");
hab_rit_obj_arr.map(
  (x) =>
    (x.info = [
      head_lvl(1) + x.name + new_line,
      `${call_start}[!${x.type}]${space}${x.name}${space}Details`,
      call_start,
      info_body +
        head_lvl(2) +
        (x.type == "habit" ? x.key : "Rituals") +
        new_line,
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
morn_start_obj_arr.map(
  (x) => (x.heading = head_lvl(x.head_level) + x.head + two_new_line)
);

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
shut_eve_obj_arr.map((x) => (x.heading = head_lvl(3) + x.head));

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
hab_head_obj_arr.map((x) => (x.heading = head_lvl(3) + x.head + two_new_line));
hab_early_obj_arr.map(
  (x) => (x.heading = !x.head ? x.head : head_lvl(4) + x.head + two_new_line)
);
hab_late_obj_arr.map(
  (x) => (x.heading = !x.head ? x.head : head_lvl(4) + x.head + two_new_line)
);

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
const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}${space}`;

function task_text(text, type, date_due, start, end, duration, remind) {
  const task_title = checkbox_task_tag + text + "_" + type;
  const inline_time =
    space +
    [
      `[time_start${dv_colon}${start}]`,
      `[time_end${dv_colon}${end}]`,
      `[duration_est${dv_colon}${duration}]`,
    ].join(two_space);
  let inline_date =
    space + ["➕", task_date_created, "📅", date_due].join(space);
  if (remind) {
    inline_date =
      space +
      ["⏰", remind, "➕", task_date_created, "📅", date_due].join(space);
  }
  return task_title + inline_time + inline_date;
}

//-------------------------------------------------------------------
// YAML FRONTMATTER
//-------------------------------------------------------------------
const yaml_bottom = [
  `file_class:${space}${file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  "tags:",
  hr_line,
].join(new_line);

//-------------------------------------------------------------------
// CREATE HABITS AND RITUALS FILES
//-------------------------------------------------------------------
for (let i = 0; i < weekday_arr.length; i++) {
  // SET DATE, TITLE, ALIASES, PROJECT AND PARENT TASK
  const date = weekday_arr[i];
  const date_link = `"[[${date}]]"`;
  const date_value = moment(date).format("YY_MM_DD");
  const date_next = moment(date).add(1, "days").format("YYYY-MM-DD");
  const date_next_value = moment(date).add(1, "days").format("YY_MM_DD");
  const date_prev = moment(date).subtract(1, "days").format("YYYY-MM-DD");
  const weekday = moment(date).format("dddd");
  const year_month_short = moment(date).format("YYYY-MM");
  const year_month_long = moment(date).format("MMMM [']YY");

  // FILE NAME AND ALIAS ARRAY
  hab_rit_obj_arr.map(
    (x) =>
      (x.file_name =
        x.type == "habit"
          ? date_value + "_" + x.full_value
          : date_value + "_" + x.value)
  );
  hab_rit_obj_arr.map(
    (x) =>
      (x.file_alias =
        new_line +
        [
          x.full_name,
          x.full_name.toLowerCase(),
          x.full_value,
          date + " " + x.full_name,
          date_value + "_" + x.full_value,
          x.name,
          x.name.toLowerCase(),
          x.value,
          date + " " + x.name,
          date_value + "_" + x.value,
        ]
          .map((x) => `${ul_yaml}"${x}"`)
          .join(new_line))
  );

  // PROJECT FILE NAME, ALIAS AND LINK
  const project_value = `${year_month_short}_${context_value}`;
  const project_name = `${year_month_long} ${context_name}`;
  const project_value_link = `${new_line}${ul_yaml}"[[${project_value}|${project_name}]]"`;

  // PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
  hab_rit_obj_arr.map(
    (x) =>
      (x.parent_value =
        x.type == "habit"
          ? year_month_short + `_0${x.ord}_` + x.key.toLowerCase()
          : year_month_short + `_0${x.ord}_` + x.value)
  );
  hab_rit_obj_arr.map(
    (x) =>
      (x.parent_name =
        year_month_long + " " + (x.type == "habit" ? x.key : x.name))
  );
  hab_rit_obj_arr.map(
    (x) =>
      (x.parent_value_link = `${new_line}${ul_yaml}"[[${x.parent_value}|${x.parent_name}]]"`)
  );
  hab_rit_obj_arr.map(
    (x) =>
      (x.file_path =
        context_dir +
        project_value +
        "/" +
        x.parent_value +
        "/" +
        x.file_name +
        ".md")
  );

  // TODAY AND TOMORROW LINKS AND TABLE LINKS
  hab_rit_obj_arr.map(
    (x) => (x.link_today = "[[" + x.file_name + "|" + x.key + "]]")
  );
  hab_rit_obj_arr.map(
    (x) => (x.link_tbl_today = "[[" + x.file_name + "\\|" + x.key + "]]")
  );
  // "[[" + date_next_value + "_" + (x.ord == 1 ? x.full_value : x.value) + "|" + x.key + "]]")
  hab_rit_obj_arr.map(
    (x) => (x.link_tomorrow = x.link_today.replace(date_value, date_next_value))
  );
  hab_rit_obj_arr.map(
    (x) =>
      (x.link_tbl_tomorrow = x.link_tbl_today.replace(
        date_value,
        date_next_value
      ))
  );

  // CALLOUT LINKS
  const today_hab_rit_link_row =
    call_tbl_start +
    hab_rit_obj_arr.map((x) => x.link_tbl_today).join(tbl_pipe) +
    call_tbl_end;
  const today_hab_rit_table = [
    hab_rit_button_table("Today"),
    today_hab_rit_link_row,
  ].join(new_line);
  const tomorrow_hab_rit_link_row =
    call_tbl_start +
    hab_rit_obj_arr.map((x) => x.link_tbl_tomorrow).join(tbl_pipe) +
    call_tbl_end;
  const tomorrow_hab_rit_table = [
    hab_rit_button_table("Tomorrow"),
    tomorrow_hab_rit_link_row,
  ].join(new_line);

  const file_today_task_due =
    call_start +
    "[!task_plan] [[" +
    date +
    "_task_event#Due Today|Today's Schedule]]";
  const file_today_task_done =
    call_start +
    "[!task_review] [[" +
    date +
    "_task_event#Completed Today|Today's Completed Tasks]]";
  const file_today_pkm =
    call_start + "[!pkm] [[" + date + "_pkm|Knowledge Review]]";
  const file_tomorrow_task =
    call_start +
    "[!task_preview] [[" +
    date_next +
    "_task_event#Due Today|Tomorrow's Tasks and Events]]";

  const callout_button_table = (title, file_name, file_alias, button) =>
    [
      title,
      call_start,
      call_tbl_start +
        [button, `[[${date_value}_${file_name}\\|${file_alias}]]`].join(
          tbl_pipe
        ) +
        call_tbl_end,
      call_tbl_div(2),
    ].join(new_line);

  const reflect_name = `${reflect_value}#What Happened ${date_prev} Yesterday?`;
  const reflection_callout = callout_button_table(
    reflect_title,
    reflect_name,
    reflect_alias,
    reflect_button
  );

  const win_loss_name = `${reflect_value}#What Unplanned Occurrences Happened?`;
  const win_loss_callout = callout_button_table(
    win_loss_title,
    win_loss_name,
    win_loss_alias,
    reflect_button
  );

  const gratitude_name = `${gratitude_value}#I Am Grateful For…`;
  const gratitude_callout = callout_button_table(
    gratitude_title,
    gratitude_name,
    gratitude_alias,
    gratitude_button
  );

  const detach_name = detach_value;
  const detach_callout = callout_button_table(
    detach_title,
    detach_name,
    detach_alias,
    detach_button
  );

  // SET BASE TASK START TIMES, SUGGESTER VALUES, AND YAML
  const datetime_full = (time_in) => moment(`${date}T${time_in}`);
  const datetime_plus_min = (time_in) =>
    moment(`${date}T${time_in}`).add(1, "minutes");
  const hour_min = (time_in) => moment(`${date}T${time_in}`).format("HH:mm");
  const hour_min_plus_min = (time_in) =>
    moment(`${date}T${time_in}`).add(1, "minutes").format("HH:mm");
  const time_end = (time_in, dur) =>
    moment(`${date}T${time_in}`).add(dur, "minutes").format("HH:mm");
  const time_remind = (time_in, remind_num) =>
    moment(`${date}T${time_in}`)
      .subtract(remind_num, "minutes")
      .format("YYYY-MM-DD HH:mm");
  const time_prompt = (prompt, time_in) =>
    [weekday, prompt, "Start Time (esc default:", hour_min(time_in)].join(
      space
    ) + "?)";
  // Habits and Rituals Start Times
  let time_morning = await tp.user.nl_time(
    tp,
    time_prompt("SRS Review and Day Starting Rituals", default_time_morning)
  );
  let time_habit_early = await tp.user.nl_time(
    tp,
    time_prompt("Early Afternoon Habits", default_time_habit_early)
  );
  let time_habit_late = await tp.user.nl_time(
    tp,
    time_prompt("Late Afternoon Habits", default_time_habit_late)
  );
  let time_evening = await tp.user.nl_time(
    tp,
    time_prompt("SRS Review and Day Ending Rituals", default_time_evening)
  );
  if (null_arr.includes(time_morning)) {
    time_morning = default_time_morning;
  }
  if (null_arr.includes(time_habit_early)) {
    time_habit_early = default_time_habit_early;
  }
  if (null_arr.includes(time_habit_late)) {
    time_habit_late = default_time_habit_late;
  }
  if (null_arr.includes(time_evening)) {
    time_evening = default_time_evening;
  }
  for (let j = 0; j < hab_rit_obj_arr.length; j++) {
    let pillar = null_yaml_li;
    let organization = null_yaml_li;
    let contact = null_yaml_li;
    if (hab_rit_obj_arr[j].type == "habit") {
      pillar = pillar_mental_value_link + pillar_physical_value_link;
      organization = null_yaml_li;
      contact = null_yaml_li;
    } else if (
      hab_rit_obj_arr[j].type == "morning_ritual" ||
      hab_rit_obj_arr[j].type == "evening_ritual"
    ) {
      pillar = pillar_mental_value_link;
      organization = null_yaml_li;
      contact = null_yaml_li;
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
    // TOP CONTENT
    hab_rit_obj_arr[j].content_top = [
      hr_line,
      `title:${space}${hab_rit_obj_arr[j].file_name}`,
      `aliases:${space}${hab_rit_obj_arr[j].file_alias}`,
      `date:${space}${date_link}`,
      "due_do: do",
      `pillar:${pillar}`,
      `context:${space}${context_value}`,
      "goal: null",
      `project:${project_value_link}`,
      `parent_task:${hab_rit_obj_arr[j].parent_value_link}`,
      `organization:${organization}`,
      `contact:${contact}`,
      `library:${null_yaml_li}`,
      `type:${space}${hab_rit_obj_arr[j].type}`,
      yaml_bottom,
      hab_rit_obj_arr[j].info,
    ].join(new_line);
    if (hab_rit_obj_arr[j].type == "morning_ritual") {
      hab_rit_obj_arr[j].content_top += new_line + today_hab_rit_table;
    } else if (hab_rit_obj_arr[j].type == "evening_ritual") {
      hab_rit_obj_arr[j].content_top += new_line + tomorrow_hab_rit_table;
    }
    // TASK TIMES FOR HABITS
    hab_srs_obj_arr[0].start = hour_min(time_morning);
    hab_srs_obj_arr[0].end = time_end(
      hab_srs_obj_arr[0].start,
      hab_srs_obj_arr[0].duration
    );
    hab_srs_obj_arr[0].reminder = time_remind(
      hab_srs_obj_arr[0].start,
      hab_srs_obj_arr[0].remind
    );
    hab_srs_obj_arr[1].start = hour_min(time_evening);
    hab_srs_obj_arr[1].end = time_end(
      hab_srs_obj_arr[1].start,
      hab_srs_obj_arr[1].duration
    );
    hab_srs_obj_arr[1].reminder = time_remind(
      hab_srs_obj_arr[1].start,
      hab_srs_obj_arr[1].remind
    );
    for (let k = 0; k < hab_early_obj_arr.length; k++) {
      if (k == 0) {
        hab_early_obj_arr[k].start = hour_min(time_habit_early);
      } else {
        hab_early_obj_arr[k].start = hour_min_plus_min(
          hab_early_obj_arr[k - 1].end
        );
      }
      hab_early_obj_arr[k].end = time_end(
        hab_early_obj_arr[k].start,
        hab_early_obj_arr[k].duration
      );
      hab_early_obj_arr[k].reminder = !hab_early_obj_arr[k].remind
        ? hab_early_obj_arr[k].remind
        : time_remind(hab_early_obj_arr[k].start, hab_early_obj_arr[k].remind);
    }
    for (let k = 0; k < hab_late_obj_arr.length; k++) {
      if (k == 0) {
        hab_late_obj_arr[k].start = hour_min(time_habit_late);
      } else {
        hab_late_obj_arr[k].start = hour_min_plus_min(
          hab_late_obj_arr[k - 1].end
        );
      }
      hab_late_obj_arr[k].end = time_end(
        hab_late_obj_arr[k].start,
        hab_late_obj_arr[k].duration
      );
      hab_late_obj_arr[k].reminder = !hab_late_obj_arr[k].remind
        ? hab_late_obj_arr[k].remind
        : time_remind(hab_late_obj_arr[k].start, hab_late_obj_arr[k].remind);
    }
    hab_late_obj_arr[0].content = new_line + detach_callout + new_line;
    hab_late_obj_arr[1].content = new_line + gratitude_callout + new_line;

    // TASK TIMES FOR MORN AND WORK START
    for (let k = 0; k < morn_start_obj_arr.length; k++) {
      if (k == 0) {
        morn_start_obj_arr[k].start = hour_min_plus_min(hab_srs_obj_arr[0].end);
      } else {
        morn_start_obj_arr[k].start = hour_min_plus_min(
          morn_start_obj_arr[k - 1].end
        );
      }
      morn_start_obj_arr[k].end = time_end(
        morn_start_obj_arr[k].start,
        morn_start_obj_arr[k].duration
      );
      morn_start_obj_arr[k].reminder = !morn_start_obj_arr[k].remind
        ? morn_start_obj_arr[k].remind
        : time_remind(
            morn_start_obj_arr[k].start,
            morn_start_obj_arr[k].remind
          );
    }
    morn_start_obj_arr.map(
      (x) =>
        (x.task_checkbox = !x.task
          ? x.task
          : task_text(
              x.task,
              x.type,
              date,
              x.start,
              x.end,
              x.duration,
              x.reminder
            ))
    );

    // TASK TIMES FOR WORK STOP AND EVE
    for (let k = 0; k < shut_eve_obj_arr.length; k++) {
      if (k == 0) {
        shut_eve_obj_arr[k].start = hour_min_plus_min(hab_srs_obj_arr[1].end);
      } else {
        shut_eve_obj_arr[k].start = hour_min_plus_min(
          shut_eve_obj_arr[k - 1].end
        );
      }
      shut_eve_obj_arr[k].end = time_end(
        shut_eve_obj_arr[k].start,
        shut_eve_obj_arr[k].duration
      );
      shut_eve_obj_arr[k].reminder = !shut_eve_obj_arr[k].remind
        ? shut_eve_obj_arr[k].remind
        : time_remind(shut_eve_obj_arr[k].start, shut_eve_obj_arr[k].remind);
    }
    shut_eve_obj_arr.map(
      (x) =>
        (x.task_checkbox = !x.task
          ? x.task
          : task_text(
              x.task,
              x.type,
              date,
              x.start,
              x.end,
              x.duration,
              x.reminder
            ))
    );

    // SPECIFIC HABIT AND RITUALS BUTTON-LINK CALLOUTS
    function call_hab_rit_link(hab_rit_type) {
      const title = hab_rit_obj_arr
        .filter((x) => x.type == hab_rit_type)
        .map(
          (x) =>
            `${call_start}[!${x.type}]${space}Today's${space}[[${x.file_name}|${x.name}]]`
        );
      const body = hab_rit_obj_arr
        .filter((x) => x.type == hab_rit_type)
        .map(
          (x) =>
            call_start + backtick + "button-" + x.button + "-today" + backtick
        );
      return (
        [hr_line + new_line, title, call_start, body, new_line + hr_line].join(
          new_line
        ) + new_line
      );
    }

    // SEPARATE AND ORGANIZE OBJECT ARRAYS BY TYPE
    const hab_obj_arr = [
      [hab_head_obj_arr.filter((x) => x.key == "srs"), hab_srs_obj_arr].flat(),
      [hab_head_obj_arr, hab_early_obj_arr, hab_late_obj_arr]
        .flat()
        .filter((x) => x.key == "detach"),
      [hab_head_obj_arr, hab_early_obj_arr, hab_late_obj_arr]
        .flat()
        .filter((x) => x.key == "gratitude"),
      [hab_head_obj_arr, hab_early_obj_arr, hab_late_obj_arr]
        .flat()
        .filter((x) => x.key == "fit"),
      [hab_head_obj_arr, hab_late_obj_arr]
        .flat()
        .filter((x) => x.key == "meditation"),
    ].flat();
    hab_obj_arr.map(
      (x) =>
        (x.task_checkbox = !x.task
          ? x.task
          : task_text(
              x.task,
              x.type,
              date,
              x.start,
              x.end,
              x.duration,
              x.reminder
            ) + new_line)
    );

    const morn_obj_arr = morn_start_obj_arr.filter(
      (x) => x.type == "morning_ritual"
    );
    morn_obj_arr[1].content = [
      file_today_task_due,
      head_morn_journal,
      daily_journal_button,
    ].join(two_new_line);
    morn_obj_arr[2].content = reflection_callout;
    morn_obj_arr[3].content = win_loss_callout;
    morn_obj_arr[4].content =
      gratitude_callout +
      two_new_line +
      call_hab_rit_link("workday_startup_ritual");

    const work_start_obj_arr = morn_start_obj_arr.filter(
      (x) => x.type == "workday_startup_ritual"
    );

    const work_shut_obj_arr = shut_eve_obj_arr.filter(
      (x) => x.type == "workday_shutdown_ritual"
    );
    work_shut_obj_arr[0].content = file_today_pkm;
    work_shut_obj_arr[work_shut_obj_arr.length - 1].content =
      file_today_task_done;

    const eve_obj_arr = shut_eve_obj_arr.filter(
      (x) => x.type == "evening_ritual"
    );
    eve_obj_arr[0].content = file_tomorrow_task;

    // PAGE CONTENT
    if (hab_rit_obj_arr[j].type == "habit") {
      hab_rit_obj_arr[j].content_bottom = hab_obj_arr
        .map(
          (x) =>
            (x.heading ? new_line + x.heading : "") +
            (x.task_checkbox ? x.task_checkbox : "") +
            (x.content ? x.content : "")
        )
        .join("");
    } else if (hab_rit_obj_arr[j].type == "morning_ritual") {
      hab_rit_obj_arr[j].content_bottom = morn_obj_arr
        .map(
          (x) =>
            (x.heading ? x.heading : "") +
            (x.task_checkbox ? x.task_checkbox + new_line : "") +
            (x.content ? new_line + x.content : "")
        )
        .join(new_line);
    } else if (hab_rit_obj_arr[j].type == "workday_startup_ritual") {
      hab_rit_obj_arr[j].content_bottom =
        work_start_obj_arr
          .map((x) => x.heading + x.task_checkbox)
          .join(two_new_line) +
        two_new_line +
        call_hab_rit_link("morning_ritual");
    } else if (hab_rit_obj_arr[j].type == "workday_shutdown_ritual") {
      hab_rit_obj_arr[j].content_bottom =
        work_shut_obj_arr
          .map(
            (x) =>
              [x.heading, x.task_checkbox].join(two_new_line) +
              (x.content ? two_new_line + x.content : "")
          )
          .join(two_new_line) +
        two_new_line +
        call_hab_rit_link("evening_ritual");
    } else if (hab_rit_obj_arr[j].type == "evening_ritual") {
      hab_rit_obj_arr[j].content_bottom =
        eve_obj_arr
          .map((x) =>
            [x.heading, x.task_checkbox, x.content].join(two_new_line)
          )
          .join(two_new_line) +
        two_new_line +
        call_hab_rit_link("workday_shutdown_ritual");
    }

    const file_content = [
      hab_rit_obj_arr[j].content_top,
      hab_rit_obj_arr[j].content_bottom,
    ].join(new_line);
    const file_path = hab_rit_obj_arr[j].file_path;
    await app.vault.create(file_path, file_content);
  }
}
%>