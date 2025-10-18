---
title: dataviewjs_markdown_table_tasks_due_today
aliases:
  - DataviewJS Markdown Table of Tasks Due Today
  - Tasks Due Today DataviewJS Markdown Table
plugin: templater, dataview
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
# DataviewJS Markdown Table of Tasks Due Today

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]], [[Dataview]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Render a markdown table of tasks due that day.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// DATAVIEW API
const dv = app.plugins.plugins[`dataview`].api;

// Today's date
const date = moment().format(`YYYY-MM-DD`);

//---------------------------------------------------------
// TASK DATA FIELD VARIABLES
//---------------------------------------------------------
// regex for task tag, task type, and inline field
const task_tag_regex = `(#task)`;
const task_type_regex = `(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*`;
const inline_field_regex = `\\[.*$`;

// Task name
const task_name = `regexreplace(regexreplace(T.text, "${task_tag_regex}|${task_type_regex}${inline_field_regex}", ""), "_$", "")) AS Task`;

// Task type
const task_type = `choice(contains(T.text, "_action_item"),	"🔨Task", choice(contains(T.text, "_meeting"), "🤝Meeting", choice(contains(T.text, "_phone_call"), "📞Call", choice(contains(T.text, "_interview"), "💼Interview", choice(contains(T.text, "_appointment"), "⚕️Appointment", choice(contains(T.text, "_event"), "🎊Event", choice(contains(T.text, "_gathering"), "✉️Gathering", choice(contains(T.text, "_hangout"), "🍻Hangout", choice(contains(T.text, "_habit"), "🤖Habit", choice(contains(T.text, "_morning_ritual"),	"🍵Rit.", choice(contains(T.text, "_workday_startup_ritual"), "🌇Rit.", choice(contains(T.text, "_workday_shutdown_ritual"), "🌆Rit.", "🛌Rit.")))))))))))) AS Type`;

// Times
const start = `T.time_start`;
const end = `T.time_end`;

// Time estimate
const task_estimate = `(T.duration_est + " min") AS Estimate`;

// Task project
const project = `file.frontmatter.project AS Project`;

//---------------------------------------------------------
// TASK DATA FILTER VARIABLES
//---------------------------------------------------------
// Task checkbox
const task_checkbox = `regextest("${task_tag_regex}", T.text)`;

// Discarded status filter
const discard = `T.status != "-"`;

//---------------------------------------------------------
// DAILY PREVIEW DATAVIEW TABLE QUERY
//---------------------------------------------------------
const query = `TABLE WITHOUT ID	regexreplace(regexreplace(T.text, '(#task)|(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\\s*\\[.*$', ''), '_$', '')) AS Task, choice(contains(T.text, '_action_item'),	'🔨Task', choice(contains(T.text, '_meeting'), '🤝Meeting', choice(contains(T.text, '_phone_call'), '📞Call', choice(contains(T.text, '_interview'), '💼Interview', choice(contains(T.text, '_appointment'), '⚕️Appointment', choice(contains(T.text, '_event'), '🎊Event', choice(contains(T.text, '_gathering'), '✉️Gathering', choice(contains(T.text, '_hangout'), '🍻Hangout', choice(contains(T.text, '_habit'), '🤖Habit', choice(contains(T.text, '_morning_ritual'),	'🍵Rit.', choice(contains(T.text, '_workday_startup_ritual'), '🌇Rit.', choice(contains(T.text, '_workday_shutdown_ritual'), '🌆Rit.', '🛌Rit.')))))))))))) AS Type, T.time_start AS Start, T.time_end AS End, (T.duration_est + ' min') AS Estimate, file.frontmatter.project AS Project FROM #task AND -"00_system/05_templates" FLATTEN file.tasks AS T WHERE regextest('(#task)', T.text) AND T.status != '-' AND T.due = date(${date}) SORT T.time_start ASC`;

const markdown = await dv.queryMarkdown(query);
tR += markdown.value
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
const query = `TABLE WITHOUT ID ${task_name}, ${task_type}, ${start} AS Start, ${end} AS End, ${task_estimate}, ${project} FROM #task AND -'00_system/05_templates' FLATTEN file.tasks AS T WHERE ${task_checkbox} AND ${discard} AND T.due = date(${date}) SORT T.time_start ASC`;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[111_40_dvmd_day_tasks]]
2. [[55_21_daily_morn_rit]]
3. [[55_22_today_morn_rit]]
4. [[55_23_tomorrow_morn_rit]]
5. [[53_24_daily_morn_rit_quickadd]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dataviewjs_markdown_table_tasks_completed_today|DataviewJS Markdown Table of Tasks Completed Today]]

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

