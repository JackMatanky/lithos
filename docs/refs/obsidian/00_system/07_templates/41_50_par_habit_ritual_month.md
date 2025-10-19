<%*
//-------------------------------------------------------------------
// GLOBAL FOLDER PATH VARIABLES
//-------------------------------------------------------------------
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
// SET THE FILE'S CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// SET THE PROJECT'S START AND END DATES
//-------------------------------------------------------------------
// Set start and end dates for monthly habits
const date_start = moment().startOf("month").format("YYYY-MM-DD");
const date_end = moment().endOf("month").format("YYYY-MM-DD");

//-------------------------------------------------------------------
// SET THE TASK CONTEXT AND TYPE
//-------------------------------------------------------------------
const context_name = "Habits and Rituals";
const context = context_name.replaceAll(/\sand\s/g, "_").toLowerCase();
const type = `parent_task`;

//-------------------------------------------------------------------
// SET TASK SUBTYPE AND FILE CLASS
//-------------------------------------------------------------------
const subtype_order = `01`;
const full_subtype_name = `Habits`;
const full_subtype_value = subtype_name.replaceAll(/\s/g, "_").toLowerCase();
const file_class = `task_${subtype_value}`;

//-------------------------------------------------------------------
// SET THE FILE'S TITLE AND SHORT TITLE
//-------------------------------------------------------------------
const year_month_short = moment().format("YYYY-MM");
const year_month_long = moment().format("MMMM [']YY");

const content_title = `${year_month_long} ${full_subtype_name}`;
const title = `${year_month_short} ${full_subtype_name}`;
const short_title = `${year_month_short} ${full_subtype_value}`;
const alias = `${year_month_short}_${full_subtype_value}`;

//-------------------------------------------------------------------
// SET PILLAR
//-------------------------------------------------------------------
// Retrieve all files in the Pillars directory
const pillars = await tp.user.file_by_status({
  dir: pillars_dir,
  status: `active`,
});

const pillar = await tp.system.suggester(
  pillars,
  pillars,
  false,
  `Pillar?`
);

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal?`
);

//-------------------------------------------------------------------
// SET ORGANIZATION
//-------------------------------------------------------------------
const organizations = await tp.user.md_file_name(organizations_dir);
const organization = await tp.system.suggester(
  organizations,
  organizations,
  false,
  `Organization?`
);

//-------------------------------------------------------------------
// SET CONTACT
//-------------------------------------------------------------------
const contacts = await tp.user.md_file_name(contacts_dir);
const contact = await tp.system.suggester(
  contacts,
  contacts,
  false,
  `Contact?`
);

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const due_do = `do`;

//-------------------------------------------------------------------
// SET TASK STATUS
//-------------------------------------------------------------------
const status = `to_do`;

//-------------------------------------------------------------------
// SET PROJECT AND PARENT FOLDER DIRECTORIES
//-------------------------------------------------------------------
const project = `${year_month_short} ${context_name}`;
const project_dir = `${habit_ritual_proj_dir}${project}`;
const parent_folder = `${year_month_short} ${subtype_order}_${full_subtype_value}`;

//-------------------------------------------------------------------
// CREATE AND MOVE TO PARENT TASK DIRECTORY
//-------------------------------------------------------------------
const folder_path = tp.file.folder(true);
const parent_task_dir = `${project_dir}/${parent_folder}`;

// Create the project directory if it does not exist
if (folder_path!= parent_task_dir) {
  await this.app.vault.createFolder(parent_task_dir);
};

// Move project file into the project's directory
await tp.file.move(`${parent_task_dir}/${title}`);

tR += "---";
%>
title: "<%* tR += title %>"
uuid: <%* tR += await tp.user.uuid() %>
aliases:
  - "<%* tR += title %>"
  - "<%* tR += content_title %>"
  - "<%* tR += short_title %>"
  - "<%* tR += alias %>"
date_start: <%* tR += date_start %>
date_end: <%* tR += date_end %>
due_do: <%* tR += due_do %>
pillar: <%* tR += pillar %>
context: <%* tR += context %>
goal: <%* tR += goal %>
project: <%* tR += project %>
organization: <%* tR += organization %>
contact: <%* tR += contact %>
status: <%* tR += status %>
type: <%* tR += type %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += content_title %>

> [!Info]
> - **Context**: `dv: choice(this.file.frontmatter.context = "habit_ritual", "Habits and Rituals", upper(substring(this.file.frontmatter.context, 0, 1)) + substring(this.file.frontmatter.context, 1))`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`

## Notes

## Tasks

### Remaining Tasks

```tasks
not done
sort by due
(path does not include 00_system) AND (path includes <%* tR += parent_task_dir %>)
```

### Completed Tasks

```dataview
TABLE WITHOUT ID
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)", "") AS Task,
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "^[A-Za-z0-9\'\-\s]*_", "") AS Type,
	T.completion AS Completed,
	T.time_start AS Start,
	T.time_end AS End,
	(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) -
	date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)) AS Duration,
	T.section AS Link
FROM #task AND "<%* tR += parent_task_dir %>"
FLATTEN file.tasks AS T
WHERE T.completed
SORT T.completion, T.time_start ASC
```

---

## Resources
