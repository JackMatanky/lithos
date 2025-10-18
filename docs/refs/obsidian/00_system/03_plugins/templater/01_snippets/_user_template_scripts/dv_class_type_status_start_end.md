---
title: dv_class_type_status_start_end
aliases:
  - Dataview Tables by File Class, Type, Status, Start Date, and End Date
  - dataview tables by file class, type, status, start date, and end date
  - dataview tables by file class type status start date end date
  - dv class type status start end
plugin: templater
language:
  - javascript
module:
  - 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-25T15:12
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Dataview Tables by File Class, Type, Status, Start Date, and End Date

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a dataview table based on file class, type, status, start date, and end date.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// SECT: >>>>> GENERAL <<<<<
// file title name and link
const title_link = `link(file.link, file.frontmatter.aliases[0])`;

// File type
const file_type = `file.frontmatter.type AS Type`;

// File subtype
const file_subtype = `file.frontmatter.subtype AS Subtype`;

// File status
const yaml_status = `file.frontmatter.status`;
const file_status = `choice(${yaml_status} = "undetermined" OR ${yaml_status} = "schedule", 
		"🤷Unknown", 
		choice(${yaml_status} = "to_do", 
			"🔜To do", 
			choice(${yaml_status} = "in_progress", 
				"👟In progress", 
				choice(${yaml_status} = "done", 
					"✔️Done", 
					"🗄️Resource"))))
	AS Status`;

// Tags
const tags = `file.etags AS Tags`;

// SECT: >>>>> NOTES <<<<<

const note_status = `choice(${yaml_status} = "schedule", 
		"🤷Unknown", 
		choice(${yaml_status} = "review", 
			"🔜Review", 
			choice(${yaml_status} = "clarify", 
				"🌱Clarify", 
                choice(${yaml_status}) = "develop",
                    "🪴Develop",
                    choice(${yaml_status} = "done", 
					    "🌳Done", 
					    "🗄️Resource")))))
	AS Status`;

// SECT: >>>>> LIBRARY <<<<<
// Author
const author = `file.frontmatter.author AS Author`;

// Date published
const date_publish = `choice(contains(file.frontmatter.type, "book"), file.frontmatter.year_published, file.frontmatter.date_published) AS "Date Published"`;

// SECT: >>>>> TASK <<<<<
// Date span
const yaml_date_start = `file.frontmatter.task_start`;
const yaml_date_end = `file.frontmatter.task_end`;
const date_span = `choice((regextest(".", ${yaml_date_start}) AND regextest(".", ${yaml_date_end})), 
		(${yaml_date_start} + " → " + ${yaml_date_end}),
		choice(regextest(".", ${yaml_date_start}),
			(${yaml_date_start} + " → Present"),
			"null")) 
	AS Dates`;

// Task context
const context = `file.frontmatter.context AS Context`;

// Task project
const project = `file.frontmatter.project AS Project`;

// Organization
const org = `file.frontmatter.organization AS "Org."`;
//---------------------------------------------------------
// SECT: >>>>> DATA SOURCES <<<<<
//---------------------------------------------------------
// Template directory
const source = `-"00_system/05_templates"`;

// Library directory
const lib_dir = `"60_library"`;

// Inbox directory
const inbox = `"inbox"`;

