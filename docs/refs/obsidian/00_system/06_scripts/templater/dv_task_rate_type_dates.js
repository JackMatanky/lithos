// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> YAML FRONTMATTER FIELDS <<<<<
//-------------------------------------------------------------------
// File name
const file_name = "file.name";

// YAML title
const yaml_title = "file.frontmatter.title";

// YAML alias
const yaml_alias = "file.frontmatter.aliases[0]";

// YAML file class
const yaml_class = "file.frontmatter.file_class";

// YAML type
const yaml_type = "file.frontmatter.type";

// YAML calendar start date
const yaml_date_start = "file.frontmatter.date_start";

// YAML calendar end date
const yaml_date_end = "file.frontmatter.date_end";

// YAML calendar day date
const yaml_date = "file.frontmatter.date";

// YAML weekday name
const yaml_weekday_name = "file.frontmatter.weekday_name";

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELD FUNCTIONS <<<<<
//-------------------------------------------------------------------
// Title link
const weekday_title_link = `link(${file_name}, ${yaml_weekday_name})`;
const day_title_link = "file.link";

// Title link for DV markdown query
const md_weekday_title_link = `"[[" + ${file_name} + "\|" + ${yaml_weekday_name} + "]]"`;
const md_day_title_link = `"[[" + file.frontmatter.date + "]]"`;

const type_obj_arr = [
  { key: "total", value: "🌀Totals" },
  { key: "task_event", value: "⚒️Tasks and Events" },
  { key: "habit_rit", value: "🤖Date" },
  { key: "habit", value: "🦿Habits" },
  { key: "morning_rit", value: "🍵Morning" },
  { key: "startup_rit", value: "🌇Startup" },
  { key: "shutdown_rit", value: "🌆Shutdown" },
  { key: "evening_rit", value: "🛌Evening" },
];

const count_rate_obj_arr = [
  { key: "plan", value: "🧩Plan" },
  { key: "cancel", value: "⏹️Cancelled" },
  { key: "due", value: "📆Due" },
  { key: "done", value: "✅Done" },
  { key: "discard", value: "❌Discard" },
];

const day_arr = [
  "sunday",
  "monday",
  "tuesday",
  "wednesday",
  "thursday",
  "friday",
  "saturday",
];

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//-------------------------------------------------------------------
// Projects directory
const cal_dir = `"10_calendar"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//-------------------------------------------------------------------
// SECT: >>>>> GENERAL FIELDS <<<<<
// Calendar file class filter
const class_filter = `contains(${yaml_class}, "cal_day")`;

// Calendar task and event type filter
const type_filter = `contains(${yaml_type}, "task_event")`;

//-------------------------------------------------------------------
// DATAVIEW TABLE FOR TASKS AND EVENTS
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR: TYPES: "day", "total", "task_event", "habit_rit", "action", "event", "habit", "morning_rit", "startup_rit", "shutdown_rit", "evening_rit"

async function dv_task_rate_type_dates({
  type: type,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  const type_str = String(type).toLowerCase();
  let type_title;
  if (day_arr.includes(type_str)) {
    type_title = moment(date_start).day(type_str).format("[⚒️]dddd");
  } else {
    type_title = type_obj_arr
    .filter((x) => x.key == type)
    .map((x) => x.value);
  }

  // TABLE DATA FIELDS
  let title_field;
  if (day_arr.includes(type_str)) {
    title_field = day_title_link;
  } else {
    title_field = weekday_title_link;
  }
  const data_field = `${title_field} AS "${type_title}",
    plan AS "🧩Plan",
    cancel AS "⏹️Cancelled",
    due AS "📆Due",
    done AS "✅Done",
    discard AS "❌Discard",
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"`;

  // FILTER
  let date_filter;
  if (day_arr.includes(type_str)) {
    date_filter = `date(${yaml_date}) = date(${date_start})`;
  } else {
    date_filter = `date(${yaml_date}) >= date(${date_start})
    AND date(${yaml_date}) <= date(${date_end})`;
  }
  const filter = `${class_filter}
    AND ${type_filter}
    AND ${date_filter}`;

  // FLATTEN
  let type_arg;
  if (day_arr.includes(type_str)) {
    type_arg = "task_event";
  } else {
    type_arg = type;
  }
  const flat_plan = `plan_${type_arg} AS plan`;
  const flat_cancel = `cancel_${type_arg} AS cancel`;
  const flat_due = `due_${type_arg} AS due`;
  const flat_done = `done_${type_arg} AS done`;
  let flat_discard;
  if (type_arg == "total" || type_arg == "task_event") {
    flat_discard = `discard_${type_arg} + reschedule_action + reschedule_event AS discard`;
  } else {
    flat_discard = `discard_${type_arg} AS discard`;
  }

  let dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${cal_dir}
