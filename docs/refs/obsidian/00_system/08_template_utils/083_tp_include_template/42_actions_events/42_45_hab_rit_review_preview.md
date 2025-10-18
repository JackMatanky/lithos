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
// TASK PLAN, REVIEW, AND PREVIEW CALLOUTS
//-------------------------------------------------------------------
const plan_call_title = `${call_start}[!task_plan]${space}`;
const call_title_review = `${call_start}[!task_review]${space}`;
const call_title_preview = `${call_start}[!task_preview]${space}`;

//-------------------------------------------------------------------
// HABITS AND RITUALS REVIEW AND PREVIEW CONTENT
//-------------------------------------------------------------------
const check_one = "Review last week's completed _habit_rit_head_.";
const check_two = "Review last week's discarded _habit_rit_head_.";
const check_three =
  "Compare last week's completed and discarded _habit_rit_head_.";
const check_four = "Write insights about comparison.";
const check_five = "Write actionable lessons learned to implement.";
const check_arr = [check_one, check_two, check_three, check_four, check_five];

function habit_rit_review_preview(head) {
  const call_title = `${call_title_review}${head}${new_line}${call_start}${new_line}`;
  const head_small = head.toLowerCase();
  const call_body =
    call_check +
    check_arr
      .map((x) => x.replaceAll(/(_habit_rit_head_)/g, head_small))
      .join(new_line + call_check);
  return call_title + call_body;
}

const hab_rit_habit = habit_rit_review_preview("Habits");
const hab_rit_morning = habit_rit_review_preview("Morning Rituals");
const hab_rit_work_start = habit_rit_review_preview("Workday Startup Rituals");
const hab_rit_work_stop = habit_rit_review_preview("Workday Shutdown Rituals");
const hab_rit_evening = habit_rit_review_preview("Evening Rituals");

heading = "Upcoming Habits and Rituals";
const check_revise =
  "If necessary, revise templates according to lessons learned during review.";
const check_upcoming = "Create habit and ritual files for upcoming week.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_revise}${new_line}${call_check}${check_upcoming}`;
const hab_rit_next_week = `${title}${body}`;

tR += hab_rit_habit;
tR += ";";
tR += hab_rit_morning;
tR += ";";
tR += hab_rit_work_start;
tR += ";";
tR += hab_rit_work_stop;
tR += ";";
tR += hab_rit_evening;
tR += ";";
tR += hab_rit_next_week;
%>