Dateview GitHub Issue 42: ["Burning out" dataviews](https://github.com/blacksmithgu/obsidian-dataview/issues/42)

Initial reference for rendering a dataview query in markdown: <https://github.com/blacksmithgu/obsidian-dataview/issues/42#issuecomment-1207951602>

### Possible Errors

#### Rendering a Dataview Query in Markdown Returning ["undefined"](https://github.com/blacksmithgu/obsidian-dataview/issues/42#issuecomment-1446812429)

I have (what I hope is) a simple/newbie question. I've been using the Templater + Dataview strategies in this thread to "burn in" query results to various periodic notes I use in my workflow. It's been working great, but I've hit a problem that I can't figure out the answer to.

I'm using this code successfully:

```js
const dv = app.plugins.plugins["dataview"].api;
const weekGoals = await dv.queryMarkdown(`LIST rows.L.text FROM "Projects" FLATTEN file.lists as L WHERE contains(L.tags, "#2023W9") GROUP BY file.link SORT L.context`);
tR += weekGoals.value;
```

It grabs all the list items tagged \#2023W9 from notes in my Projects folder and writes a list of them sorted by their context in my weekly note. Works great. But as I'm using it with Templater, I'd like that tag to change each week - and that's what I cannot figure out how to make happen.

I've tried this:

```js
const dv = app.plugins.plugins["dataview"].api;
const weekTag = "#" + tp.file.title.split(" ")[0] + tp.file.title.split(" ")[1];
const weekGoals = await dv.queryMarkdown(`LIST rows.L.text FROM Projects FLATTEN file.lists as L WHERE contains(L.tags, "${weekTag}") GROUP BY file.link SORT L.context`);
tR += weekGoals.value;
```

But it just returns "undefined". I've checked that weekTag is indeed creating the correct tag (#2023W9). But it's not generating the same output as when the tag itself is there in the contains() part. My weekly notes are named like "2023 W9", for reference.

Have I got a syntax problem in here somewhere? Is there a better/different/other way to pass that weekTag info dynamically into the template? Can anyone take a look and give me a hand with this? I'm kind of at wits end. Thanks!

#### Response to Returning ["undefined"](https://github.com/blacksmithgu/obsidian-dataview/issues/42#issuecomment-1458403896)

I was able to get your code to work with

```js
	const dv = app.plugins.plugins["dataview"].api;
	const now = moment()
	const year = now.format('YYYY')
	const week = now.format('w')
	let tag = "#"+year+"W"+week
	const query = `LIST rows.L.text FROM "Projects" FLATTEN file.lists as L WHERE contains(L.tags, "${tag}") GROUP BY file.link SORT L.context`
	let out = await dv.queryMarkdown(query)
	tR += out.value
```

During my testing ==***templater+dataview seemed really finicky about double quotes.***== So it could have been changing Projects to”Projects”.

Anyway, I ended up using moment to get the year format and week format. `(moment().format(‘w’) == “9”` and `moment().format(‘ww’) == “09”).

Its just how I like to get dates generated for any templates that depend on time. Mine are usually generated automatically based on the actual day or week. If you’re manually generating a bunch of notes for past/future dates, then you should switch back to using the title 👍

---

## Flashcards
