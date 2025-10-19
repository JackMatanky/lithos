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

const moment_day = (int) => moment(date_start).day(int).format("YYYY-MM-DD");
const weekday_arr = [0, 1, 2, 3, 4, 5, 6].map((x) => moment_day(x));

//-------------------------------------------------------------------
// WEEK TASKS AND EVENTS SUBFILE DETAILS
//-------------------------------------------------------------------
const full_name = "Tasks and Events";
const name = "Tasks and Events";
const value = "task_event";

const file_name = `${short_date}_${value}`;
const file_section = file_name + hash;

//-------------------------------------------------------------------
// WEEKLY COMPLETED TASKS AND EVENTS BUTTON
//-------------------------------------------------------------------
const button_comment = "Adjust replace lines";
const button = [
  three_backtick + "button",
  "name ✅Completed Tasks and Events",
  "type append template",
  "action 112_41_dvmd_week_tasks_done",
  "replace [100, 500]",
  "color blue",
  three_backtick,
  new_line,
  `${cmnt_html_start}${button_comment}${cmnt_html_end}`,
].join(new_line);

//-------------------------------------------------------------------
// WEEKLY TASKS AND EVENTS OBJECT ARRAY AND DATAVIEW QUERIES
//-------------------------------------------------------------------
const file_obj_arr = [
  {
    sect_level: 1,
    head_key: "Active Projects",
    toc_key: "Active",
    type: "project",
    status: "active",
    include_tbl: null,
  },
  {
    sect_level: 1,
    head_key: "Overdue Projects",
    toc_key: "Overdue",
    type: "project",
    status: "overdue",
    include_tbl: null,
  },
  {
    sect_level: 2,
    head_key: "Active Parent Tasks",
    toc_key: "Active",
    type: "parent",
    status: "active",
    include_tbl: null,
  },
  {
    sect_level: 2,
    head_key: "Overdue Parent Tasks",
    toc_key: "Overdue",
    type: "parent",
    status: "overdue",
    include_tbl: null,
  },
  {
    sect_level: 2,
    head_key: "Completed Parent Tasks",
    toc_key: "Completed",
    type: "parent",
    status: null,
    include_tbl: "112_42_dvmd_wk_par_task_done",
  },
  {
    sect_level: 3,
    head_key: "Sunday",
    toc_key: "Due",
    type: "child",
    status: "due",
    include_tbl: "112_431_dvmd_wk_task_sun_done",
    include_rate: "112_431_dvmd_wk_task_sun_rate",
  },
  {
    sect_level: 3,
    head_key: "Monday",
    toc_key: "Due",
    type: "child",
    status: "due",
    include_tbl: "112_432_dvmd_wk_task_mon_done",
    include_rate: "112_432_dvmd_wk_task_mon_rate",
  },
  {
    sect_level: 3,
    head_key: "Tuesday",
    toc_key: "Due",
    type: "child",
    status: "due",
    include_tbl: "112_433_dvmd_wk_task_tue_done",
    include_rate: "112_433_dvmd_wk_task_tue_rate",
  },
  {
    sect_level: 3,
    head_key: "Wednesday",
    toc_key: "Due",
    type: "child",
    status: "due",
    include_tbl: "112_434_dvmd_wk_task_wed_done",
    include_rate: "112_434_dvmd_wk_task_wed_rate",
  },
  {
    sect_level: 3,
    head_key: "Thursday",
    toc_key: "Due",
    type: "child",
    status: "due",
    include_tbl: "112_435_dvmd_wk_task_thu_done",
    include_rate: "112_435_dvmd_wk_task_thu_rate",
  },
  {
    sect_level: 3,
    head_key: "Friday",
    toc_key: "Due",
    type: "child",
    status: "due",
    include_tbl: "112_436_dvmd_wk_task_fri_done",
    include_rate: "112_436_dvmd_wk_task_fri_rate",
  },
  {
    sect_level: 3,
    head_key: "Saturday",
    toc_key: "Due",
    type: "child",
    status: "due",
    include_tbl: "112_437_dvmd_wk_task_sat_done",
    include_rate: "112_437_dvmd_wk_task_sat_rate",
  },
];

heading = "Projects";
const head_project = head_lvl(3) + heading;
const toc_project = `[[${file_section}${heading}\\|${heading}]]${tbl_pipe}`;

heading = "Parent Tasks";
const head_parent = head_lvl(3) + heading;
const toc_parent_task = `[[${file_section}${heading}\\|${heading}]]`;

