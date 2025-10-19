<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const projects_dir = "40_projects/";
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
const do_due_date = "40_task_do_due_date";
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = "Project";
const type_value = type_name.toLowerCase();
const file_class = `task_${type_value}`;

//-------------------------------------------------------------------
// SET FILE'S TITLE AND ALIAS
//-------------------------------------------------------------------
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt("Title", null, true, false);
} else {
  title = tp.file.title;
}
title = title.trim();
const alias = title.toLowerCase();

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar = include_arr[0];
const pillar_name = include_arr[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, "Goal?");

/* ------------------- FILE PATH VARIABLES ------------------ */
const folder_path = `${tp.file.folder(true)}/`;
const folder_path_split = folder_path.split("/");
const folder_path_length = folder_path_split.length;

//-------------------------------------------------------------------
// SET TASK CONTEXT BY FILE PATH OR SUGGESTER
//-------------------------------------------------------------------
let context_dir;
let context_value;
let context_name;
if (projects_dir == `${folder_path_split[0]}/` && folder_path_length >= 2) {
  context_dir = `${projects_dir}${folder_path_split[1]}/`;
  context_value = folder_path_split[1].slice(3);
  const context_arr = context_value.split("_");
  for (var i = 0; i < context_arr.length; i++) {
    context_name += `${context_arr[i].charAt(0).toUpperCase()}${context_arr[i].substring(1)} `;
  };
  context_name.trim();
} else {
  const context_obj = await tp.user.task_context(tp);
  context_dir = context_obj.directory;
  context_value = context_obj.value;
  context_name = context_obj.key;
}

//-------------------------------------------------------------------
// PROJECT DIRECTORY
//-------------------------------------------------------------------
const project_dir = `${context_dir}${title}`;

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
temp_file_path = `${sys_temp_include_dir}${org_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const organization_value = include_arr[0];
const organization_name = include_arr[1].replace(/\n/, "");
const organization_link = `[[${organization_value}|${organization_name}]]`;
const organization_value_link = `${new_line}${ul_yaml}"${organization_link}"`;

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
temp_file_path = `${sys_temp_include_dir}${contact_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const contact = include_arr[0];
const contact_name = include_arr[1];

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const do_due_date = await tp.user.include_template(tp, "40_task_do_due_date");
const due_do_value = do_due_date.split(";")[0];
const due_do_name = do_due_date.split(";")[1];

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const status_obj_arr = [
  { key: "To do", value: "to_do" },
  { key: "In Progress", value: "in_progress" },
  { key: "Done", value: "done" },
  { key: "Schedule", value: "schedule" },
];
const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  "Status?"
);
const status = status_obj.value;

//-------------------------------------------------------------------
// DATAVIEW TASK TABLES
//-------------------------------------------------------------------
const proj_task_remaining = await tp.user.dv_proj_task(project_dir, "due");
const proj_task_completed = await tp.user.dv_proj_task(project_dir, "done");

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = project_dir;

if (folder_path!= directory) {
  await tp.file.move(`${directory}${title}`);
}

//-------------------------------------------------------------------
// CREATE SUBDIRECTORIES AND FILES
//-------------------------------------------------------------------
const subdir_obj_arr = [
  {
    name: `Tasks for ${title}`,
    value: `tasks for ${alias}`,
  },
  {
    name: `Events for ${title}`,
    value: `events for ${alias}`,
  },
];

// Subdirectory page frontmatter variables
let fmatter_title;
let fmatter_alias;
let fmatter_date_start = `date_start:`;
let fmatter_date_end = `date_end:`;
let fmatter_due_do = `due_do: ${due_do}`;
let fmatter_pillar = `pillar: ${pillar}`;
let fmatter_context = `context: ${context}`;
let fmatter_goal = `goal: ${goal}`;
let fmatter_project = `project: ${title}`;
let fmatter_organization = `organization: ${organization}`;
let fmatter_contact = `contact: ${contact}`;
let fmatter_status = `status: ${status}`;
let fmatter_type = `type: parent_task`;
let fmatter_file_class = `file_class: task_parent`;
let fmatter_date_created = `date_created: ${date_created}`;
let fmatter_date_modified = `date_modified: ${date_modified}`;

// Variables for creating a new file
let file_name;
let file_content;
let project_subdir;

// Loop through the array of objects
for (var i = 0; i < subdir_obj_arr.length; i++) {
  file_name = `${subdir_obj_arr[i].name}`;
  project_subdir = `${project_dir}/${file_name}`;
  await this.app.vault.createFolder(project_subdir);

  fmatter_title = `title: ${file_name}`;
  fmatter_alias = `aliases:  [${subdir_obj_arr[i].name}, ${subdir_obj_arr[i].value}]`;

  //-------------------------------------------------------------------
  // PARENT TASK CALLOUTS
  //-------------------------------------------------------------------
  const callout = `> [!parent_task] Parent Task Details

>
> - **Context**: `dv: choice(this.file.frontmatter.context = "habit_ritual", "Habits and Rituals", upper(substring(this.file.frontmatter.context, 0, 1)) + substring(this.file.frontmatter.context, 1))`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`
  //-------------------------------------------------------------------
  // DATAVIEW TABLES
  //-------------------------------------------------------------------

  // Field variables
  const date_start = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)`;
  const date_end = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)`;
  const task_duration = `dur(${date_end} - ${date_start})`;
  const task_estimate = `dur(T.duration_est + " minutes")`;



  // Remaining task code blocks
