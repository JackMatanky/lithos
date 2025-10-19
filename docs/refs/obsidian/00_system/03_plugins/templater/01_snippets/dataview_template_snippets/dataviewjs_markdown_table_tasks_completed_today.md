---
title: dataviewjs_markdown_table_tasks_completed_today
aliases:
  - DataviewJS Markdown Table of Tasks Completed Today
  - Tasks Completed Today DataviewJS Markdown Table
plugin: templater
language:
  - javascript
module:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-06T15:39
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/dataviewjs, obsidian/dataview, markdown/table
---
# DataviewJS Markdown Table of Tasks Completed Today

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Render a markdown table of the day's completed tasks.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
const dv = app.plugins.plugins[`dataview`].api;

const today = moment().format(`YYYY-MM-DD`);

// regex for task tag, task type, and inline field
const task_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;

// Field variables
const date_start = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)`;
const date_end = `date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)`;
const task_duration = `dur(${date_end} - ${date_start})`;
const task_estimate = `dur(T.duration_est + " minutes")`;

const query = `TABLE WITHOUT ID
	regexreplace(regexreplace(T.text, ${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""), "_$", "") AS Task,
	choice(contains(T.text, "_action_item"),
		"Action Item",
		choice(contains(T.text, "_meeting"),
			"Meeting",
			choice(contains(T.text, "_habit"),
				"Habit",
				choice(contains(T.text, "_morning_ritual"),
					"Morning Rit.",
					choice(contains(T.text, "_workday_startup_ritual"),
						"Workday Startup Rit.",
						choice(contains(T.text, "_workday_shutdown_ritual"),
							"Workday Shutdown Rit.",
							"Evening Rit.")))))) AS Type,
	T.time_start AS Start,
	T.time_end AS End,
	(T.duration_est + " min") AS Estimate,
	choice((${task_estimate} = ${task_duration}),
		"👍On Time",
		choice(
			(${task_estimate} > ${task_duration}),
				"🟢" + (${task_estimate} - ${task_duration}),
				"❗" + (${task_duration} - ${task_estimate}))) AS Accuracy,
	file.frontmatter.project AS Project
FROM
	#task
	AND -"00_system/05_templates"
FLATTEN
	file.tasks AS T
WHERE
	any(file.tasks, (t) =>
		t.completion = date(${today}))
SORT
	T.time_start ASC`

const markdown = await dv.queryMarkdown(query);
tR += markdown.value
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript

```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[00_system/04_templates/111_43_dvmd_day_tasks_done]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dataviewjs_markdown_table_tasks_due_today|DataviewJS Markdown Table of Tasks Due Today]]

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
[[DataviewJS dv.queryMarkdown Function|The DataviewJS dv.queryMarkdown() Function]]

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

["Burning out" dataviews · Issue #42 · blacksmithgu/obsidian-dataview (github.com)](https://github.com/blacksmithgu/obsidian-dataview/issues/42)

<https://github.com/blacksmithgu/obsidian-dataview/issues/42#issuecomment-1207951602>

---

## Flashcards