// Projects directory
const proj_dir = `"40_projects"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//---------------------------------------------------------
// Content creation date
const creation_date = `file.frontmatter.date_created`;

// Schedule or undetermined status filter
const undetermined_filter = `(contains(file.frontmatter.status, "undetermined")
	OR contains(file.frontmatter.status, "schedule"))`;

// Active filter
const active_filter = `(contains(file.frontmatter.status, "to_do")
OR contains(file.frontmatter.status, "in_progress"))`;

// Content completion date
const completion_date = `file.frontmatter.date_completed`;

//---------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//---------------------------------------------------------
const status_sort = `choice(${yaml_status} = "undetermined" OR ${yaml_status} = "schedule", 
		1, 
		choice(${yaml_status} = "to_do", 
			2, 
			choice(${yaml_status} = "in_progress", 
				3, 
				choice(${yaml_status} = "done", 
					4, 
					5))))`;

//---------------------------------------------------------
// DATAVIEW TABLE FOR JOURNALS WRITTEN BETWEEN DATES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

// VAR FILE CLASS OPTIONS: "task", "pkm", "lib"
// VAR TASK TYPE: "project", "parent_task"
// VAR PKM TYPE: "permanent", "literature", "fleeting", "info"
// VAR LIB TYPE: ""

// VAR COMPLETED STATUS OPTIONS: "completed", "done"
// VAR CREATED STATUS OPTIONS: "created", "new"
// VAR DETERMINE STATUS OPTIONS: "undetermined", "determine", "schedule"
// VAR ACTIVE STATUS OPTIONS: "active", "to_do", "in_progress"

async function dv_class_type_status_start_end({
  file_class: file_class,
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
}) {
  const class_arg = `${file_class}`;
  const type_arg = `${type}`;
  const status_arg = `${status}`;

  let class_filter = `contains(file.frontmatter.file_class, "${class_arg}")`;
  let type_filter = `contains(file.frontmatter.type, "${type_arg}")`;
  let status_filter = `contains(file.frontmatter.status, "${status_arg}")`;
  let date_start_filter = `date(${creation_date}) >= date(${date_start})`;
  let date_end_filter = `date(${creation_date}) >= date(${date_end})`;

  let dataview_table;
  // >>>>> TASK TABLES <<<<<
  if (class_arg.startsWith("task")) {
    if (type_arg.startsWith("proj")) {
      if (
        status_arg.startsWith("act") ||
        status_arg.startsWith("to_do") ||
        status_arg.startsWith("in_pr")
      ) {
        // Table for ACTIVE PROJECTS
        dv_links_table = `${three_backtick}dataview
TABLE WITHOUT ID
    ${title_link} AS Project,
    ${file_status},
    ${date_span},
    ${context}
FROM
    ${source}
    AND ${proj_dir}
WHERE
    ${class_filter}
    AND ${type_filter}
    AND ${active_filter}
SORT
    ${yaml_date_end},
    file.frontmatter.title ASC