const remaining_tasks = `${three_backtick}dataview
TABLE WITHOUT ID
	link(T.section,
		regexreplace(
			regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""),
		"_$", "")) AS Task,
	choice(contains(T.text, "_action_item"),
		"🔨Task",
		choice(contains(T.text, "_meeting"),
			"🤝Meeting",
			choice(contains(T.text, "_habit"),
				"🤖Habit",
				choice(contains(T.text, "_morning_ritual"),
					"🍵Rit.",
					choice(contains(T.text, "_workday_startup_ritual"),
						"🌇Rit.",
						choice(contains(T.text, "_workday_shutdown_ritual"),
							"🌆Rit.",
							"🛌Rit.")))))) AS Type,
	T.due,
	T.time_start AS Start,
	T.time_end AS End,
	T.duration_est + " min" AS Estimate
FROM
	#task
	AND "${project_subdir}"
FLATTEN
	file.tasks AS T
WHERE
	any(file.tasks, (t) =>
! t.completed)
		AND t.status!= "-")
SORT
	T.due,
	T.time_start ASC
${three_backtick}`;



  // Completed task code blocks
const completed_tasks = `${three_backtick}dataview
TABLE WITHOUT ID
	link(T.section,
		regexreplace(
			regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""),
		"_$", "")) AS Task,
	choice(contains(T.text, "_action_item"),
		"🔨Task",
		choice(contains(T.text, "_meeting"),
			"🤝Meeting",
			choice(contains(T.text, "_habit"),
				"🤖Habit",
				choice(contains(T.text, "_morning_ritual"),
					"🍵Rit.",
					choice(contains(T.text, "_workday_startup_ritual"),
						"🌇Rit.",
						choice(contains(T.text, "_workday_shutdown_ritual"),
							"🌆Rit.",
							"🛌Rit.")))))) AS Type,
	T.time_start AS Start,
	T.time_end AS End,
	T.duration_est + " min" AS Estimate,
	choice((${task_estimate} = ${task_duration}),
		"👍On Time",
		choice(
			(${task_estimate} > ${task_duration}),
				"🟢" + (${task_estimate} - ${task_duration}),
				"❗" + (${task_duration} - ${task_estimate}))) AS Accuracy,
FROM
	#task
	AND ${project_subdir}
FLATTEN
	file.tasks AS T
WHERE
	any(file.tasks, (t) =>
		t.completed)
		AND T.status!= "-")
SORT
	T.completion,
	T.time_start ASC
${three_backtick}`;

  remaining_tasks = `${three_backtick}tasks
not done
sort by due
(path does not include 00_system) AND (path includes ${project_subdir})
${three_backtick}`;

  file_content = `---
${fmatter_title}
${fmatter_alias}
${fmatter_date_start}
${fmatter_date_end}
${fmatter_due_do}
${fmatter_pillar}
${fmatter_context}
${fmatter_goal}
${fmatter_project}
${fmatter_organization}
${fmatter_contact}
${fmatter_status}
${fmatter_type}
${fmatter_file_class}
${fmatter_date_created}
${fmatter_date_modified}
---\n
tags::\n
---\n

# ${file_name}\n

${callout}\n

## Notes\n

## Tasks\n

### Remaining Tasks\n

${remaining_tasks}\n

### Completed Tasks\n

${completed_tasks}\n
---\n

## Resources

\n`;

  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(project_subdir)
);
}



tR += "---"
%>
title: "<%* tR += title %>"
aliases:
  - "<%* tR += title %>"
  - "<%* tR += alias %>"
date_start:
date_end:
due_do: <%* tR += due_do %>
pillar: <%* tR += pillar %>
context: <%* tR += context %>
goal: <%* tR += goal %>
organization: <%* tR += organization %>
contact: <%* tR += contact %>
status: <%* tR += status %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
---

# <%* tR += title %>

> [!<%* tR += type_value %> ] <%* tR += type_name %> Details
>
> - **Context**: `dv: choice(this.file.frontmatter.context = "habit_ritual", "Habits and Rituals", upper(substring(this.file.frontmatter.context, 0, 1)) + substring(this.file.frontmatter.context, 1))`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> Organization: [[<%* tR += organization %>]]
> - **Contact**: `dv: this.file.frontmatter.contact`

## Notes

## Tasks

### Remaining Tasks

<%* tR += proj_task_remaining %>

### Completed Tasks

<%* tR += proj_task_completed %>

### All Tasks

---

## Resources
