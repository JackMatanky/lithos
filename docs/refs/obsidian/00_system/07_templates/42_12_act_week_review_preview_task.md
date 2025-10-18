<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const cal_week_dir = "10_calendar/12_weeks/";
const goals_dir = "30_goals/";
const personal_proj_dir = "41_personal/";
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

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Personal";
const context_value = context_name.toLowerCase();
const context_dir = personal_proj_dir;

//-------------------------------------------------------------------
// PROJECT FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const project_name = "Durational Reviews";
const project_value = project_name.replaceAll(/\s/g, "_").toLowerCase();
const project_value_link = yaml_li(`[[${project_value}|${project_name}]]`);
const project_dir = `${context_dir}${project_value}/`;

//-------------------------------------------------------------------
// PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const parent_task_name = "Weekly Reviews";
const parent_task_value = parent_task_name.replaceAll(/\s/g, "_").toLowerCase();
const parent_task_value_link = yaml_li(
  `[[${parent_task_value}|${parent_task_name}]]`
);
const parent_task_dir = `${project_dir}${parent_task_value}`;

//-------------------------------------------------------------------
// ACTION ITEM TAG, TYPE, AND FILE CLASS
//-------------------------------------------------------------------
const task_tag = "#task";
const type_name = "Action Item";
const type_value = type_name.replaceAll(/\s/g, "_").toLowerCase();
const file_class = "task_child";

//-------------------------------------------------------------------
// SET WEEK
//-------------------------------------------------------------------
const week_file_regex = /\d{4}.W\d{2}$/;
const week_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: cal_week_dir,
  file_class: "cal_week",
  type: "",
});
const week_obj = await tp.system.suggester(
  (item) => item.key,
  week_obj_arr.filter((f) => f.value.match(week_file_regex)).reverse(),
  false,
  "Preview Week?"
);
const week_preview_value = week_obj.value;
const week_preview_name = week_obj.key;
const week_preview_link = `[[${week_preview_value}|${week_preview_name}]]`;

const week_preview_year = Number(week_preview_value.split("-W")[0]);
const week_preview_num = Number(week_preview_value.split("-W")[1]);

let week_review_year = week_preview_year;
let week_review_num = week_preview_num - 1;
if (week_preview_num == 1) {
  week_review_year = week_preview_year - 1;
  week_review_num = 52;
} else if (week_review_num < 10) {
  week_review_num = `0${week_review_num}`;
}

const week_review_value = `${week_preview_year}-W${week_review_num}`;
const week_review_name = `Week${space}${week_review_num},${space}${week_preview_year}`;
const week_review_link = `[[${week_review_value}|${week_review_name}]]`;

//-------------------------------------------------------------------
// WEEKLY REFLECTION FILE EMBED
//-------------------------------------------------------------------
const reflection_alias = "Weekly Reflection";
const reflection_value = reflection_alias.replaceAll(/\s/g, "_").toLowerCase();
const reflection_file_name = `${week_review_value}_${reflection_value}`;
const reflection_link = `[[${reflection_file_name}|${reflection_alias}]]`;
const reflection_callout =
  `${call_start}[!reflection]${space}${reflection_link}` +
  (new_line + call_start).repeat(2) +
  `${backtick}button-reflection-weekly${backtick}` +
  two_new_line;

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  "20_03_pillar_name_alias_preset_mental"
);
const pillar_value = pillar_name_alias.split(";")[0];
const pillar_value_link = pillar_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, "Goal?");

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const org_name_alias = await tp.user.include_template(
  tp,
  "52_organization_name_alias"
);
const organization_value = org_name_alias.split(";")[0];
const organization_value_link = org_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
const contact_name_alias = await tp.user.include_template(
  tp,
  "51_contact_name_alias"
);
const contact_value = contact_name_alias.split(";")[0];
const contact_name = contact_name_alias.split(";")[1];
const contact_value_link = contact_name_alias.split(";")[2];

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const do_due_date = await tp.user.include_template(tp, "40_task_do_due_date");
const due_do_value = do_due_date.split(";")[0];
const due_do_name = do_due_date.split(";")[1];

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, "40_task_status");
const status_value = task_status.split(";")[0];
const status_name = task_status.split(";")[1];
const status_symbol = task_status.split(";")[2];

