---
title: 50_21_habit_rit_month_proj_parent_tasks
aliases:
  - Monthly Habits and Rituals Project Parent Tasks
  - monthly habit and rituals project parent tasks
  - proj parent tasks habit ritual month
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-03T09:25
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Monthly Habits and Rituals Project Parent Tasks

## Description

> [!snippet] Snippet Details
>
> Plugin:: [[Templater]]
> Language:: [[JavaScript]]
> Input::
> Output::
> Description:: Return the monthly habits and rituals project parent tasks folders and files.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const parent_habit_rit_month = "41_51_parent_habit_rit_month";

//---------------------------------------------------------
// HABITS AND RITUALS PARENT TASKS
//---------------------------------------------------------
// Retrieve the Habits and Rituals Project
// Parent Tasks template and content
template = await tp.file.find_tfile(habit_ritual_monthly_parent_tasks);
content = await tp.file.include(template);
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// HABITS AND RITUALS PARENT TASKS
//---------------------------------------------------------
template = await tp.file.find_tfile(habit_ritual_monthly_parent_tasks);
content = await tp.file.include(template);
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// FOLDER PATH VARIABLES
//---------------------------------------------------------
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = `20_pillars/`;
const goals_dir = `30_goals/`;
const projects_dir = `40_projects/`;
const habit_ritual_proj_dir = `45_habit_ritual/`;
const contacts_dir = `51_contacts/`;
const organizations_dir = `52_organizations/`;

//---------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//---------------------------------------------------------
const pillar_name_alias = `20_00_pillar_name_alias`;
const contact_name_alias = `61_contact_name_alias`;
const org_name_alias = `62_organization_name_alias`;

//---------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//---------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//---------------------------------------------------------
// HABITS AND RITUALS PARENT TASKS ARRAY OF OBJECTS
//---------------------------------------------------------
const subdir_obj_arr = [
  {
    order: "01",
    name: "Habits",
    value: "habits",
    filter: "habit",
    file_class: "task_habit",
  },
  {
    order: "02",
    name: "Morning Rituals",
    value: "morning_rituals",
    filter: "morning_ritual",
    file_class: "task_ritual_morning",
  },
  {
    order: "03",
    name: "Workday Startup Rituals",
    value: "workday_startup_rituals",
    filter: "workday_startup_ritual",
    file_class: "task_ritual_work_startup",
  },
  {
    order: "04",
    name: "Workday Shutdown Rituals",
    value: "workday_shutdown_rituals",
    filter: "workday_shutdown_ritual",
    file_class: "task_ritual_work_shutdown",
  },
  {
    order: "05",
    name: "Evening Rituals",
    value: "evening_rituals",
    filter: "evening_ritual",
    file_class: "task_ritual_evening",
  },
];

//---------------------------------------------------------
// START DATE, END DATE, AND MONTH YEAR VARIATIONS
//---------------------------------------------------------
const date_start = moment().startOf("month").format("YYYY-MM-DD");
const date_end = moment().endOf("month").format("YYYY-MM-DD");
const year_month_short = moment().format("YYYY-MM");
const year_month_long = moment().format("MMMM [']YY");

//---------------------------------------------------------
// CONTEXT NAME AND DIRECTORY
//---------------------------------------------------------
const context_name = `Habits and Rituals`;
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();
const context_dir = habit_ritual_proj_dir;

//---------------------------------------------------------
// PROJECT NAME, ALIAS, LINK, AND DIRECTORY
//---------------------------------------------------------
const project_name = `${year_month_long} ${context_name}`;
const project_value = `${year_month_short}_${context_value}`;
const project_link = `${project_value}|${project_name}`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;
const project_dir = `${context_dir}${project_value}/`;

//---------------------------------------------------------
// FRONTMATTER VARIABLES
//---------------------------------------------------------
let fmatter_title;
let fmatter_alias;
let fmatter_date_start = `date_start: ${date_start}`;
let fmatter_date_end = `date_end: ${date_end}`;
let fmatter_due_do = "due_do: do";
let fmatter_pillar;
let fmatter_context = `context: ${context_value}`;
let fmatter_goal;
let fmatter_project = `project: ${project_value}`;
let fmatter_organization;
let fmatter_contact;
let fmatter_status = "status: to_do";
let fmatter_type = "type: parent_task";
let fmatter_file_class;
let fmatter_date_created = `date_created: ${date_created}`;
let fmatter_date_modified = `date_modified: ${date_modified}`;

