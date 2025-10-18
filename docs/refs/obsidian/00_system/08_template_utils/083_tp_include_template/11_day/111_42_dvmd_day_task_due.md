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
// DATAVIEW API AND FUNCTIONS FOR TASK COUNTERS
//-------------------------------------------------------------------
const dv = this.app.plugins.plugins["dataview"].api;
const task_pages = dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks;
const duration = dv.luxon.Duration;
const datetime = dv.luxon.DateTime;
const regex_task_name = /#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_|workday_|evening_).+/g;

function task_name_clean(task) {
  const name = task.text.replace(regex_task_name, "$1");
  return name;
}

function status_date_filter(task, status_arg) {
  let date_filter = dv.equal(
    datetime.fromISO(task.due).toFormat("yyyy-MM-dd"),
    date
  );
  let filter = date_filter && (task.status != "-" || task.status != "<");
  if (status_arg == "cancel") {
    filter = date_filter && task.status == "<";
  }
  return filter;
}

function task_type_filter(task, type_arg) {
  const tag_filter = task.text.includes("#task");
  const task_hab_rit_filter =
    task.text.includes("_habit") || task.text.includes("_ritual");
  const action_filter = task.text.includes("_action_item");

  let type_filter;
  if (
    type_arg == "all_type" ||
    type_arg == "all" ||
    type_arg == "total" ||
    type_arg == "" ||
    type_arg == null
  ) {
    type_filter = tag_filter;
  } else if (type_arg == "task_event") {
    type_filter = tag_filter && !task_hab_rit_filter;
  } else if (["hab_rit", "habit_rit"].includes(type_arg)) {
    type_filter = tag_filter && task_hab_rit_filter;
  } else if (type_arg == "action") {
    type_filter = tag_filter && action_filter;
  } else if (type_arg == "event") {
    type_filter = tag_filter && !(action_filter || task_hab_rit_filter);
  } else {
    type_filter = tag_filter && task.text.includes(`_${type_arg}`);
  }
  return type_filter;
}

function task_count(status_arg, type_arg) {
  const count_pages = task_pages
    .filter(
      (task) =>
        task_type_filter(task, type_arg) && status_date_filter(task, status_arg)
    )
    .map((task) => task_name_clean(task));
  const count = [...new Set(count_pages)].length;
  return `[${status_arg}_${type_arg}${dv_colon}${count}]`;
}

const totals_arr = ["total", "task_event", "habit_rit"];
const action_event_arr = ["action", "event"];
const hab_rit_arr = [
  "habit",
  "morning_rit",
  "startup_rit",
  "shutdown_rit",
  "evening_rit",
];

const status_count_list = (arr_arg, status_arg) =>
  ul +
  arr_arg
    .map((x) =>
      !status_arg
        ? [task_count("due", x), task_count("cancel", x)].join(tbl_pipe)
        : task_count(status_arg, x)
    )
    .join(tbl_pipe);

const due_cancel_count = [
  status_count_list(totals_arr),
  status_count_list(action_event_arr),
  status_count_list(hab_rit_arr, "due"),
  status_count_list(hab_rit_arr, "cancel"),
].join(new_line);

//-------------------------------------------------------------------
// DATAVIEW INLINE VARIABLES
//-------------------------------------------------------------------
const dv_yaml = "file.frontmatter";
const dv_current_yaml = `dv.current().${dv_yaml}.`;
const dv_luxon_iso = "dv.luxon.DateTime.fromISO";

const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

//-------------------------------------------------------------------
// DAILY TASKS AND EVENTS OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const obj_arr = [
  {
    head_key: "Planned for Today",
    toc_key: "Plan",
    status: null,
    md: null,
    content: null,
  },
  {
    head_key: "Due Today",
    toc_key: "Due",
    status: "preview",
    md: "true",
    content: due_cancel_count,
  },
  {
    head_key: "Completed Today",
    toc_key: "Done",
    status: "review",
    md: null,
    content: null,
    file: "111_43_dvmd_day_task_done",
  },
  {
    head_key: "Created Today",
    toc_key: "New",
    status: null,
    md: null,
    file: "111_44_dvmd_day_task_new",
  },
];

const toc_title = [`${call_start}[!toc]`, "Day", name, dv_content_link].join(
  space
);
const toc_body =
  call_tbl_start +
  obj_arr
    .map((x) => `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc = [toc_title, call_start, toc_body, call_tbl_div(4)].join(new_line);

obj_arr.map((x) => (x.head = head_lvl(3) + x.head_key));

for (let i = 0; i < obj_arr.length; i++) {
  if (!obj_arr[i].md) {
    continue;
  }
  obj_arr[i].query = await tp.user.dv_task_type_status_dates({
    type: "child",
    status: obj_arr[i].status,
    start_date: date,
    end_date: "",
    md: obj_arr[i].md,
  });
}

obj_arr.map(
  (x) =>
    (x.body =
      x.toc_key == "Due"
        ? [x.content, x.query].join(two_new_line)
        : [x.head, toc, temp_include(x.file)].join(two_new_line))
);

const day_task_event =
  new_line +
  obj_arr
    .filter((x) => x.toc_key != "Plan")
    .map((x) => x.body)
    .join(two_new_line);

tR += day_task_event;
%>