${three_backtick}`;
      } else if (
        status_arg.startsWith("done") ||
        status_arg.startsWith("comp")
      ) {
        date_start_filter = `date(${yaml_date_end}) >= date(${date_start})`;
        date_end_filter = `date(${yaml_date_end}) >= date(${date_end})`;
        // Table for COMPLETED PROJECTS
        dv_links_table = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Project,
	${file_status},
	${date_span},
	${context}, 
    ${org}
FROM
	${source}
    AND ${proj_dir}
WHERE
	${class_filter}
	AND ${type_filter}
	AND ${date_start_filter}
	AND ${date_end_filter}
SORT 
	file.frontmatter.title ASC
${three_backtick}`;
      }
    } else if (type_arg.startsWith(`par`)) {
      if (
        status_arg.startsWith("act") ||
        status_arg.startsWith("to_do") ||
        status_arg.startsWith("in_pr")
      ) {
        // Table for ACTIVE PARENT TASKS
        dv_links_table = `${three_backtick}dataview
TABLE WITHOUT ID
    ${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${context},
    ${project}
FROM
	${source}
    AND ${proj_dir}
WHERE
	${class_filter}
	AND ${type_filter}
	AND ${active_filter}
SORT 
    file.frontmatter.project,
    ${yaml_date_end},
    file.frontmatter.title ASC
${three_backtick}`;
      } else if (
        status_arg.startsWith("done") ||
        status_arg.startsWith("comp")
      ) {
        date_start_filter = `date(${yaml_date_end}) >= date(${date_start})`;
        date_end_filter = `date(${yaml_date_end}) >= date(${date_end})`;
        // Table for COMPLETED PARENT TASKS
        dv_links_table = `${three_backtick}dataview
TABLE WITHOUT ID
    ${title_link} AS "Parent Task",
    ${file_status},
    ${date_span},
    ${context},
    ${project}
FROM
    ${source}
    AND ${proj_dir}
WHERE
    ${class_filter}
    AND ${type_filter}
	AND ${date_start_filter}
	AND ${date_end_filter}
SORT 
    file.frontmatter.project,
    file.frontmatter.title ASC
${three_backtick}`;
      }
    }
    // >>>>> NOTE TABLES <<<<<
  } else if (class_arg.startsWith("pkm")) {
    if (
      type_arg.startsWith("perm") ||
      type_arg.startsWith("lit") ||
      type_arg.startsWith("fle")
    ) {
      // Table for PERMANENT, LITERATURE, OR FLEETING notes
      dataview_note_table = `${three_backtick}dataview
TABLE WITHOUT ID
    ${title_link},
    ${file_subtype}, 
    ${file_status},
    ${tags}
FROM
    ${source}
WHERE
    ${class_filter}
    AND ${type_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
SORT
    file.frontmatter.date_created,
    file.frontmatter.status ASC
${three_backtick}`;
    } else if (type_arg.startsWith("info")) {
      type_filter = `contains(file.frontmatter.file_class, "${type_arg}")`;
      // Table for INFO notes
      dataview_note_table = `${three_backtick}dataview
TABLE WITHOUT ID
    ${title_link},
    ${file_type},
    ${file_status} 
    ${tags}
FROM
    ${source}
WHERE
    ${class_filter}
    AND ${type_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
SORT
    file.frontmatter.type,
    file.frontmatter.date_created ASC
${three_backtick}`;
    } else if (
      status_arg.startsWith("rev") ||
      status_arg.startsWith("clar") ||
      status_arg.startsWith("dev")
    ) {
      dataview_note_table = `${three_backtick}dataview
TABLE WITHOUT ID
    ${title_link},
    ${file_type},
    ${file_subtype}, 
    ${tags}
FROM
    ${source}
WHERE
    ${class_filter}
    AND ${type_filter}
    AND ${status_filter}
SORT
    file.frontmatter.type,
    file.frontmatter.date_created ASC
${three_backtick}`;
    }
    // >>>>> LIBRARY TABLES <<<<<
  } else if (class_arg.startsWith("lib")) {
    if (status_arg.startsWith("done") || status_arg.startsWith("comp")) {
      date_start_filter = `date(${completion_date}) >= date(${date_start})`;
      date_end_filter = `date(${completion_date}) >= date(${date_end})`;
      // Table for library resources COMPLETED between dates
      dataview_library_table = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link},
	${file_status},
	${author},
	${date_publish},
	${file_type},
	${tags}
FROM
	${source}
	OR ${lib_dir}
	OR ${inbox}
WHERE
    ${class_filter}
	AND ${date_start_filter}
	AND ${date_end_filter}
SORT
	${status_sort},
	${completion_date} ASC
LIMIT 50
${three_backtick}`;
    } else if (status_arg.startsWith("creat") || status_arg.startsWith("new")) {
      date_start_filter = `date(${creation_date}) >= date(${date_start})`;
      date_end_filter = `date(${creation_date}) >= date(${date_end})`;
      // Table for library resources CREATED between dates
      dataview_library_table = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link},
	${file_status},
	${author},
	${date_publish},
	${file_type},
	${tags}
FROM
	${source}
	OR ${lib_dir}
	OR ${inbox}
WHERE
	${class_filter}
	AND ${date_start_filter}
	AND ${date_end_filter}
SORT
	${status_sort},
	file.frontmatter.type ASC
LIMIT 50
${three_backtick}`;
    } else if (
      status_arg.startsWith("sched") ||
      status_arg.startsWith("und") ||
      status_arg.startsWith("det")
    ) {
      // Table for library resources TO DETERMINE AND SCHEDULE
      dataview_library_table = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link},
	${file_status},
	${author},
	${date_publish},
	${file_type},
	${tags}
FROM
	${source}
	OR ${lib_dir}
	OR ${inbox}
WHERE
	${class_filter}
	AND ${undetermined_filter}
SORT
	${status_sort},
	file.frontmatter.date_created ASC
LIMIT 50
${three_backtick}`;
    } else if (
      status_arg.startsWith("act") ||
      status_arg.startsWith("to") ||
      status_arg.startsWith("in")
    ) {
      // Table for library resources TO DO OR IN PROGRESS
      dataview_library_table = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link},
	${file_status},
	${author},
	${date_publish},
	${file_type},
	${tags}
FROM
	${source}
	OR ${lib_dir}
	OR ${inbox}
WHERE
	${class_filter}
	AND ${active_filter}
SORT
	${status_sort},
	file.frontmatter.date_created ASC
LIMIT 50
${three_backtick}`;
    }
  }
  return dataview_table;
}