//---------------------------------------------------------
// TITLE AND ALIAS VARIABLES
//---------------------------------------------------------
let full_title_name;
let short_title_name;
let full_title_value;
let short_title_value;
let alias_arr;

//---------------------------------------------------------
// PILLAR, ORGANIZATION, AND CONTACT VARIABLES
//---------------------------------------------------------
const pillars_obj_arr = await tp.user.file_by_status({
  dir: pillars_dir,
  status: `active`,
});
let pillars_obj;
let pillar_value;
let pillar_name;
let pillar_link;

const organizations_obj_arr = await tp.user.md_file_name_alias(
  organizations_dir
);
let organizations_obj;
let organization_value;
let organization_name;
let organization_link;

const contact_obj_arr = await tp.user.md_file_name_alias(contacts_dir);
let contact_obj;
let contact_value;
let contact_name;
let contact_link;
const goals = await tp.user.md_file_name(goals_dir);

//---------------------------------------------------------
// INFO, PREVIEW, AND REVIEW CALLOUT VARIABLES
//---------------------------------------------------------
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);

let info_callout;
let preview_callout;
let review_callout;

//---------------------------------------------------------
// TP.CREATE_NEW VARIABLES
//---------------------------------------------------------
let file_name;
let file_content;
let directory;

//---------------------------------------------------------
// LOOP THROUGH ARRAY OF OBJECTS
//---------------------------------------------------------
for (var i = 0; i < subdir_obj_arr.length; i++) {
  // TITLES, ALIAS, AND FILE NAME
  // Titles
  full_title_name = `${year_month_long} ${subdir_obj_arr[i].name}`;
  short_title_name = `${year_month_short} ${subdir_obj_arr[i].name}`;
  full_title_value = `${year_month_long
    .replaceAll(/'/g, "")
    .replaceAll(/\s/g, "_")
    .toLowerCase()}_${subdir_obj_arr[i].value}`;
  short_title_value = `${year_month_short}_${subdir_obj_arr[i].order}_${subdir_obj_arr[i].value}`;
  // Alias
  alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}${full_title_value}${ul_yaml}${short_title_value}`;

  // File name
  file_name = short_title_value;

  fmatter_title = `title: ${file_name}`;
  fmatter_alias = `aliases: ${alias_arr}`;

  // FILE CLASS
  fmatter_file_class = `file_class: ${subdir_obj_arr[i].file_class}`;

  // SET PILLAR FILE AND FULL NAME
  pillar_obj = await tp.system.suggester(
    (item) => item.key,
    pillars_obj_arr,
    false,
    `Pillar for Monthly ${subdir_obj_arr[i].name}?`
  );
  pillar_value = pillar_obj.value;
  pillar_name = pillar_obj.key;
  pillar_link = `${pillar_value}|${pillar_name}`;

  fmatter_pillar = `pillar: ${pillar_value}`;

  // SET GOAL
  fmatter_goal =
    "goal: " +
    (await tp.system.suggester(
      goals,
      goals,
      false,
      `Goal for Monthly ${subdir_obj_arr[i].name}?`
    ));

  // SET ORGANIZATION FILE NAME AND TITLE
  organizations_obj = await tp.system.suggester(
    (item) => item.key,
    organizations_obj_arr,
    false,
    `Organization for Monthly ${subdir_obj_arr[i].name}?`
  );

  organization_value = organizations_obj.value;
  organization_name = organizations_obj.key;

  if (organization_value.includes(`_user_input`)) {
    organization_name = await tp.system.prompt(
      `Organization for Monthly ${subdir_obj_arr[i].name}?`,
      ``,
      false,
      false
    );
    organization_value = organization_name
      .replaceAll(/[,']/g, "")
      .replaceAll(/\s/g, "_")
      .replaceAll(/\//g, "-")
      .replaceAll(/&/g, "and")
      .toLowerCase();
  }

  organization_link = `${organization_value}|${organization_name}`;

  fmatter_organization = `organization: ${organization_value}`;

  // SET CONTACT FILE NAME AND TITLE
  contact_obj = await tp.system.suggester(
    (item) => item.key,
    contact_obj_arr,
    false,
    `Contact for Monthly ${subdir_obj_arr[i].name}?`
  );

  contact_value = contact_obj.value;
  contact_name = contact_obj.key;
  if (contact_value.includes(`_user_input`)) {
    const contact_names = await tp.user.dirContactNames(tp);
    const full_name = contact_names.full_name;
    const last_first_name = contact_names.last_first_name;
    contact_value = full_name;
    contact_value = last_first_name.replaceAll(/[^\w]/g, "_").toLowerCase();
  }
  contact_link = `${contact_value}|${contact_name}`;

  fmatter_contact = `contact: ${contact_value}`;

  // set the subdirectory file class
  fmatter_file_class = `file_class: ${subdir_obj_arr[i].file_class}`;

  // INFO, PREVIEW, AND REVIEW CALLOUTS
  info_callout = `>${space}[!parent_task] ${subdir_obj_arr[i].name} Details${two_space}