const checkbox_task_tag = `${ul}[${status_symbol}]${space}${task_tag}${space}`;
const inline_creation_date = `${space}➕${space}${task_date_created}`;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    key: "Tasks and Events",
    value: "Tasks and Events",
    file: null,
  },
  {
    key: "Related Tasks and Events",
    value: "Related Tasks",
    file: "142_00_related_sect_task_child",
  },
  {
    key: "Related Knowledge",
    value: "PKM",
    file: "100_70_related_pkm_sect",
  },
  {
    key: "Related Library Content",
    value: "Library",
    file: "100_60_related_lib_sect",
  },
  {
    key: "Related Directory",
    value: "Directory",
    file: "100_50_related_dir_sect",
  },
];

// Content and heading
for (let i = 0; i < section_obj_arr.length; i++) {
  if (!section_obj_arr[i].file) {
    continue;
  }
  section_obj_arr[i].content = await tp.user.include_template(
    tp,
    section_obj_arr[i].file
  );
}
section_obj_arr.map((x) => (x.head = head_lvl(2) + x.key + two_new_line));
section_obj_arr.map((x) => (x.toc = `[[_file_section_${x.key}\\|${x.value}]]`));

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_title = `${call_start}[!toc]${space}${dv_content_link}`;

const toc_body =
  call_tbl_start +
  section_obj_arr.map((x) => x.toc).join(tbl_pipe) +
  call_tbl_end;

const toc =
  [toc_title, call_start, toc_body, call_tbl_div(5)].join(new_line) +
  two_new_line;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_callout = await tp.user.include_file(
  "42_10_act_week_review_preview_info_callout"
);
const info = info_callout
  .toString()
  .replace("__review__", week_review_link)
  .replace("__preview__", week_preview_link);

//-------------------------------------------------------------------
// TASKS AND EVENTS ACTION ITEM OBJECT ARRAYS
//-------------------------------------------------------------------
const task_rev_prev = await tp.user.include_template(
  tp,
  "42_40_task_review_preview"
);

const task_sunday = task_rev_prev.split(";")[0];
const task_monday = task_rev_prev.split(";")[1];
const task_tuesday = task_rev_prev.split(";")[2];
const task_wednesday = task_rev_prev.split(";")[3];
const task_thursday = task_rev_prev.split(";")[4];
const task_friday = task_rev_prev.split(";")[5];
const task_saturday = task_rev_prev.split(";")[6];
const task_parent_done = task_rev_prev.split(";")[7];
const task_week_plan_review = task_rev_prev.split(";")[8];
const task_project_active = task_rev_prev.split(";")[9];
const task_parent_active = task_rev_prev.split(";")[10];
const task_week_plan_preview = task_rev_prev.split(";")[11];
const task_daily_schedule = task_rev_prev.split(";")[12];

const tasks_obj_arr = [
  {
    type: "Review",
    key: "Sunday Tasks",
    content: task_sunday,
    duration: 3,
    remind: 5,
  },
  {
    type: "Review",
    key: "Monday Tasks",
    content: task_monday,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Tuesday Tasks",
    content: task_tuesday,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Wednesday Tasks",
    content: task_wednesday,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Thursday Tasks",
    content: task_thursday,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Friday Tasks",
    content: task_friday,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Saturday Tasks",
    content: task_saturday,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Completed Parent Tasks",
    content: task_parent_done,
    duration: 3,
    remind: 5,
  },
  {
    type: "Review",
    key: "Weekly Plan",
    content: task_week_plan_review,
    duration: 5,
    remind: false,
  },
  {
    type: "Preview",
    key: "Active Projects",
    content: task_project_active,
    duration: 5,
    remind: false,
  },
  {
    type: "Preview",
    key: "Active Parent Tasks",
    content: task_parent_active,
    duration: 5,
    remind: false,
  },
  {
    type: "Preview",
    key: "Weekly Plan",
    content: task_week_plan_preview,
    duration: 5,
    remind: false,
  },
  {
    type: "Preview",
    key: "Daily Tasks and Events",
    content: task_daily_schedule,
    duration: 15,
    remind: 5,
  },
];

//-------------------------------------------------------------------
// HABITS AND RITUALS ACTION ITEM OBJECT ARRAYS
//-------------------------------------------------------------------
const hab_rit_rev_prev = await tp.user.include_template(
  tp,
  "42_45_hab_rit_review_preview"
);