module.exports = dv_class_type_status_start_end;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATAVIEW TABLE BY CLASS, TYPE, STATUS, AND DATES
//---------------------------------------------------------
// VAR FILE CLASS OPTIONS: "task", "pkm", "lib"
// VAR TASK TYPE: "project", "parent_task"
// VAR PKM TYPE: "permanent", "literature", "fleeting", "info"
// VAR LIB TYPE: ""

// VAR COMPLETED STATUS OPTIONS: "completed", "done"
// VAR CREATED STATUS OPTIONS: "created", "new"
// VAR DETERMINE STATUS OPTIONS: "undetermined", "determine", "schedule"
// VAR ACTIVE STATUS OPTIONS: "active", "to_do", "in_progress"
const dv_note_table = await tp.user.dv_class_type_status_start_end({
  file_class: file_class,
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
});
```

#### Examples

```javascript
//---------------------------------------------------------
// WEEKLY CALENDAR TEMPLATE DATAVIEW TABLES
//---------------------------------------------------------
const week_note_perm = await tp.user.dv_class_type_status_start_end({
  file_class: "pkm",
  type: "permanent",
  status: "",
  start_date: date_start,
  end_date: date_end,
});
const week_note_lit = await tp.user.dv_class_type_status_start_end({
  file_class: "pkm",
  type: "literature",
  status: "",
  start_date: date_start,
  end_date: date_end,
});
const week_note_fleet = await tp.user.dv_class_type_status_start_end({
  file_class: "pkm",
  type: "fleeting",
  status: "",
  start_date: date_start,
  end_date: date_end,
});
const week_note_info = await tp.user.dv_class_type_status_start_end({
  file_class: "pkm",
  type: "info",
  status: "",
  start_date: date_start,
  end_date: date_end,
});
const note_review = await tp.user.dv_class_type_status_start_end({
  file_class: "pkm",
  type: "",
  status: "review",
  start_date: "",
  end_date: "",
});
const note_clarify = await tp.user.dv_class_type_status_start_end({
  file_class: "pkm",
  type: "",
  status: "clarify",
  start_date: "",
  end_date: "",
});
const note_develop = await tp.user.dv_class_type_status_start_end({
  file_class: "pkm",
  type: "",
  status: "develop",
  start_date: "",
  end_date: "",
});

const week_lib_done = await tp.user.dv_class_type_status_start_end({
  file_class: "lib",
  type: "",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});
const week_lib_new = await tp.user.dv_class_type_status_start_end({
  file_class: "lib",
  type: "",
  status: "new",
  start_date: date_start,
  end_date: date_end,
});
const week_lib_undetermined = await tp.user.dv_class_type_status_start_end({
  file_class: "lib",
  type: "",
  status: "determine",
  start_date: "",
  end_date: "",
});
const week_lib_active = await tp.user.dv_class_type_status_start_end({
  file_class: "lib",
  type: "",
  status: "active",
  start_date: "",
  end_date: "",
});
const active_proj = await tp.user.dv_class_type_status_start_end({
  file_class: "task",
  type: "project",
  status: "active",
  start_date: "",
  end_date: "",
});
const active_parent_task = await tp.user.dv_class_type_status_start_end({
  file_class: "task",
  type: "parent_task",
  status: "active",
  start_date: "",
  end_date: "",
});
const week_comp_proj = await tp.user.dv_class_type_status_start_end({
  file_class: "task",
  type: "project",
  status: "active",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});
const week_comp_parent_task = await tp.user.dv_class_type_status_start_end({
  file_class: "task",
  type: "parent_task",
  status: "done",
  start_date: date_start,
  end_date: date_end,
});
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
2. [[32_00_week|Weekly Calendar Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->  
1.[[dv_class_type_status_start_end.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_week_journal|Journals and Attributes Between Dates Dataview Table]]
2. [[dv_day_class_type_file|Dataview Table by File Class, Type, and Date]]

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
