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
function temp_include(file) {
  const tp_start = [
    less_than + String.fromCodePoint(0x25) + "*",
    "tR",
    "+=",
    "await",
  ].join(space);
  const tp_func =
    ["tp", "user", "include_template"].join(".") + `(tp, "${file}")`;
  const tp_end = String.fromCodePoint(0x25) + great_than;
  return [tp_start, tp_func, tp_end].join(space);
}

//-------------------------------------------------------------------
// RETRIEVE CURRENTLY ACTIVE FILE METADATA CACHE
//-------------------------------------------------------------------
const current_file = this.app.workspace.getActiveFile();
const current_file_name = current_file.name;
const current_file_path = await app.vault
  .getMarkdownFiles()
  .filter((file) => file.path.includes(current_file_name))
  .map((file) => file.path);
const abstract_file = await app.vault.getAbstractFileByPath(current_file_path);
const file_cache = await app.metadataCache.getFileCache(abstract_file);
const date = file_cache?.frontmatter?.date;

//-------------------------------------------------------------------
// DAY TASKS AND EVENTS SUBFILE DETAILS
//-------------------------------------------------------------------
const name = "Tasks and Events";
const value = "_task_event";
const file_name = date + value;
const file_section = file_name + hash;

//-------------------------------------------------------------------
// DATAVIEW INLINE VARIABLES
//-------------------------------------------------------------------
const dv_yaml = "file.frontmatter";
const dv_current_yaml = `dv.current().${dv_yaml}.`;

const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

//-------------------------------------------------------------------
// DATAVIEW API AND FUNCTIONS FOR TASK COUNTERS
//-------------------------------------------------------------------
const dv = this.app.plugins.plugins["dataview"].api;
const task_pages = dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks;
const duration = dv.luxon.Duration;
const datetime = dv.luxon.DateTime;
const regex_task_name =
  /#task\s(.+)_(action_|(phone|video)_call|interview|appointment|lecture|event|hangout|habit|(meet|gather|morn|even)ing_|workday_).+/g;

function task_name_clean(task) {
  const name = task.text.replace(regex_task_name, "$1");
  return name;
}
function status_date_filter(task, status_arg) {
  let status_type = " ";
  let date_type = task.due;
  if (status_arg == "done") {
    date_type = task.completion;
    status_type = "x";
  } else if (status_arg == "discard") {
    status_type = "-";
  } else if (
    status_arg == "all_status" ||
    status_arg == "all" ||
    status_arg == "" ||
    status_arg == null ||
    status_arg == "reschedule"
  ) {
    status_type = null;
  }
  let date_filter = dv.equal(
    datetime.fromISO(date_type).toFormat("yyyy-MM-dd"),
    date
  );
  if (status_arg == "overdue") {
    date_filter = datetime.fromISO(date_type).toFormat("yyyy-MM-dd") < date;
  }

  if (status_type) {
    return date_filter && task.status == status_type;
  } else {
    return date_filter;
  }
}
function type_filter(task, type_arg) {
  const tag_filter = task.text.includes("#task");
  const task_hab_rit_filter =
    task.text.includes("_habit") || task.text.includes("_ritual");
  const action_filter = task.text.includes("_action_item");

  let filter;
  if (
    type_arg == "all_type" ||
    type_arg == "all" ||
    type_arg == "total" ||
    type_arg == "" ||
    type_arg == null
  ) {
    filter = tag_filter;
  } else if (type_arg == "task_event") {
    filter = tag_filter && !task_hab_rit_filter;
  } else if (type_arg == "hab_rit" || type_arg == "habit_rit") {
    filter = tag_filter && task_hab_rit_filter;
  } else if (type_arg == "action") {
    filter = tag_filter && action_filter;
  } else if (type_arg == "event") {
    filter = tag_filter && !(action_filter || task_hab_rit_filter);
  } else {
    filter = tag_filter && task.text.includes(`_${type_arg}`);
  }
  return filter;
}

function task_count(status_arg, type_arg) {
  const count = [
    ...new Set(
      task_pages
        .where(
          (task) =>
            type_filter(task, type_arg) && status_date_filter(task, status_arg)
        )
        .map((task) => task_name_clean(task))
    ),
  ].length;
  return status_arg + "_" + type_arg + dv_colon + count;
}

//-------------------------------------------------------------------
// TASK COUNTERS
//-------------------------------------------------------------------
const num_arr = ["0", "1", "2", "3", "4", "5", "6"];
const resched_act = Number(
  await tp.system.suggester(num_arr, num_arr, false, "Rescheduled Actions?")
);
const resched_event = Number(
  await tp.system.suggester(num_arr, num_arr, false, "Rescheduled Events?")
);

const task_hab_rit_total =
  `${ul}[` +
  [
    task_count("done", "total"),
    task_count("discard", "total"),
    task_count("done", "task_event"),
    task_count("discard", "task_event"),
    task_count("done", "habit_rit"),
    task_count("discard", "habit_rit"),
  ].join(`]${tbl_pipe}[`) +
  "]";

const done_task_event =
  `${ul}[` +
  [
    task_count("done", "action"),
    task_count("discard", "action"),
    `reschedule_action${dv_colon}${resched_act}`,
    task_count("done", "event"),
    task_count("discard", "event"),
    `reschedule_event${dv_colon}${resched_event}`,
  ].join(`]${tbl_pipe}[`) +
  "]";

const done_hab_rit =
  `${ul}[` +
  [
    task_count("done", "habit"),
    task_count("done", "morning_rit"),
    task_count("done", "startup_rit"),
    task_count("done", "shutdown_rit"),
    task_count("done", "evening_rit"),
  ].join(`]${tbl_pipe}[`) +
  "]";

const discard_hab_rit =
  `${ul}[` +
  [
    task_count("discard", "habit"),
    task_count("discard", "morning_rit"),
    task_count("discard", "startup_rit"),
    task_count("discard", "shutdown_rit"),
    task_count("discard", "evening_rit"),
  ].join(`]${tbl_pipe}[`) +
  "]";

const cnt_done = [
  task_hab_rit_total,
  done_task_event,
  done_hab_rit,
  discard_hab_rit,
].join(new_line);

//-------------------------------------------------------------------
// DAILY TASKS AND EVENTS OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const query = await tp.user.dv_task_type_status_dates({
  type: "child",
  status: "review",
  start_date: date,
  end_date: "",
  md: "true",
});

const day_task_event = new_line + [cnt_done, query].join(two_new_line);

tR += day_task_event;
%>