>${space}
>${space}Context:: ${context_name}${two_space}
>${space}Pillar:: [[${pillar_link}]]${two_space}
>${space}Goal::${two_space}
>${space}Project:: [[$${project_link}]]${two_space}
>${space}Organization:: [[${organization_link}]]${two_space}
>${space}Contact:: [[${contact_link}]]${two_space}
>${space}
>${space}| Start Date        | End Date        |${two_space}
>${space}| :---------------: | :-------------: |${two_space}
>${space}| [[${date_start}]] | [[${date_end}]] |`;

  preview_callout = `>${space}[!task_preview] ${subdir_obj_arr[i].name} Preview${two_space}
>${space}
>${space}**INCLUDED TASKS AND THEIR PURPOSE**${two_space}
>${space}
>${space}*First Task*${two_space}
>${space}1.${space}habit_ritual::${two_space}
>${space}2.${space}purpose::${two_space}
>${space}
>${space}*Second Task*${two_space}
>${space}1.${space}habit_ritual::${two_space}
>${space}2.${space}purpose::${two_space}
>${space}
>${space}*Third Task*${two_space}
>${space}1.${space}habit_ritual::${two_space}
>${space}2.${space}purpose::${two_space}
>${space}
>${space}*Fourth Task*${two_space}
>${space}1.${space}habit_ritual::${two_space}
>${space}2.${space}purpose::${two_space}
>${space}
>${space}*Fifth Task*${two_space}
>${space}1.${space}habit_ritual::${two_space}
>${space}2.${space}purpose::${two_space}`;

  review_callout = `>${space}[!task_review] ${subdir_obj_arr[i].name} Review${two_space}
>${space}
>${space}How many time was the habit or ritual completed per total?${two_space}
>${space}1. outcome::${two_space}
>${space}
>${space}What went well?${two_space}
>${space}2. keep::${two_space}
>${space}
>${space}What can be improved?${two_space}
>${space}3. improve::${two_space}
>${space}
>${space}What can be started?${two_space}
>${space}4. start::${two_space}
>${space}
>${space}What can be stopped?${two_space}
>${space}5. stop::`;

  // DATAVIEW TASK TABLES
  // TYPES: "parent", "child"
  // STATUS: "due", "done", "null"
  child_task_remaining = await tp.user.dv_proj_task("child", "due");
  child_task_completed = await tp.user.dv_proj_task("child", "done");
  habit_ritual_review = await tp.user.dv_proj_habit_ritual(
    `"${subdir_obj_arr[i].filter}"`
  );

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
# ${full_title_name}\n
${info_callout}\n
---\n
## Prepare and Reflect\n
### Preview\n
${preview_callout}\n
### Review\n
${review_callout}\n
---\n
## Habits and Rituals\n
### Remaining Tasks\n
${child_task_remaining}\n
### Completed Tasks\n
${child_task_completed}\n
### Review of Task Completion Rate\n
${habit_ritual_review}\n
---\n
## Notes\n
---\n
## Resources
\n`;

  // PARENT TASK DIRECTORY PATH AND CREATION
  directory = `${project_dir}${file_name}`;
  await this.app.vault.createFolder(directory);

  // Create subdirectory file
  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(directory)
  );
}
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_21_proj_habit_ritual_month|Monthly Habits and Rituals Project Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[51_21_parent_habit_rit_month]]
2. [[10_pillar_file_name_title_suggester|Pillar File and Full Name]]
3. [[62_organization_file_name_title_suggester|Organization File Name and Title]]
4. [[61_contact_file_name_title_suggester|Contact File Name and Title]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[10_pillar_file_name_title_suggester|Pillar File and Full Name]]

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
