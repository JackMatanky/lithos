---
title: dv_linked_file
aliases:
  - Linked File Dataview Table
  - Dataview Table for Linked Files
  - dv linked file
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-19T18:17
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Linked File Dataview Table

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a dataview table for linked files based on file class and type

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATA FIELDS
//---------------------------------------------------------
// SECT: >>>>> GENERAL <<<<<
// file title name and link
const title_link = `link(file.link, file.frontmatter.aliases[0])`;

// File type
const file_type = `file.frontmatter.type AS Type`;

// File subtype
const file_subtype = `file.frontmatter.subtype AS Subtype`;

// Status
const status = `file.frontmatter.status AS Status`;

// Tags
const tags = `file.etags AS Tags`;

// SECT: >>>>> LIBRARY <<<<<
// Author
const author = `file.frontmatter.author AS Author`;

// Date published
const date_publish = `choice(contains(file.frontmatter.type, "book"), file.frontmatter.year_published, file.frontmatter.date_published) AS "Date Published"`;

// SECT: >>>>> DIRECTORY <<<<<
const job_title = `file.frontmatter.job_title AS "Job Title"`;

const website = `file.frontmatter.website AS Website`;

const linkedin = `file.frontmatter.linkedin AS LinkedIn`;

const org_about = `file.frontmatter.about AS About`;

// SECT: >>>>> TASK <<<<<
// regex for task tag, task type, and inline field
const task_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;

// Task name and link
const task_link = `link(T.section,
		regexreplace(
			regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""),
		"_$", "")) 
	AS Task`;

// Task type
const task_type = `choice(contains(T.text, "_action_item"),	"🔨Task",
	choice(contains(T.text, "_meeting"), "🤝Meeting",
	choice(contains(T.text, "_phone_call"), "📞Call",
	choice(contains(T.text, "_interview"), "💼Interview",
	choice(contains(T.text, "_appointment"), "⚕️Appointment",
	choice(contains(T.text, "_event"), "🎊Event",
	choice(contains(T.text, "_gathering"), "✉️Gathering",
	choice(contains(T.text, "_hangout"), "🍻Hangout",
	choice(contains(T.text, "_habit"), "🤖Habit",
	choice(contains(T.text, "_morning_ritual"),	"🍵Rit.",
	choice(contains(T.text, "_workday_startup_ritual"), "🌇Rit.",
	choice(contains(T.text, "_workday_shutdown_ritual"), "🌆Rit.", "🛌Rit.")))))))))))) 
	AS Type`;

// Task status
const task_status = `choice((T.status != "-"), 
		(choice((T.status = "x"),
			"✔️Done",
			"🔜To do")),
		"❌Discard")
	AS Status`;

// Due or completed date
const due_date = `T.due AS Due`;
const done_date = `T.completion AS Date`;
const task_date = `choice((T.status != "-"), 
		(choice((T.status = "x"),
			T.completion,
			T.due)),
		"❌Discard")
	AS Date`;

// Time span
const start = `T.time_start`;
const end = `T.time_end`;
const time_span = `(${start} + " - " + ${end}) AS Time`;

// Date span
const date_start = `file.frontmatter.task_start`;
const date_end = `file.frontmatter.task_end`;
const date_span = `choice((regextest(".", ${date_start}) AND regextest(".", ${date_end})), 
		(${date_start} + " → " + ${date_end}),
		choice(regextest(".", ${date_start}),
			(${date_start} + " → Present"),
			"null")) 
	AS Dates`;

// Time estimate
const task_estimate = `(T.duration_est + " min") AS Estimate`;

// Time duration
const task_duration = `dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)))`;
const task_estimate_dur = `dur(T.duration_est + " minutes")`;
const task_estimate_accuracy = `choice((${task_estimate_dur} = ${task_duration}),
		"👍On Time",
		choice(
			(${task_estimate_dur} > ${task_duration}),
				"🟢" + (${task_estimate_dur} - ${task_duration}),
				"❗" + (${task_duration} - ${task_estimate_dur}))) 
	AS Accuracy`;