const hab_rit_habit = hab_rit_rev_prev.split(";")[0];
const hab_rit_morning = hab_rit_rev_prev.split(";")[1];
const hab_rit_work_start = hab_rit_rev_prev.split(";")[2];
const hab_rit_work_stop = hab_rit_rev_prev.split(";")[3];
const hab_rit_evening = hab_rit_rev_prev.split(";")[4];
const hab_rit_next_week = hab_rit_rev_prev.split(";")[5];

const habit_ritual_obj_arr = [
  {
    type: "Review",
    key: "Habits",
    content: hab_rit_habit,
    duration: 3,
    remind: 5,
  },
  {
    type: "Review",
    key: "Morning Rituals",
    content: hab_rit_morning,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Workday Startup Rituals",
    content: hab_rit_work_start,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Workday Shutdown Rituals",
    content: hab_rit_work_stop,
    duration: 3,
    remind: false,
  },
  {
    type: "Review",
    key: "Evening Rituals",
    content: hab_rit_evening,
    duration: 3,
    remind: false,
  },
  {
    type: "Preview",
    key: "Upcoming Habits and Rituals",
    content: hab_rit_next_week,
    duration: 3,
    remind: 5,
  },
];

//-------------------------------------------------------------------
// YAML FRONTMATTER
//-------------------------------------------------------------------
const yaml_bottom = [
  `due_do:${space}do`,
  `pillar:${pillar_value_link}`,
  `context:${space}${context_value}`,
  `goal:${space}${goal}`,
  `project:${project_value_link}`,
  `parent_task:${parent_task_value_link}`,
  `organization:${organization_value_link}`,
  `contact:${contact_value_link}`,
  `library:${null_yaml_li}`,
  `type:${space}${type_value}`,
  `file_class:${space}${file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  `tags:`,
  hr_line,
].join(new_line);

//-------------------------------------------------------------------
// WEEKLY PREVIEW AND REVIEW OBJECT ARRAY
//-------------------------------------------------------------------
const review_preview_obj_arr = [
  {
    object_array: tasks_obj_arr,
    key: "Tasks and Events",
    subfile: "Tasks and Events",
    value: "task_event",
  },
  {
    object_array: habit_ritual_obj_arr,
    key: "Habits and Rituals",
    subfile: "Habits and Rituals",
    value: "habit_ritual",
  },
];

//-------------------------------------------------------------------
// WEEKLY REVIEW AND PREVIEW FILES CALLOUT WITH LINKS
//-------------------------------------------------------------------
heading = "Weekly Review and Preview Files";
const callout_subfile_link =
  [
    `${call_start}[!subfile]${space}${heading}`,
    call_start,
    call_tbl_start +
      review_preview_obj_arr
        .map(
          (x) =>
            `[[${week_preview_value}_week_${x.value}_review_preview\\|${x.subfile}]]`
        )
        .join(tbl_pipe) +
      call_tbl_end,
    call_tbl_div(5),
  ].join(new_line) + two_new_line;

//-------------------------------------------------------------------
// WEEKLY REVIEW AND PREVIEW FILES
//-------------------------------------------------------------------
let file_content;
for (let i = 0; i < review_preview_obj_arr.length; i++) {
  const review_preview_name = review_preview_obj_arr[i].key;
  const review_preview_value = review_preview_obj_arr[i].value;

  const title = `Weekly${space}${review_preview_name}${space}Review${space}and${space}Preview`;
  const title_value = `week_${review_preview_value}_review_preview`;

  const full_title_name = `${week_preview_name} ${title}`;
  const short_title_name = `${title.toLowerCase()}`;
  const short_title_value = title_value;
  const full_title_value = `${week_preview_value}_${title_value}`;

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

  // SET DATE AND TIME
  const datetime_str_arg = review_preview_name + " Review and Preview ";
  let time;
  if (i == 0) {
    review_preview_obj_arr[i].date = await tp.user.nl_date(
      tp,
      `${datetime_str_arg}Date?`
    );
    time = await tp.user.nl_time(tp, `${datetime_str_arg}Start${space}Time?`);
  } else {
    last_date_time_str = "last datetime: ";
    last_date = review_preview_obj_arr[i - 1].date;
    last_time = review_preview_obj_arr[i - 1].time;
    last_datetime = `${space}(${last_date_time_str}${last_date}T${last_time})`;
    review_preview_obj_arr[i].date = await tp.user.nl_date(
      tp,
      `${datetime_str_arg}Date?${last_datetime}`
    );
    time = await tp.user.nl_time(
      tp,
      `${datetime_str_arg}Start${space}Time?${last_datetime}`
    );
  }
  const date = review_preview_obj_arr[i].date;
  const date_link = `"[[${date}]]"`;
  const inline_due_date = `${space}📅${space}${date}`;

  const full_date_time = moment(`${date}T${time}`);
  // Heading, task checkboxes, and subtask callouts
  const content_obj_arr = review_preview_obj_arr[i].object_array;
  for (let j = 0; j < content_obj_arr.length; j++) {
    const head_task_title = head_lvl(3) + content_obj_arr[j].key;
    if (j == 0) {
      content_obj_arr[j].start = moment(full_date_time).format("HH:mm");
      content_obj_arr[j].end = moment(full_date_time)
        .add(content_obj_arr[j].duration, "minutes")
        .format("HH:mm");
    } else {
      prev_date_time = moment(`${date}T${content_obj_arr[j - 1].end}`).add(
        1,
        "minutes"
      );
      content_obj_arr[j].start = moment(prev_date_time).format("HH:mm");
      content_obj_arr[j].end = moment(prev_date_time)
        .add(content_obj_arr[j].duration, "minutes")
        .format("HH:mm");
    }

    const start_time = content_obj_arr[j].start;
    const inline_time_start = `[time_start${dv_colon}${start_time}]`;
    const end_time = content_obj_arr[j].end;
    const inline_time_end = `[time_end${dv_colon}${end_time}]`;
    const duration_est = content_obj_arr[j].duration;
    const inline_duration_est = `[duration_est${dv_colon}${duration_est}]`;
    const inline_metadata_time = `${space}${inline_time_start}${two_space}${inline_time_end}${two_space}${inline_duration_est}`;

    let inline_metadata_date = `${inline_creation_date}${inline_due_date}`;
    if (content_obj_arr[j].remind) {
      if (j == 0) {
        content_obj_arr[j].remind = moment(full_date_time)
          .subtract(content_obj_arr[j].remind, "minutes")
          .format("YYYY-MM-DD HH:mm");
      } else {
        content_obj_arr[j].remind = moment(prev_date_time)
          .subtract(content_obj_arr[j].remind, "minutes")
          .format("YYYY-MM-DD HH:mm");
      }
      const inline_reminder_date = `${space}⏰${space}${content_obj_arr[j].remind}`;
      inline_metadata_date = `${inline_reminder_date}${inline_metadata_date}`;
    }
    const task_title = `${content_obj_arr[j].key}${space}${content_obj_arr[j].type}`;
    const task_checkbox = `${checkbox_task_tag}${task_title}_${type_value}${inline_metadata_time}${inline_metadata_date}`;
    const task_subtasks = content_obj_arr[j].content;
    const task_subsection =
      [head_task_title, task_checkbox, task_subtasks].join(two_new_line) +
      two_new_line;
    if (j == 0) {
      section_obj_arr[0].content = task_subsection;
    } else {
      section_obj_arr[0].content += task_subsection;
    }
  }
  // Populate tasks and events content value
  section_obj_arr[0].content += hr_line + new_line;

  // Assign the max end time to review_preview_obj_arr time
  const file_datetime_arr = content_obj_arr.map((t) =>
    moment(`${date}T${t.end}`)
  );
  review_preview_obj_arr[i].time = moment
    .max(file_datetime_arr)
    .format("HH:mm");

  const toc_file = toc.replaceAll("_file_section_", file_section);

  const sections_content = section_obj_arr
    .map((s) => `${s.head}${toc_file}${s.content}`)
    .join(new_line);

  const frontmatter = [
    hr_line,
    `title:${space}${file_name}`,
    `uuid:${space}${await tp.user.uuid()}`,
    `aliases:${space}${file_alias}`,
    `date:${space}${date_link}`,
    yaml_bottom,
  ].join(new_line);

  const file_content = [
    frontmatter,
    head_lvl(1) + title + new_line,
    callout_subfile_link,
    info,
    i == 0 ? reflection_callout + sections_content : sections_content,
  ].join(new_line);

  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(parent_task_dir)
  );
}
%>