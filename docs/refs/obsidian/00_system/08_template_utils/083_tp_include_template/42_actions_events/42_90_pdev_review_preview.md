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
// PDEV REVIEW AND PREVIEW CONTENT
//-------------------------------------------------------------------
heading = "Daily Recounting Review";
check_one = "Skim through daily reflections.";
check_two = "Write about the past week.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const pdev_recall = `${title}${body}`;

heading = "Daily Best Experience Review";
check_one = "Review last week's best experiences.";
check_two = "Write about trends in the week's best experiences.";
check_three = "Write insights about the trends.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const pdev_experience = `${title}${body}`;

heading = "Daily Achievement Review";
check_one = "Review the last week's achievements.";
check_two = "Congratulate myself on my achievements.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const pdev_achievement = `${title}${body}`;

heading = "Daily Gratitude Review";
check_one = "Review and recall moments of gratitude.";
check_two = "Review and recall moments of self-gratitude.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const pdev_gratitude = `${title}${body}`;

heading = "Daily Blindspot Review";
check_one = "Review last week's blindspots.";
check_two = "Write about trends in the blindspots.";
check_three = "Write actionable insights on the trends on how to avoid repeating blindspots.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const pdev_blindspot = `${title}${body}`;

heading = "Daily Detachment Review";
check_one = "Review last week's detachments.";
check_two = "Write about trends in the detachments.";
check_three = "Determine focus areas based on detachment trends and prep.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const pdev_detach = `${title}${body}`;

heading = "Limiting Belief Review";
check_one = "Review last week's limiting beliefs.";
check_two = "Review and clarify the limiting belief.";
check_three = "Review and clarify the perspective created by the limiting belief.";
check_four = "Review and clarify the limiting belief's effect and cost.";
check_five = "Review and clarify the limiting belief's background.";
check_six = "Review and clarify the limiting belief's reframing.";
check_seven = "Review and clarify the limiting truth's affirmation.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}${new_line}${call_check}${check_four}${new_line}${call_check}${check_five}${new_line}${call_check}${check_six}${new_line}${call_check}${check_seven}`;
const pdev_limiting_belief = `${title}${body}`;

heading = "Daily Lessons Learned Review";
check_one = "Review last week's lessons learned.";
check_two = "Distill last week's lessons learned and include lessons from other journals.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const pdev_lesson = `${title}${body}`;

tR += pdev_recall;
tR += ";";
tR += pdev_experience;
tR += ";";
tR += pdev_achievement;
tR += ";";
tR += pdev_gratitude;
tR += ";";
tR += pdev_blindspot;
tR += ";";
tR += pdev_detach;
tR += ";";
tR += pdev_limiting_belief;
tR += ";";
tR += pdev_lesson;
%>