const plan_prefix = "Planned for ";
const due_prefix = "Due on ";
const done_prefix = "Completed on ";

// WEEK HABITS AND RITUALS SUBFILE TABLE OF CONTENTS
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;
const toc_title = `${call_start}[!toc]${space}Week${space}${name}${space}${dv_content_link}`;

const toc_week_task_proj_parent =
  call_tbl_start +
  toc_project +
  toc_parent_task +
  call_tbl_end +
  new_line +
  call_tbl_div(2);

const toc_week_task_proj =
  call_tbl_start +
  toc_project +
  file_obj_arr
    .filter((x) => x.sect_level == 1)
    .map((x) => `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_task_parent =
  call_tbl_start +
  toc_parent_task +
  tbl_pipe +
  file_obj_arr
    .filter((x) => x.sect_level == 2)
    .map((x) => `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_task_child_day =
  call_tbl_start +
  file_obj_arr
    .filter((x) => x.sect_level == 3)
    .map((x) => `[[${file_section}${x.head_key}${space}Tasks\\|${x.head_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_task_child_lvl = (prefix, alias) =>
  call_tbl_start +
  file_obj_arr
    .filter((x) => x.sect_level == 3)
    .map((x) => `[[${file_section}${prefix}${x.head_key}\\|${alias}]]`)
    .join(tbl_pipe) +
  call_tbl_end;
const toc_week_task_child_status = [
  toc_week_task_child_lvl(plan_prefix, "Plan"),
  toc_week_task_child_lvl(due_prefix, "Due"),
  toc_week_task_child_lvl(done_prefix, "Done"),
].join(new_line);

const toc = [
  toc_title,
  call_start,
  toc_week_task_proj_parent,
  call_start,
  call_start + "**Daily Tasks**",
  call_start,
  toc_week_task_child_day,
  call_tbl_div(7),
].join(new_line);

for (let i = 0; i < file_obj_arr.length; i++) {
  if (!file_obj_arr[i].status) {
    file_obj_arr[i].query = temp_include(file_obj_arr[i].include_tbl);
  } else if (file_obj_arr[i].type == "child") {
    file_obj_arr[i].query = await tp.user.dv_task_type_status_dates({
      type: file_obj_arr[i].type,
      status: file_obj_arr[i].status,
      start_date: moment_day(file_obj_arr[i].head_key),
      end_date: "",
      md: "true",
    });
  } else {
    file_obj_arr[i].query = await tp.user.dv_task_type_status_dates({
      type: file_obj_arr[i].type,
      status: file_obj_arr[i].status,
      start_date: date_start,
      end_date: date_end,
      md: "true",
    });
  }
}

// Day task file section block embed
file_obj_arr
  .filter((x) => x.sect_level == 3)
  .map(
    (x) =>
      (x.plan =
        head_lvl(4) +
        plan_prefix +
        x.head_key +
        two_new_line +
        "![[" +
        moment_day(x.head_key) +
        "_task_event#Planned for Today]]")
  );
file_obj_arr
  .filter((x) => x.sect_level == 3)
  .map(
    (x) =>
      (x.due =
        head_lvl(4) +
        due_prefix +
        x.head_key +
        two_new_line +
        "![[" +
        moment_day(x.head_key) +
        "_task_event#Due Today]]")
  );
file_obj_arr
  .filter((x) => x.sect_level == 3)
  .map((x) => (x.done = head_lvl(4) + done_prefix + x.head_key));

const sect_proj = file_obj_arr
  .filter((x) => x.sect_level == 1)
  .map((x) => [head_lvl(4) + x.head_key, x.query].join(two_new_line))
  .join(two_new_line);
const sect_parent = file_obj_arr
  .filter((x) => x.sect_level == 2)
  .map((x) => [head_lvl(4) + x.head_key, x.query].join(two_new_line))
  .join(two_new_line);
const sect_child = file_obj_arr
  .filter((x) => x.sect_level == 3)
  .map((x) =>
    [
      head_lvl(3) + x.head_key + " Tasks",
      toc,
      temp_include(x.include_rate),
      x.plan,
      // x.query,
      x.due,
      x.done,
      temp_include(x.include_tbl),
    ].join(two_new_line)
  )
  .join(two_new_line);

const week_file =
  new_line +
  [
    temp_include("112_40_dvmd_wk_task_rate"),
    head_project,
    toc,
    sect_proj,
    head_parent,
    toc,
    sect_parent,
    sect_child,
  ].join(two_new_line);

tR += week_file;
%>
