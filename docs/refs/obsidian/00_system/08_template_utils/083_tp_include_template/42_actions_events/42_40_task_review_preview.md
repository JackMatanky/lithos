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
const call_title_plan = `${call_start}[!task_plan]${space}`;
const call_title_review = `${call_start}[!task_review]${space}`;
const call_title_preview = `${call_start}[!task_preview]${space}`;

//-------------------------------------------------------------------
// HABITS AND RITUALS REVIEW AND PREVIEW CONTENT
//-------------------------------------------------------------------
function task_day_review(day) {
  const call_title = call_title_review + day;
  const check_one = "Review task counts and schedule from the weekly preview.";
  const check_two = "Review task counts and schedule from daily preview.";
  const check_three = "Review completed tasks.";
  const check_four = "Compare the weekly and daily previews with completed.";
  const check_five =
    "Write insights about comparison and actionable lessons learned.";
  const call_body =
    call_check +
    [check_one, check_two, check_three, check_four, check_five].join(
      new_line + call_check
    );
  return [call_title, call_start, call_body].join(new_line);
}

const task_sunday = task_day_review("Sunday");
const task_monday = task_day_review("Monday");
const task_tuesday = task_day_review("Tuesday");
const task_wednesday = task_day_review("Wednesday");
const task_thursday = task_day_review("Thursday");
const task_friday = task_day_review("Friday");
const task_saturday = task_day_review("Saturday");

heading = "Completed Parent Tasks Review";
check_one = "Review parent tasks completed last week.";
check_two = "Write down insights from the parent tasks.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const task_parent_done = `${title}${body}`;

heading = "Weekly Execution Plan Review";
check_one = "Review the critical actions from last week's plan.";
check_two = "Write down insights about execution and progress.";
check_three = "If necessary, update execution plan.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const task_week_plan_review = `${title}${body}`;

heading = "Active Projects Preview";
check_one = "Review active projects.";
check_two = "Determine projects of interest according to predefined plan or urgency.";
check_three = "Update project next steps or update project status.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const task_project_active = `${title}${body}`;

heading = "Active Parent Tasks Preview";
check_one = "Review active parent tasks of projects of interest.";
check_two = "Determine parent tasks of interest according to predefined plan or urgency.";
check_three = "Update task dates for parent task.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const task_parent_active = `${title}${body}`;

heading = "Weekly Plan Preview";
check_one = "Review strategies and priorities for upcoming week.";
check_two = "Write down upcoming critical actions.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const task_week_plan_preview = `${title}${body}`;

heading = "Upcoming Daily Tasks and Events Preview";
check_one = "Define critical actions' constituent tasks.";
check_two = "Schedule strategic blocks to work on improving performance.";
check_three = "Schedule buffer blocks for 'administrative' tasks.";
check_four = "Schedule three hours worth of breakout blocks for reenergizing activities.";
check_five = "Configure daily schedules for each day of the week.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}${new_line}${call_check}${check_four}${new_line}${call_check}${check_five}`;
const task_daily_schedule = `${title}${body}`;

tR += task_sunday;
tR += ";";
tR += task_monday;
tR += ";";
tR += task_tuesday;
tR += ";";
tR += task_wednesday;
tR += ";";
tR += task_thursday;
tR += ";";
tR += task_friday;
tR += ";";
tR += task_saturday;
tR += ";";
tR += task_parent_done;
tR += ";";
tR += task_week_plan_review;
tR += ";";
tR += task_project_active;
tR += ";";
tR += task_parent_active;
tR += ";";
tR += task_week_plan_preview;
tR += ";";
tR += task_daily_schedule;
%>
