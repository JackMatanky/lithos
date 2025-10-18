---
title: proj_task_completed_dataview_table
aliases:
  - Completed Project Tasks Dataview Table
  - Dataview Table of Completed Project Tasks
  - dataview_table_completed_project_tasks
plugin:
  - templater
  - dataview
language:
  - javascript
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:23
tags: obsidian/templater, obsidian/dataview, javascript
---

tags: #obsidian/templater #obsidian/dataview #javascript

# Completed Project Tasks Dataview Table

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]], [[Dataview]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description::

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>DEFINE <directory> VARIABLE

// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

//---------------------------------------------------------
// COMPLETED TASKS DATAVIEW TABLE
//---------------------------------------------------------
// regex tags for completed and remaining task code blocks
const task_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;
const task_title_regex = `^[A-Za-z0-9;:\\'\\s\\-]*_`;

// Field variables
const date_start = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)`;
const date_end = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)`;
const task_duration = `dur(${date_end} - ${date_start})`;
const task_estimate = `dur(T.duration_est + " minutes")`;
  
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
	file.frontmatter.project AS Project
FROM 
	#task 
	AND "${directory}"
FLATTEN 
	file.tasks AS T
WHERE 
	any(file.tasks, (t) => 
		t.completed)
		AND T.status != "-")
SORT 
	T.completion,
	T.time_start ASC
${three_backtick}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

//---------------------------------------------------------
// COMPLETED TASKS DATAVIEW TABLE
//---------------------------------------------------------
// regex tags for completed and remaining task code blocks
const task_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;
const task_title_regex = `^[A-Za-z0-9;:\\'\\s\\-]*_`;

// Field variables
const date_start = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)`;
const date_end = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)`;
const task_duration = `dur(${date_end} - ${date_start})`;
const task_estimate = `dur(T.duration_est + " minutes")`;
  
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
	file.frontmatter.project AS Project
FROM 
	#task 
	AND ${directory}
FLATTEN 
	file.tasks AS T
WHERE 
	any(file.tasks, (t) => 
		t.completed)
		AND T.status != "-")
SORT 
	T.completion,
	T.time_start ASC
${three_backtick}`;
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