// Task context
const context = `file.frontmatter.context AS Context`;

// Task project
const project = `file.frontmatter.project AS Project`;

// Parent Task field
const parent_task = `file.frontmatter.parent_task AS "Parent Task"`;

// Organization
const org = `file.frontmatter.organization AS "Org."`;

// Contact
const contact = `file.frontmatter.contact AS "Contact"`;

//---------------------------------------------------------
// TASK DATA SOURCE
//---------------------------------------------------------
// Template directory
const template_dir = `-"00_system/05_templates"`;

//---------------------------------------------------------
// DATA FILTER
//---------------------------------------------------------
// SECT: >>>>> GENERAL <<<<
// Current file filter
const current_file_filter = `file.name != this.file.name`;

// Same directory
const folder_filter = `!contains(file.path, this.file.folder)`;

// File inlinks and outlinks
const in_out_link_filter = `(contains(file.outlinks, this.file.link) 
	OR contains(file.inlinks, this.file.link))`;

// SECT: >>>>> TASK <<<<<
// Task checkbox
const task_checkbox_filter = `regextest("${task_tag_regex}", T.text)`;

// Discarded status filter
const discard_filter = `T.status != "-"`;

// Task completion filter
const status_filter = `T.completed`;

//---------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

// VAR FILE CLASS OPTIONS: "task", "pkm", "lib", "dir"
// VAR TASK TYPE: "project", "parent_task", "child_task"
// VAR PKM TYPE: "permanent", "literature", "fleeting", "info"
// VAR LIB TYPE:
// VAR DIR TYPE: "contact", "organization"