FLATTEN
    ${flat_plan}
FLATTEN
    ${flat_cancel}
FLATTEN
    ${flat_due}
FLATTEN
    ${flat_done}
FLATTEN
    ${flat_discard}
WHERE
    ${filter}
SORT
    ${yaml_date} ASC
${three_backtick}`;

  if (md == "true" || md == true) {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    let md_query = String(
      dataview_query
        .replace(dataview_block_start_regex, "")
        .replace(dataview_block_end_regex, "")
        .replaceAll(/\n\s+/g, " ")
        .replaceAll(/\n/g, " ")
        .replace(weekday_title_link, md_weekday_title_link)
    );
    if (day_arr.includes(type_str)) {
      md_query = String(
        dataview_query
          .replace(dataview_block_start_regex, "")
          .replace(dataview_block_end_regex, "")
          .replaceAll(/\n\s+/g, " ")
          .replaceAll(/\n/g, " ")
          .replace(day_title_link, md_day_title_link)
      );
    }

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
    // dataview_query = md_query;
  }

  return dataview_query;
}

module.exports = dv_task_rate_type_dates;

// Original queries and inline data in Obsidian

// week
// if (date_end != "") {
//   dataview_query = `${dataview_block}
// TABLE WITHOUT ID
//   `"[[" + file.frontmatter.date + "]]" AS ${title_arg}`,
//   count_rate_obj_arr.map((x) => `${x.key} AS ${x.value}`).join(", "),
//   `string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %"`,
//   `string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"`
// FROM
//   "10_calendar"
// count_rate_obj_arr
//   .map((x) => `FLATTEN ${x.key}_${type_arg} AS ${x.key}`)
//   .join(" "),
// WHERE
//   contains(file.frontmatter.file_class, "cal_day")`,
//   `AND date(file.frontmatter.date) >= date(${date_start})`,
//   `AND date(file.frontmatter.date) <= date(${date_end})`,
// SORT
//   file.frontmatter.date ASC
// ${three_backtick}`;
// } else {
// day
//   dataview_query = `${dataview_block}
// TABLE WITHOUT ID
//   `"[[" + file.frontmatter.date + "]]" AS ${title_arg}`,
//   count_rate_obj_arr.map((x) => `${x.key} AS ${x.value}`).join(", "),
//   `string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %"`,
//   `string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"`
// FROM
//   "10_calendar"
// count_rate_obj_arr
//   .map((x) => `FLATTEN ${x.key}_${type_arg} AS ${x.key}`)
//   .join(" "),
// WHERE
//   contains(file.frontmatter.file_class, "cal_day")`,
//   `AND date(file.frontmatter.date) = date(${date_start})`,
// ${three_backtick}`;
// }

// plan_total
// plan_task_event
// plan_habit_rit
// plan_action
// plan_event
// plan_habit
// plan_morning_rit
// plan_startup_rit
// plan_shutdown_rit
// plan_evening_rit

// due_total
// due_task_event
// due_habit_rit
// due_action
// due_event
// due_habit
// due_morning_rit
// due_startup_rit
// due_shutdown_rit
// due_evening_rit

// cancel_total
// cancel_task_event
// cancel_habit_rit
// cancel_action
// cancel_event
// cancel_habit
// cancel_morning_rit
// cancel_startup_rit
// cancel_shutdown_rit
// cancel_evening_rit

// done_total
// done_task_event
// done_habit_rit
// done_action
// done_event
// done_habit
// done_morning_rit
// done_startup_rit
// done_shutdown_rit
// done_evening_rit

// discard_total
// discard_task_event
// discard_habit_rit
// discard_action
// discard_event
// discard_habit
// discard_morning_rit
// discard_startup_rit
// discard_shutdown_rit
// discard_evening_rit

// reschedule_action
// reschedule_event
