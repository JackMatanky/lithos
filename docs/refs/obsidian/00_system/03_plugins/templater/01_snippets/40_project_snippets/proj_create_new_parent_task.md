---
title: proj_create_new_parent_task
aliases:
  - Create Parent Task Files and Folders for Project
  - Create Project Parent Task Files and Folders
  - create_new_project_parent_task
plugin: templater
language:
  - javascript
module:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-05T09:14
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/create_new
---
# Create Parent Task Files and Folders for Project

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description::

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// CREATE SUBDIRECTORIES AND FILES
//---------------------------------------------------------
// Object array of subdirectory variable
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
let fmatter_organization = `organization: ${organization}`
let fmatter_contact = `contact: ${contact}`;
let fmatter_status = "status: ${status}";
let fmatter_type = "type: parent_task";
let fmatter_file_class = `file_class: task_parent`;
let fmatter_date_created = `date_created: ${date_created}`;
let fmatter_date_modified = `date_modified: ${date_modified}`;

// Variables for creating a new file
let file_name;
let file_content;
let project_subdir;

// Loop through the array of objects
for (var i = 0; i < subdir_obj_arr.length; i++) {
  // Set the file name
  file_name = `${subdir_obj_arr[i].name}`;
  // Set the frontmatter title
  fmatter_title = `title: ${file_name}`;
  // Set the frontmatter aliases
  fmatter_alias = `aliases:  [${subdir_obj_arr[i].name}, ${subdir_obj_arr[i].value}]`;

  // set the project subdirectory
  project_subdir = `${project_dir}/${file_name}`;

  // Create subdirectory
  await this.app.vault.createFolder(project_subdir);

  // Unicode for backticks
  backtick = String.fromCodePoint(0x60);
  three_backtick = backtick.repeat(3);

  // regex tags for completed and remaining task code blocks
  tag_inline_regex = `(#task)|\[.*$`;
  type_regex = `(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)`;
  task_title_regex = `^[A-Za-z0-9:;\'\-\s]*_`;

  // Completed task code blocks
  completed_tasks = `${three_backtick}dataview
TABLE WITHOUT ID
	regexreplace(regexreplace(T.text, "${tag_inline_regex}", ""), "${type_regex}", "") AS Task,
	T.completion AS Completed,
	T.time_start AS Start,
	T.time_end AS End,
	(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) -
	date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)) AS Duration,
	T.section AS Link
FROM #task AND "${project_dir}"
FLATTEN file.tasks AS T
WHERE
	T.completed
	AND contains(T.text, "${subdir_obj_arr[i].filter}")
SORT T.completion, T.time_start ASC
${three_backtick}`;

  // Remaining task code blocks
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
> [!Info]
> - **Context**: `dv: choice(this.file.frontmatter.context = "habit_ritual", "Habits and Rituals", upper(substring(this.file.frontmatter.context, 0, 1)) + substring(this.file.frontmatter.context, 1))`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`  \n
## Notes\n
## Tasks\n
### Remaining Tasks\n
${remaining_tasks}\n
### Completed Tasks\n
${completed_tasks}\n
---\n
## Resources
\n`;

  // Create subdirectory file
  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(project_subdir)
  );
};
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// CREATE SUBDIRECTORIES AND FILES
//---------------------------------------------------------
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
let fmatter_organization = `organization: ${organization}`
let fmatter_contact = `contact: ${contact}`;
let fmatter_status = "status: ${status}";
let fmatter_type = "type: parent_task";
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

  backtick = String.fromCodePoint(0x60);
  three_backtick = backtick.repeat(3);

  tag_inline_regex = `(#task)|[.*$`;
  type_regex = `(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)`;
  task_title_regex = `^[A-Za-z0-9:;\'\-\s]*_`;

  completed_tasks = `${three_backtick}dataview
TABLE WITHOUT ID
	regexreplace(regexreplace(T.text, "${tag_inline_regex}", ""), "${type_regex}", "") AS Task,
	T.completion AS Completed,
	T.time_start AS Start,
	T.time_end AS End,
	(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) -
	date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)) AS Duration,
	T.section AS Link
FROM #task AND "${project_dir}"
FLATTEN file.tasks AS T
WHERE T.completed
	AND contains(T.text, "${subdir_obj_arr[i].filter}")
SORT T.completion, T.time_start ASC
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
> [!Info]
> - **Context**: `dv: choice(this.file.frontmatter.context = "habit_ritual", "Habits and Rituals", upper(substring(this.file.frontmatter.context, 0, 1)) + substring(this.file.frontmatter.context, 1))`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`  \n
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
};
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

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
	file.frontmatter.definition AS Definition
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
