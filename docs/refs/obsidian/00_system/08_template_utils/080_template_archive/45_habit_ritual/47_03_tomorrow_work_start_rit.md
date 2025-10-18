<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const habit_ritual_proj_dir = "45_habit_ritual/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";

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
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const pillar_name_alias = "20_00_pillar_name_alias";
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";

const child_task_info_callout = "42_child_task_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

/* ---------------------------------------------------------- */
/*                      SET DATE AND TIME                     */
/* ---------------------------------------------------------- */
const date = moment().add(1, "days").format("YYYY-MM-DD");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const time = await tp.user.nl_time(tp, "");
const full_date_time = moment(`${date}T${time}`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Habits and Rituals";
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();
const context_dir = habit_ritual_proj_dir;

//-------------------------------------------------------------------
// PROJECT FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const year_month_short = moment(full_date_time).format("YYYY-MM");
const year_month_long = moment(full_date_time).format("MMMM [']YY");

const project_name = `${year_month_long} ${context_name}`;
const project_value = `${year_month_short}_${context_value}`;
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);
const project_dir = `${context_dir}${project_value}/`;

//-------------------------------------------------------------------
// PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const habit_ritual_order = "03";
const habit_ritual_name = "Workday Startup Rituals";
const parent_task_name = `${year_month_long} ${habit_ritual_name}`;
const parent_task_value = `${year_month_short}_${habit_ritual_order}_${habit_ritual_name.replaceAll(/\s/g, "_").toLowerCase()}`;
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);
const parent_task_dir = `${project_dir}${parent_task_value}/`;

//-------------------------------------------------------------------
// RITUAL TASK TAG, TYPE NAMES, AND FILE CLASS
//-------------------------------------------------------------------
const task_tag = "#task";
const full_type_name = `Daily ${habit_ritual_name}`;
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name
  .split(" ")
  .splice(1, full_type_name.split(" ").length)
  .join(" ");
const type_lower = type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_value = type_lower.slice(0, -1);
const file_class = "task_child";

//-------------------------------------------------------------------
// RITUAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${short_date} ${full_type_name}`;
const partial_title_name = `${short_date} ${type_name}`;
const short_title_name = full_type_name.toLowerCase();
const full_title_value = `${short_date_value}_${full_type_value}`;
const partial_title_value = `${short_date_value}_${type_lower}`;
const short_title_value = full_type_value;


const file_name = `${short_date_value}_${type_lower}`;

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  "20_00_pillar_name_alias"
);
const pillar_value = pillar_name_alias.split(";")[0];
const pillar_value_link = pillar_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${full_type_name}?`
);

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const org_name_alias = await tp.user.include_template(
  tp,
  "52_organization_name_alias"
);
const organization_value = org_name_alias.split(";")[0];
const organization_value_link = org_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
const contact_name_alias = await tp.user.include_template(
  tp,
  "51_contact_name_alias"
);
const contact_value = contact_name_alias.split(";")[0];
const contact_name = contact_name_alias.split(";")[1];
const contact_value_link = contact_name_alias.split(";")[2];

//-------------------------------------------------------------------
// DO/DUE DATE
//-------------------------------------------------------------------
const due_do_value = "do";

//-------------------------------------------------------------------
// TASK STATUS AND SYMBOL
//-------------------------------------------------------------------
const status_name = "To do";
const status_value = status_name.replaceAll(/\s/g, "_").toLowerCase();
const status_symbol = " ";

const space = String.fromCodePoint(0x20);
const checkbox_task_tag = `-${space}[${status_symbol}]${space}${task_tag}${space}`;

//-------------------------------------------------------------------
// EMAIL PREVIEW START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const email_duration = 6;
const email_start = moment(full_date_time).format("HH:mm");
const email_end = moment(full_date_time)
  .add(email_duration, "minutes")
  .format("HH:mm");
const email_reminder = moment(full_date_time)
  .subtract(5, "minutes")
  .format("YYYY-MM-DD HH:mm");

const email_title = "Daily Email Review";

// Daily email review task checkbox
const email_task_checkbox = `${checkbox_task_tag}${email_title}_${type_value} [time_start:: ${email_start}]  [time_end:: ${email_end}]  [duration_est:: ${email_duration}] ⏰ ${email_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// WHATSAPP PREVIEW START AND END TIMES
//-------------------------------------------------------------------
const whatsapp_duration = 6;
const whatsapp_start = moment(`${date}T${email_end}`)
  .add(1, "minutes")
  .format("HH:mm");
const whatsapp_end = moment(`${date}T${whatsapp_start}`)
  .add(whatsapp_duration, "minutes")
  .format("HH:mm");

const whatsapp_title = "Daily WhatsApp Review";

// Daily email review task checkbox
const whatsapp_task_checkbox = `${checkbox_task_tag}${whatsapp_title}_${type_value} [time_start:: ${whatsapp_start}]  [time_end:: ${whatsapp_end}]  [duration_est:: ${whatsapp_duration}] ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// DAILY MORNING RITUALS FILE LINK
//-------------------------------------------------------------------
const morn_rit_order = "02";
const full_morn_rit_name = "Daily Morning Rituals";
const morn_rit_name = full_morn_rit_name
  .split(" ")
  .splice(1, full_morn_rit_name.split(" ").length)
  .join(" ");
const morn_rit_value = morn_rit_name.replaceAll(/\s/g, "_").toLowerCase();
const morn_rit_type = morn_rit_value.slice(0, -1);
const morn_rit_link = `[[${short_date_value}_${morn_rit_value}\\|${full_morn_rit_name}]]`;

//-------------------------------------------------------------------  
// CHILD TASK INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${child_task_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const child_task_info = include_arr;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${context_value}]${space}${type_name}${space}Details${new_line}${call_start}${new_line}`;

const dv_date = `${backtick}dv:${space}this.file.frontmatter.date${backtick}`;
const info_date = `${call_start}Date::${space}${dv_date}${two_space}`;

const info_body = `${child_task_info}${new_line}${info_date}`;

const info = `${info_title}${info_body}${two_new_line}${hr_line}${new_line}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = parent_task_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
due_do: <%* tR += due_do_value %>
pillar: <%* tR += pillar_value_link %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
project: <%* tR += project_value_link %>
parent_task: <%* tR += parent_task_value_link %>
organization: <%* tR += organization_value_link %>
contact: <%* tR += contact_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_type_name %>

<%* tR += info %>

## <%* tR += type_name %>

### Email Review

<%* tR += email_task_checkbox %>

### WhatsApp Review

<%* tR += whatsapp_task_checkbox %>

---

> [!<%* tR += morn_rit_type %> ] Today's Morning Rituals
> 
> | `BUTTON[button-morn-rit-today]` | <%* tR += morn_rit_link %> |
> | --------------------- | ---------------------- |

---

## Related

### Tasks and Events

### Notes

---

## Resources
