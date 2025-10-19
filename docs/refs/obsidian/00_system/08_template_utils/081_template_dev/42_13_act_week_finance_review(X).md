<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const cal_week_dir = "10_calendar/12_weeks/";
const goals_dir = "30_goals/";
const personal_proj_dir = "41_personal/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";

/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const hr_line = hyphen.repeat(3);
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const cmnt_ob_start = `${String.fromCodePoint(37).repeat(2)}${space}`;
const cmnt_ob_end = `${space}${String.fromCodePoint(37).repeat(2)}`;
const colon = String.fromCodePoint(0x3a);
const tbl_start =`${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end =`${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${String.fromCodePoint(0x2d)}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${String.fromCodePoint(0x3e)}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_ul}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;
const dv_colon = `${colon.repeat(2)}${space}`;

//-------------------------------------------------------------------
// FORMATTING FUNCTIONS
//-------------------------------------------------------------------
const snake_case_fmt = (name) =>
  name.replaceAll(/(\-\s\-)|(\s)|(\-)]/g, "_").toLowerCase();

const head_two = `${hash.repeat(2)}${space}`;

//-------------------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const pillar_name_alias = "20_00_pillar_name_alias";
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";

const do_due_date = "40_task_do_due_date";
const task_status = "40_task_status";

const related_sect_task_child = "142_00_related_sect_task_child";
const related_lib_sect = "100_60_related_lib_sect";
const related_pkm_sect = "100_70_related_pkm_sect";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

/* ---------------------------------------------------------- */
/*             CONTEXT NAME, VALUE, AND DIRECTORY             */
/* ---------------------------------------------------------- */
const context_name = "Personal";
const context_value = context_name.toLowerCase();
const context_dir = personal_proj_dir;

//-------------------------------------------------------------------
// PROJECT FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const project_value = "financial_planning";
const project_name = "Financial Planning";
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;
const project_dir = `${context_dir}${project_value}/`;

//-------------------------------------------------------------------
// PARENT TASK FILE NAME, ALIAS, LINK, AND DIRECTORY
//-------------------------------------------------------------------
const parent_task_value = "weekly_finance_review";
const parent_task_name = "Weekly Finance Review";
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = `${new_line}${ul_yaml}"${parent_task_link}"`;
const parent_task_dir = `${project_dir}${parent_task_value}/`;

//-------------------------------------------------------------------
// ACTION ITEM TAG, TYPE, AND FILE CLASS
//-------------------------------------------------------------------
const task_tag = "#task";
const type_name = "Action Item";
const type_value = type_name.replaceAll(/\s/g, "_").toLowerCase();
const file_class = "task_child";

//-------------------------------------------------------------------
// SET WEEK
//-------------------------------------------------------------------
const week_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: cal_week_dir,
  file_class: "cal",
  type: "week",
});
const week_obj = await tp.system.suggester(
  (item) => item.key,
  week_obj_arr,
  false,
  "Preview Week?"
);
const week_preview_value = week_obj.value;
const week_preview_name = week_obj.key;
const week_preview_link = `[[${week_preview_value}|${week_preview_name}]]`;

const week_preview_num_regex = /$\d\d/g;
const week_preview_num = week_preview_value.match(week_preview_num_regex);
const week_preview_year_regex = /^\d\d\d\d/g;
const week_preview_year = week_preview_value.match(week_preview_year_regex);

let week_review_num;
if (Number(week_preview_num) == 01) {
  week_review_num = 52;
} else if (Number(week_preview_num) <= 10) {
  week_review_num = `0${Number(week_preview_num) - 1}`;
} else {
  week_review_num = Number(week_preview_num) - 1;
}
const week_review_value = `${week_preview_year}-W${week_review_num}`;
const week_review_name = `Week ${week_review_num}, ${week_preview_year}`;
const week_review_link = `[[${week_review_value}|${week_review_name}]]`;

/* ---------------------------------------------------------- */
/*                      SET DATE AND TIME                     */
/* ---------------------------------------------------------- */
const date = await tp.user.nl_date(tp, "start");
const time = await tp.user.nl_time(tp, "");
const full_date_time = moment(`${date}T${time}`);

//-------------------------------------------------------------------
// WEEKLY REVIEW AND PREVIEW TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const title = "Weekly Finances Review";
const full_title_name = `${week_preview_name} ${title}`;
const short_title_name = `${title.toLowerCase()}`;
const short_title_value = title
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/'/g, "")
  .toLowerCase();
const full_title_value = `${week_preview_value}_${short_title_value}`;

const alias_arr = `${new_line}${ul_yaml}"${title}"${ul_yaml}"${full_title_name}"${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}"${short_title_value}"${new_line}${ul_yaml}"${full_title_value}"`;

const file_name = full_title_value;

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const pillar_value = "finance_budget";
const pillar_name = "Finance and Budget";
const pillar_link = `[[${pillar_value}|${pillar_name}]]`;
const pillar_value_link = `${new_line}${ul_yaml}"${pillar_link}"`;

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, `Goal?`);

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
temp_file_path = `${sys_temp_include_dir}${contact_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const contact_value = include_arr[0];
const contact_name = include_arr[1];
const contact_link = `${contact_value}|${contact_name}`;
const contact_value_link = `${new_line}${ul_yaml}"${contact_link}"`;

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const due_do_value = "do";

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const task_status = await tp.user.include_template(tp, "40_task_status");
const status_value = task_status.split(";")[0];
const status_name = task_status.split(";")[1];
const status_symbol = task_status.split(";")[2];

const space = String.fromCodePoint(0x20);
const checkbox_task_tag = `-${space}[${status_symbol}]${space}${task_tag}${space}`;

//-------------------------------------------------------------------
// FINANCE REVIEW START, END, AND REMINDER TIMES
//-------------------------------------------------------------------
const finance_duration = 60;
const finance_start = moment(full_date_time).format("HH:mm");
const finance_end = moment(full_date_time)
  .add(finance_duration, "minutes")
  .format("HH:mm");
const finance_reminder = moment(full_date_time)
  .subtract(10, "minutes")
  .format("YYYY-MM-DD HH:mm");

// Daily preview task checkbox
const finance_task_checkbox = `${checkbox_task_tag}${title}_${type_value} [time_start:: ${finance_start}]  [time_end:: ${finance_end}]  [duration_est:: ${finance_duration}] ⏰ ${finance_reminder} ➕ ${moment().format("YYYY-MM-DD")} 📅 ${date}`;

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION FOR CHILD TASKS
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_sect_task_child}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_task_event_section = include_arr;

//-------------------------------------------------------------------
// RELATED PKM SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_pkm_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_pkm_section = include_arr;

//-------------------------------------------------------------------
// RELATED LIBRARY SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_lib_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_lib_section = include_arr;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = parent_task_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
   await tp.file.move(`${directory}${file_name}`);
};

tR += "---"
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
---

# <%* tR += title %>

> [!<%* tR += type_value %> ] <%* tR += type_name %> Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) => !contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`
>
> - Week: <%* tR += week_review_link %>
> - Week:: <%* tR += week_preview_link %>

---

## Weekly Finance Review

<%* tR += journal_task_checkbox %>

<%* tR += journal_review_preview_checklist %>

---

## Related Tasks and Events

<%* tR += related_task_event_section %>

## Related Knowledge

<%* tR += related_pkm_section %>

## Related Library Content

<%* tR += related_library_section %>
