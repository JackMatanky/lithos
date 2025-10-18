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
// PKM REVIEW AND PREVIEW CONTENT
//-------------------------------------------------------------------
heading = "Permanent Notes Review";
check_one = "Review last week's permanent notes.";
check_two = "Indicate notes with content about which I want to learn more.";
check_three = "Create tasks for learning more about topics in select permanent notes.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const pkm_permanent = `${title}${body}`;

heading = "Literature Notes Review";
check_one = "Review last week's literature notes.";
check_two = "Determine which notes to develop into permanent notes.";
check_three = "Create tasks for developing literature notes.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const pkm_literature = `${title}${body}`;

heading = "Fleeting Notes Review";
check_one = "Review last week's permanent notes.";
check_two = "Determine which notes to develop into permanent notes.";
check_three = "Create tasks for developing select fleeting notes.";
title = `${call_title_review}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}${new_line}${call_check}${check_three}`;
const pkm_fleeting = `${title}${body}`;

heading = "Notes with Review Status";
check_one = "Review notes with review status.";
check_two = "Determine new note status.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const pkm_review = `${title}${body}`;

heading = "Notes with Clarify Status";
check_one = "Review notes with clarify status.";
check_two = "Clarify the note and/or change the note status.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const pkm_clarify = `${title}${body}`;

heading = "Notes with Develop Status";
check_one = "Review notes with develop status.";
check_two = "Develop the note and/or change the note status.";
title = `${call_title_preview}${heading}${new_line}${call_start}${new_line}`;
body = `${call_check}${check_one}${new_line}${call_check}${check_two}`;
const pkm_develop = `${title}${body}`;

tR += pkm_permanent;
tR += ";";
tR += pkm_literature;
tR += ";";
tR += pkm_fleeting;
tR += ";";
tR += pkm_review;
tR += ";";
tR += pkm_clarify;
tR += ";";
tR += pkm_develop;
%>