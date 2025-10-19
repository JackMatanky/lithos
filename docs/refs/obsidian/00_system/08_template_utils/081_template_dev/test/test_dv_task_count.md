<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const education_proj_dir = "42_education/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";
const lib_books_dir = "60_library/61_books/";

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
// NULL VALUE, NAME, LINK, AND YAML LINK
//-------------------------------------------------------------------
const null_value = "null";
const null_name = "Null";
const null_link = `[[${null_value}|${null_name}]]`;
const null_yaml_li = yaml_li(null_link);

const dv = this.app.plugins.plugins["dataview"].api

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const duration = dv.luxon.Duration;
const datetime = dv.luxon.DateTime;
const today = moment().format("YYYY-MM-DD");
const tomorrow = moment().add(1, "days").format("YYYY-MM-DD");
//const today = datetime.now().toFormat("yyyy-MM-dd");
//const yesterday = datetime.now().minus({days: 2}).toFormat("yyyy-MM-dd");

//-------------------------------------------------------------------
// REGEX VARIABLES
//-------------------------------------------------------------------
const regex_task_name = /#task\s(.+)_(action_|meeting|phone_|interview|appointment|event|gathering|hangout|habit|morning_|workday_|evening_).+/g;
const regex_time_start = /#task.+time_start::\s(\d\d:\d\d).+/g;
const regex_time_end = /#task.+time_end::\s(\d\d:\d\d).+/g;

//-------------------------------------------------------------------
// TASKLIST QUERIES
//-------------------------------------------------------------------
const task_pages = dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks;

// TASK COUNT AND COMPLETION RATES
function task_name_clean(task) {
  const name = task.text.replace(regex_task_name, "$1");
  return name;
}

const today_total = [
  ...new Set(
    task_pages
      .where(
        (t) =>
          dv.equal(datetime.fromISO(t.due).toFormat("yyyy-MM-dd"), today) &&
          t.text.includes("#task")
      )
      .map((t) => task_name_clean(t))
  ),
].length;

const today_total_task_event = [
  ...new Set(
    task_pages
      .where(
        (t) =>
          dv.equal(datetime.fromISO(t.due).toFormat("yyyy-MM-dd"), today) &&
          t.text.includes("#task") &&
          !(t.text.includes("_habit") || t.text.includes("_ritual"))
      )
      .map((t) => task_name_clean(t))
  ),
].length;

const today_total_habit_rit = [
  ...new Set(
    task_pages
      .where(
        (t) =>
          dv.equal(datetime.fromISO(t.due).toFormat("yyyy-MM-dd"), today) &&
          t.text.includes("#task") &&
          (t.text.includes("_habit") || t.text.includes("_ritual"))
      )
      .map((t) => task_name_clean(t))
  ),
].length;

tR += today_total;
tR += two_new_line;
tR += today_total_task_event;
tR += two_new_line;
tR += today_total_habit_rit;
%>
