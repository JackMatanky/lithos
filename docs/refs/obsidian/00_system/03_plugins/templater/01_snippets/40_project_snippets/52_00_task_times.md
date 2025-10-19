---
title: 52_00_task_times
aliases:
  - Task Start, Reminder, Duration, and End Times
  - Task Times
plugin: templater
language:
  - javascript
module:
  - user
  - momentjs
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T15:52
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs
---
# Task Start, Reminder, Duration, and End Times

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Set the start, reminder, duration, and end for tasks and events.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET TASK START AND REMINDER TIME
//---------------------------------------------------------
const date = moment(<full_date_time>).format(`YYYY-MM-DD`);
const start_time = moment(<full_date_time>).format(`HH:mm`);
const reminder_date = moment(<full_date_time>)
  .subtract(10, `minutes`)
  .format(`YYYY-MM-DD HH:mm`);

//---------------------------------------------------------
// SET TASK DURATION AND END TIME
//---------------------------------------------------------
const duration_min = await tp.user.durationMin(tp);
const full_end_date = moment(<full_date_time>).add(
  Number(duration_min),
  `minutes`
);
const end_time = moment(full_end_date).format(`HH:mm`);
const duration_est = moment
  .duration(full_end_date.diff(<full_date_time>))
  .as(`minutes`);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET TASK START AND REMINDER TIME
//---------------------------------------------------------
const date = moment(<full_date_time>).format(`YYYY-MM-DD`);
const start_time = moment(<full_date_time>).format(`HH:mm`);
const reminder_date = moment(<full_date_time>)
  .subtract(10, `minutes`)
  .format(`YYYY-MM-DD HH:mm`);

//---------------------------------------------------------
// SET TASK DURATION AND END TIME
//---------------------------------------------------------
const duration_min = await tp.user.duration_min(tp);
const full_end_date = moment(<full_date_time>).add(
  Number(duration_min),
  `minutes`
);
const end_time = moment(full_end_date).format(`HH:mm`);
const duration_est = moment
  .duration(full_end_date.diff(<full_date_time>))
  .as(`minutes`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

#### Files

<!-- Files containing the snippet  -->

1. [[52_00_task_event]]
2. [[53_00_action_item]]
3. [[54_00_meeting]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[nl_date_and_time|Natural Language Date and Time]]
2. [[duration_min|Duration Minutes Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

### Incoming Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	file.frontmatter.description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND contains(file.outlinks, this.file.link)
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### Incoming Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.definition AS Definition
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND contains(file.outlinks, this.file.link)
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
