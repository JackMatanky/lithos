---
title: side_panel_tasks_week
aliases:
  - Side Panel Tasks Week
  - side panel tasks week
  - Side Panel Weekly Tasks
  - side panel weekly tasks
  - Dataview Weekly Tasks
  - side_panel_tasks_week
cssclasses:
  - inline_title_hide
  - side_panel_narrow
  - read_narrow_margin
  - paragraph_narrow
  - font_size_side_panel
  - metadata_icon_remove
file_class: task
date_created: 2023-09-03T19:26
date_modified: 2023-09-27050T3419:17
---
```dataviewjs
/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
const new_line = String.fromCodePoint(0xa);
const space = String.fromCodePoint(0x20);

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const duration = dv.luxon.Duration;
const datetime = dv.luxon.DateTime;
const week = moment().format("YYYY-[W]ww");
const moment_day = (int) =>
  moment().startOf("week").day(int).format("YYYY-MM-DD");
const moment_day_name = (int) =>
  moment().startOf("week").day(int).format("dddd");
const date_link = (date) => dv.fileLink(`${date}_task_event`, false, date);
const date_link_day = (date) =>
  dv.fileLink(
    `${date}_task_event`,
    false,
    moment(date).format("dddd, MMMM D, YYYY ")
  );

//const today = datetime.now().toFormat("yyyy-MM-dd");
//const yesterday = datetime.now().minus({days: 2}).toFormat("yyyy-MM-dd");

//-------------------------------------------------------------------
// REGEX VARIABLES
//-------------------------------------------------------------------
const regex_task_name =
  /#task\s(.+)_((action_|(phone|video)_call|interview|appointment|lecture|tutorial|event|hangout|habit|(meet|gather)ing|(morn|even)ing_|workday_).+?)\[.+/g;
const regex_date_due = /#task.+📅\s*([\d\-]+).+/g;
const regex_time_start = /#task.+time_start::\s*([\d:]+).+/g;
const regex_time_end = /#task.+time_end::\s*([\d:]+).+/g;
const regex_task_file_link = /.+\/(.+)\s>.+\]\]$/g;
const regex_task_section_link = /.+\/.+\s>(.+)\]\]$/g;

//-------------------------------------------------------------------
// OBJECT ARRAYS
//-------------------------------------------------------------------
const type_obj_arr = [
  { key: "act", value: "🔨Action" },
  { key: "meet", value: "🤝Meeting" },
  { key: "video", value: "📹Call" },
  { key: "phone", value: "📞Call" },
  { key: "int", value: "💼Interview" },
  { key: "app", value: "⚕️Appointment" },
  { key: "lecture", value: "🧑‍🏫Lecture" },
  { key: "event", value: "🎊Event" },
  { key: "gath", value: "✉️Gathering" },
  { key: "hang", value: "🍻Hangout" },
  { key: "habit", value: "🦿Habit" },
  { key: "morn", value: "🍵Rit." },
  { key: "day_start", value: "🌇Rit." },
  { key: "day_shut", value: "🌆Rit." },
  { key: "eve", value: "🛌Rit." },
];

const status_obj_arr = [
  { key: " ", value: "🔜" },
  { key: "x", value: "✔️" },
  { key: "<", value: "⏹️" },
  { key: "-", value: "❌" },
];

const project_path_arr = [
  `"41_personal"`,
  `"42_education"`,
  `"43_professional"`,
  `"44_work"`,
  `"45_habit_ritual"`,
];

const general_filter_arr = ["all", "", null];

//-------------------------------------------------------------------
// DATA FETCHING
//-------------------------------------------------------------------
const task_pages = dv.pages(project_path_arr.join(" OR ")).file.tasks;

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
const get_task_info = (task, info) =>
  info !== "duration"
    ? task.text.replace(regex_patterns[info], "$1")
    : Number(task.text.replace(regex_patterns.duration, "$1"));

const get_task_datetime = (task, time) =>
  DateTime.fromISO(
    `${get_task_info(task, "date_due")}T${get_task_info(task, time)}`
  );

const get_task_duration_act = (task) =>
  get_task_datetime(task, "time_end")
    .diff(get_task_datetime(task, "time_start"))
    .as("minutes");

const get_task_duration_str = (task) =>
  [
    "⏳" + get_task_info(task, "time_start"),
    "→",
    get_task_info(task, "time_end") + "⌛",
    ":",
    "⏲️" + get_task_duration_act(task),
  ].join(" ");

const get_file_name = (task) => {
  const file_path = task.path.split("/");
  return file_path[file_path.length - 1].replace(".md", "");
};

const get_file_section = (task) =>
  task.link.toString().replace(regex_patterns.task_section_link, "$1");

const get_task_link = (task) =>
  dv.sectionLink(
    get_file_name(task),
    get_file_section(task),
    false,
    get_task_info(task, "task_name")
  );

const task_obj = (task) => ({
  name: get_task_info(task, "task_name"),
  link: get_task_link(task),
  start_datetime: get_task_datetime(task, "time_start"),
  end_datetime: get_task_datetime(task, "time_end"),
  duration_est: get_task_info(task, "duration"),
  duration_act: get_task_duration_act(task, "duration"),
  text: task.text,
  status: task.status,
  path: task.path,
});

//-------------------------------------------------------------------
// FUNCTIONS FOR TASK COUNT AND LIST QUERIES
//-------------------------------------------------------------------
function name_clean(task) {
  const name = task.text.replace(regex_task_name, "$1");
  return name;
}

function task_link(task) {
  const name = task.text.replace(regex_task_name, "$1");
  const status = status_obj_arr
    .filter((x) => task.status == x.key)
    .map((x) => x.value);

  const file = task.link
    .toString()
    .replace(regex_task_file_link, "$1")
    .replace(/(.+)\.md.+/g, "$1");
  const section = task.link.toString().replace(regex_task_section_link, "$1");

  return dv.sectionLink(file, section, false, name);
}

function task_time(task) {
  const start = task.text.replace(regex_time_start, "$1");
  const end = task.text.replace(regex_time_end, "$1");
  const time = `⏳${start} → ${end}⌛`;
  
  return time;
}

function task_status(task) {
  const name = task.text.replace(regex_task_name, "$1");
  const status = status_obj_arr
    .filter((x) => task.status == x.key)
    .map((x) => x.value);
  
  return status;
}

function time_status(task){
  return `${task_time(task)} (${task_status(task)})`
}

function task_details(task) {
  const task_type = task.text.replace(regex_task_name, "$2");
  const type = type_obj_arr
    .filter((x) => task_type.includes(x.key))
    .map((x) => x.value);
  const start = task.text.replace(regex_time_start, "$1");
  const end = task.text.replace(regex_time_end, "$1");
  const time = `⏳${start} → ${end}⌛`;
  const type_status = `${type}: ${time}`;

  return [task.path].map((x) =>
    dv
      .pages(`"${x}"`)
      .file.frontmatter.flatMap((x) => [x.parent_task, x.project].flat())
  );
}

function filter_date(task, day_int) {
  return (
    task.text.includes("#task") &&
    dv.equal(
      datetime.fromISO(task.due).toFormat("yyyy-MM-dd"),
      moment_day(day_int)
    )
  );
}

function task_time_sort(task) {
  const start = task.text.replace(regex_time_start, "$1");
  const time_start = datetime.fromFormat(start, "HH:mm").toFormat("HH:mm");
  return time_start;
}

function datetime_sort(task) {
  const date = task.text.replace(regex_date_due, "$1");
  const time = task.text.replace(regex_time_start, "$1");
  const date_time = datetime.fromFormat(`${date} ${time}`, "yyyy-MM-dd HH:mm");
  
  return date_time;
}

function count(day_int) {
  const count_pages = task_pages
    .filter((task) => filter_date(task, day_int))
    .map((task) => name_clean(task));
  return [new Set(count_pages)].length;
}

function task_list_nested(date_arg) {
  const list_root = dv.el("ul", "");
  const day_task_sort = task_pages
    .filter(
      (task) =>
        task.text.includes("#task") &&
        dv.equal(datetime.fromISO(task.due).toFormat("yyyy-MM-dd"), date_arg)
    )
    .sort((task) => task_time_sort(task));
  const task_list = day_task_sort.forEach((task) => {
    dv.el("li", task_link(task), { container: list_root });
    const task_root = dv.el("ul", "", { container: list_root });
    const project =
      [task.path]
        .map((x) => dv.pages(`"${x}"`).file.frontmatter.project)
        .toString()
        .replaceAll(/\[\[.+\|(.+)(\]){3}/g, "🏗️: $1");
    const parent =
      [task.path]
        .map((x) => dv.pages(`"${x}"`).file.frontmatter.parent_task)
        .toString()
        .replaceAll(/\[\[.+\|(.+)(\]){3}/g, "🧰: $1");
    dv.el("li", time_status(task), { container: task_root });
    dv.el("li", project, { container: task_root });
    dv.el("li", parent, { container: task_root });
  });
  return task_list;
}

function day_list(day_int) {
  const day = moment_day(day_int);
  const head = dv.header(2, date_link_day(day));
  let output;
  if (count(day_int)) {
    output = task_list_nested(day);
  } else {
    output = dv.paragraph("No tasks scheduled");
  }
  return head + output;
}

//-------------------------------------------------------------------
// LISTS OF TASK LINKS OUTPUT
//-------------------------------------------------------------------
dv.header(1, date_link(week));
day_list(0);
day_list(1);
day_list(2);
day_list(3);
day_list(4);
day_list(5);
day_list(6);
```
