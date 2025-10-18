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

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const date_start = file_cache?.frontmatter?.date_start;
const date_end = file_cache?.frontmatter?.date_end;
const long_date = moment(date_start).format("[Week ]ww[,] YYYY");
const short_date = moment(date_start).format("YYYY-[W]ww");

//-------------------------------------------------------------------
// WEEK HABITS AND RITUALS SUBFILE DETAILS
//-------------------------------------------------------------------
const full_name = "Habits and Rituals";
const name = "Habits and Rituals";
const value = "habit_ritual";

const file_name = `${short_date}_${value}`;
const file_section = file_name + hash;

//-------------------------------------------------------------------
// WEEKLY HABITS AND RITUALS OBJECT ARRAY
//-------------------------------------------------------------------
const file_obj_arr = [
  {
    head_key: "Habits",
    toc_key: "Due",
    type: "habit",
    status: "due",
    include_tbl: "112_45_dvmd_wk_habit_done",
    include_rate: "112_45_dvmd_wk_habit_rate",
  },
  {
    head_key: "Morning Rituals",
    toc_key: "Due",
    type: "morning_ritual",
    status: "due",
    include_tbl: "112_46_dvmd_wk_morn_rit_done",
    include_rate: "112_46_dvmd_wk_morn_rit_rate",
  },
  {
    head_key: "Workday Startup Rituals",
    toc_key: "Due",
    type: "workday_startup_ritual",
    status: "due",
    include_tbl: "112_47_dvmd_wk_start_rit_done",
    include_rate: "112_47_dvmd_wk_start_rit_rate",
  },
  {
    head_key: "Workday Shutdown Rituals",
    toc_key: "Due",
    type: "workday_shutdown_ritual",
    status: "due",
    include_tbl: "112_48_dvmd_wk_shut_rit_done",
    include_rate: "112_48_dvmd_wk_shut_rit_rate",
  },
  {
    head_key: "Evening Rituals",
    toc_key: "Due",
    type: "evening_ritual",
    status: "due",
    include_tbl: "112_49_dvmd_wk_eve_rit_done",
    include_rate: "112_49_dvmd_wk_eve_rit_rate",
  },
];

const due_suffix = " Due This Week";
const done_suffix = " Completed This Week";
const regex_rit = /\sRituals$/g;

file_obj_arr.map((x) => (x.head = head_lvl(3) + x.head_key));
file_obj_arr.map(
  (x) => (x.due = head_lvl(4) + x.head_key + due_suffix)
);
file_obj_arr.map(
  (x) => (x.done = head_lvl(4) + x.head_key + done_suffix)
);

// WEEK HABITS AND RITUALS SUBFILE TABLE OF CONTENTS
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;
const toc_title = `${call_start}[!toc]${space}Week${space}${name}${space}${dv_content_link}`;

file_obj_arr.map(
  (x) =>
    (x.toc_type = x.head_key.match(regex_rit)
      ? x.head_key.replace(regex_rit, "")
      : x.head_key)
);

const toc_high =
  call_tbl_start +
  file_obj_arr
    .map((x) => `[[${file_section}${x.head_key}\\|${x.toc_type}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_low_lvl = (suffix, toc_alias) =>
  call_tbl_start +
  file_obj_arr
    .map((x) => `[[${file_section}${x.head_key}${suffix}\\|${toc_alias}]]`)
    .join(tbl_pipe) +
  call_tbl_end;

const toc = [
  toc_title,
  call_start,
  toc_high,
  call_tbl_div(5),
  toc_low_lvl(due_suffix, "Due"),
  toc_low_lvl(done_suffix, "Done"),
].join(new_line);

// WEEK PDEV SUBFILE DATAVIEW QUERIES
for (let i = 0; i < file_obj_arr.length; i++) {
  file_obj_arr[i].query = await tp.user.dv_task_type_status_dates({
    type: file_obj_arr[i].type,
    status: file_obj_arr[i].status,
    start_date: date_start,
    end_date: date_end,
    md: "true",
  });
}

const week_file =
  new_line +
  temp_include("112_45_dvmd_wk_hab_rit_rate") +
  two_new_line +
  file_obj_arr
    .map((x) =>
      [
        x.head,
        toc,
        temp_include(x.include_rate),
        x.due,
        x.query,
        x.done,
        temp_include(x.include_tbl),
      ].join(two_new_line)
    )
    .join(two_new_line);

tR += week_file;
%>