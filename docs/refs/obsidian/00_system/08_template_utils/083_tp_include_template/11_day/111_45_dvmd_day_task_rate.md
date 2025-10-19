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

//-------------------------------------------------------------------
// RETRIEVE CURRENTLY ACTIVE FILE METADATA CACHE
//-------------------------------------------------------------------
const current_file = this.app.workspace.getActiveFile();
const current_file_name = current_file.name;
const current_file_path = await app.vault
  .getMarkdownFiles()
  .filter((file) => file.path.includes(current_file_name))
  .map((file) => file.path);
const current_file_path_wo_ext = current_file_path.toString().replace("\.md$", "");
const abstract_file = await app.vault.getAbstractFileByPath(current_file_path);
const file_cache = await app.metadataCache.getFileCache(abstract_file);
const date = file_cache?.frontmatter?.date;
const day_name = file_cache?.frontmatter?.weekday_name;

//-------------------------------------------------------------------
// DATAVIEW API AND FUNCTIONS FOR TASK COUNTERS
//-------------------------------------------------------------------
const dv = this.app.plugins.plugins["dataview"].api;
const day_task_page_arr = dv.pages(`"${current_file_path_wo_ext}"`);
const day_task_page_weekday = day_task_page_arr.map(
  (f) => f.file.frontmatter.weekday_name
)[0];

//-------------------------------------------------------------------
// DAILY TASKS AND EVENTS DAILY RATE
//-------------------------------------------------------------------
const inline_field_arr = [
  "plan",
  "cancel",
  "due",
  "done",
  "discard",
  "reschedule",
];
const task_hab_rit_total_obj_arr = [
  { key: "total", value: "🌀Totals" },
  { key: "task_event", value: "⚒️Tasks and Events" },
  { key: "habit_rit", value: "🤖Habits and Rituals" },
];
const task_event_obj_arr = [
  { key: "action", value: "🔨Action Items" },
  { key: "event", value: "🤝Events" },
];
const habit_rit_obj_arr = [
  { key: "habit", value: "🦿Habits" },
  { key: "morning_rit", value: "🍵Morning" },
  { key: "startup_rit", value: "🌇Startup" },
  { key: "shutdown_rit", value: "🌆Shutdown" },
  { key: "evening_rit", value: "🛌Evening" },
];
const head_row = [
  day_task_page_weekday,
  "🧩Plan",
  "⏹️Cancelled",
  "📆Due",
  "✅Done",
  "❌Discard",
  "⛔Cancel %",
  "✔️Comp %",
  "🗑️Disc %",
];

const field_key_str = (key_str) =>
  inline_field_arr.map((field) => field + "_" + key_str);
const day_inline_data = (field) => day_task_page_arr.map((file) => file[field]);
const inline_key = (type_obj_arr) =>
  type_obj_arr.map((x) => field_key_str(x.key)).map((x) => day_inline_data(x));
function rate(numerator, denominator) {
  const percent = (round_dig) =>
    ((numerator / denominator) * 100).toFixed(round_dig);
  const percent_str = (round_dig) => percent(round_dig).toString();
  let percent_rate;
  if (!percent_str(2).endsWith("0")) {
    percent_rate = percent(2);
  } else if (!percent_str(1).endsWith("0")) {
    percent_rate = percent(1);
  } else {
    percent_rate = percent(0);
  }
  if (denominator == 0) {
    percent_rate = 0;
  };
  return percent_rate;
}

function data_rows(type_obj_arr) {
  let data_row_arr = [];
  type_obj_arr.forEach((type_obj) => {
    const index_cell = type_obj.value;
    const inline_key_str_arr = field_key_str(type_obj.key);
    const day_inline_data_arr = inline_key_str_arr.flatMap((inline_key) =>
      day_inline_data(inline_key)
    );
    let data_cells = day_inline_data_arr.map(
      (inline_data) => inline_data[0] * 1
    );
    const plan = data_cells[0];
    const cancel = data_cells[1];
    const due = data_cells[2];
    const done = data_cells[3];
    let discard = data_cells[5] ? data_cells[4] + data_cells[5] : data_cells[4];
    data_cells.pop();
    const cancel_rate = rate(cancel, plan);
    const comp_rate = rate(done, due);
    const discard_rate = rate(discard, due);
    const data_row = [
      index_cell,
      plan,
      cancel,
      due,
      done,
      discard,
      `${cancel_rate}%`,
      `${comp_rate}%`,
      `${discard_rate}%`,
    ];
    data_row_arr.push(data_row);
  });
  return data_row_arr;
}

const table = dv.markdownTable(
  head_row,
  [data_rows(task_event_obj_arr), data_rows(habit_rit_obj_arr)].flat()
);

tR += table;
%>