async function dv_linked_file(file_class, type) {
  const class_arg = `${file_class}`;
  const type_arg = `${type}`;

  const class_filter = `contains(file.frontmatter.file_class, "${file_class}")`;
  let type_filter = `contains(file.frontmatter.type, "${type}")`;

  let dataview_query;
  if (class_arg == `task`) {
    if (type_arg.startsWith(`proj`)) {
      // Table for linked PROJECTS
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Project,
	${status},
	${date_span},
	${context}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${folder_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	file.frontmatter.title ASC
${three_backtick}`;
    } else if (type_arg.startsWith(`par`)) {
      // Table for linked PARENT TASKS
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS "Parent Task",
	${status},
	${date_span},
	${context},
	${project}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${folder_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	file.frontmatter.title ASC
${three_backtick}`;
    } else {
      // Table for linked PARENT TASKS
      type_filter = `(contains(file.frontmatter.file_class, "action_item") 
	OR contains(file.frontmatter.file_class, "meeting"))`;
      // Table for linked ACTION ITEMS AND MEETINGS
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${task_link},
	${task_type},
	${task_status},
	${task_date},
	${project}
FROM
	${template_dir}
FLATTEN
	file.tasks AS T
WHERE
	${current_file_filter}
	AND ${folder_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${task_checkbox_filter}
SORT 
	T.due ASC
${three_backtick}`;
    }
  } else if (class_arg == `pkm`) {
    if (
      type_arg.startsWith(`perm`) ||
      type_arg.startsWith(`lit`) ||
      type_arg.startsWith(`fleet`)
    ) {
      // Table for linked PERMANENT, LITERATURE, AND FLEETING NOTES
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Title,
	${file_subtype},
	${status},
	${tags}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	file.frontmatter.title ASC
${three_backtick}`;
    } else {
      // Table for linked CONCEPT, DEFINITION, AND GENERAL NOTES
      type_filter = `contains(file.frontmatter.file_class, "info")`;
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Title,
	${file_type},
	${status},
	${tags}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	file.frontmatter.type,
	file.frontmatter.title ASC
${three_backtick}`;
    }
  } else if (class_arg == `lib`) {
    if (type_arg != "") {
      type_filter = `(contains(file.frontmatter.file_class, "${type}")`;
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Title,
	${author},
	${date_publish},
	${file_type},
	${status},
	${tags}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	file.frontmatter.title ASC
${three_backtick}`;
    } else {
      // Table for linked LIBRARY content
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Title,
	${author},
	${date_publish},
	${file_type},
	${status},
	${tags}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
SORT 
	file.frontmatter.type,
	file.frontmatter.title ASC
${three_backtick}`;
    }
  } else if (class_arg == `dir`) {
    if (type_arg.startsWith(`cont`)) {
      // Table for linked CONTACTS
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Name,
	${job_title},
	${org},
	${tags}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	file.frontmatter.title ASC
${three_backtick}`;
    } else {
      // Table for linked ORGANIZATIONS
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Name,
	${website},
	${linkedin},
	${org_about},
	${tags}
FROM
	${template_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	file.frontmatter.title ASC
${three_backtick}`;
    }
  }
  return dataview_query;
}

module.exports = dv_linked_file;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// LINKED FILES DATAVIEW TABLE
//---------------------------------------------------------
// FILE CLASS OPTIONS: "task", "pkm", "lib", "dir"
// TASK TYPE: "project", "parent_task", "child_task"
// PKM TYPE: "permanent", "literature", "fleeting", "info"
// LIB TYPE:
// DIR TYPE: "contact", "organization"
const linked_file_table = await tp.user.dv_linked_file(file_class, type)
```

#### Examples

```javascript
//---------------------------------------------------------
// LINKED FILES DATAVIEW TABLE
//---------------------------------------------------------
// RELATED TASK TABLES
// TASK TYPE: "project", "parent_task", "child_task"
const linked_projects = await tp.user.dv_linked_file("task", "project");
const linked_parent_tasks = await tp.user.dv_linked_file("task", "parent");
const linked_child_tasks = await tp.user.dv_linked_file("task", "child");

// RELATED NOTE TABLES
// PKM TYPE: "permanent", "literature", "fleeting", "info"
const linked_note_permanent = await tp.user.dv_linked_file("pkm", "perm");
const linked_note_lit = await tp.user.dv_linked_file("pkm", "lit");
const linked_note_fleet = await tp.user.dv_linked_file("pkm", "fleet");
const linked_note_info = await tp.user.dv_linked_file("pkm", "info");

// RELATED LIBRARY TABLES
const linked_lib_content = await tp.user.dv_linked_file("lib", "");

// RELATED DIRECTORY TABLES
// DIR TYPE: "contact", "organization"
const linked_dir_contact = await tp.user.dv_linked_file("dir", "contact");
const linked_dir_org = await tp.user.dv_linked_file("dir", "organization");
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_00_project|General Project Template]]
2. [[51_00_parent_task|General Parent Task Template]]
3. [[90_00_note|General Note Template]]
4. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
5. [[90_11_note_quote|Quote Fleeting Note Template]]
6. [[90_12_note_idea|Idea Fleeting Note Template]]
7. [[90_20_note_literature(X)|General Literature Note Template]]
8. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
9. [[90_31_note_question|QEC Question Note Template]]
10. [[90_32_note_evidence|QEC Evidence Note Template]]
11. [[90_33_note_conclusion|QEC Conclusion Note Template]]
12. [[90_40_note_lit_psa(X)|PSA Note Template]]
13. [[90_41_note_problem|PSA Problem Note Template]]
14. [[90_42_note_steps|PSA Steps Note Template]]
15. [[90_43_note_answer|PSA Answer Note Template]]
16. [[90_50_note_info(X)|General Info Note Template]]
17. [[90_51_note_concept|Concept Note Template]]
18. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->  

1. [[dv_linked_file.js.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_task_linked|Linked Tasks and Events Files Dataview Table]]
2. [[dv_dir_linked|Linked Directory Files Dataview Table]]
3. [[dv_lib_linked|Linked Library Files Dataview Table]]
4. [[dv_pkm_linked|Linked Personal Knowledge Files Dataview Table]]

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
