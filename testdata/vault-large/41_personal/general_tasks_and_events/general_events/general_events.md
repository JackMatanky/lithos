---
title: general_events
uuid: 38520601-a8a0-41f3-9693-629e5cba4e92
aliases:
  - General Events
  - general events
  - general_events
task_start:
task_end:
due_do: do
pillar:
context: personal
goal: null
project:
  - "[[general_tasks_and_events|General Tasks and Events]]"
organization:
contact:
library:
status: in_progress
type: parent_task
file_class: task_parent
date_created: 2023-06-28T22:39
date_modified: 2024-05-31T14:34
tags:
---
# General Events

> [!parent_task] Parent Task Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Goal**: `dv: this.file.frontmatter.goal`
> - **Project**: `dv: this.file.frontmatter.project`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) =>!contains(lower(x), "null")), " | ")`
>
> - **Dates**: `dv: join([this.file.frontmatter.task_start, this.file.frontmatter.task_end], " - ")`

---

## Prepare and Reflect

### Preview

> [!task_preview] Parent Task Preview
>
> What is the problem to solve?
> 1. need_case::
>
> What are possible solutions?
> 1. solution::

### Review

> [!task_review] Parent Task Review
>
> What was the outcome?
> 1. outcome::
>
> What went well?
> 1. keep::
>
> What can be improved?
> 1. improve::
>
> What can be started?
> 1. start::
>
> What can be stopped?
> 1. stop::

---

## Tasks and Events

### Remaining Tasks

```dataview
TABLE WITHOUT ID
	link(T.section,
		regexreplace(
			regexreplace(T.text, "(#task)|(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\s*\[.*$", ""),
		"_$", ""))
	AS Task,
	choice(contains(T.text, "_action_item"),	"🔨Task",
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
	AS Type,
	T.due AS Due,
	T.time_start AS Start,
	T.time_end AS End,
	(T.duration_est + " min") AS Estimate,
	file.frontmatter.parent_task AS "Parent Task"
FROM
	-"00_system/05_templates"
FLATTEN
	file.tasks AS T
WHERE
	contains(file.path, this.file.folder)
	AND regextest("(#task)", T.text)
	AND T.status != "-"
	AND !T.completed
SORT
	T.due,
	T.time_start ASC
```

### Completed Tasks

```dataview
TABLE WITHOUT ID
	link(T.section,
		regexreplace(
			regexreplace(T.text, "(#task)|(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\s*\[.*$", ""),
		"_$", ""))
	AS Task,
	choice(contains(T.text, "_action_item"),	"🔨Task",
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
	AS Type,
	T.completion AS Date,
	(T.time_start + " - " + T.time_end) AS Time,
	(T.duration_est + " min") AS Estimate,
	choice((dur(T.duration_est + " minutes") = dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)))),
	"👍On Time",
	choice(
		(dur(T.duration_est + " minutes") > dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)))),
			"🟢" + (dur(T.duration_est + " minutes") - dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)))),
			"❗" + (dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start))) - dur(T.duration_est + " minutes"))))
AS Accuracy,
	file.frontmatter.parent_task AS "Parent Task"
FROM
	-"00_system/05_templates"
FLATTEN
	file.tasks AS T
WHERE
	contains(file.path, this.file.folder)
	AND regextest("(#task)", T.text)
	AND T.status != "-"
	AND T.completed
SORT
	T.completion,
	T.time_start ASC
```

---

## Related Tasks and Events

### Outgoing Task and Events Links

<!-- Link related tasks and events here -->

### Projects

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Project,
	file.frontmatter.status AS Status,
	choice((regextest(".", file.frontmatter.task_start) AND regextest(".", file.frontmatter.task_end)),
		(file.frontmatter.task_start + " → " + file.frontmatter.task_end),
		choice(regextest(".", file.frontmatter.task_start),
			(file.frontmatter.task_start + " → Present"),
			"null"))
	AS Dates,
	file.frontmatter.context AS Context
FROM
	-"00_system/05_templates"
WHERE
	file.name != this.file.name
	AND !contains(file.path, this.file.folder)
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "task")
	AND contains(file.frontmatter.type, "project")
SORT
	file.frontmatter.title ASC
```

### Parent Tasks

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS "Parent Task",
	file.frontmatter.status AS Status,
	choice((regextest(".", file.frontmatter.task_start) AND regextest(".", file.frontmatter.task_end)),
		(file.frontmatter.task_start + " → " + file.frontmatter.task_end),
		choice(regextest(".", file.frontmatter.task_start),
			(file.frontmatter.task_start + " → Present"),
			"null"))
	AS Dates,
	file.frontmatter.context AS Context,
	file.frontmatter.project AS Project
FROM
	-"00_system/05_templates"
WHERE
	file.name != this.file.name
	AND !contains(file.path, this.file.folder)
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "task")
	AND contains(file.frontmatter.type, "parent")
SORT
	file.frontmatter.title ASC
```

### Child Tasks

```dataview
TABLE WITHOUT ID
	link(T.section,
		regexreplace(
			regexreplace(T.text, "(#task)|(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\s*\[.*$", ""),
		"_$", ""))
	AS Task,
	choice(contains(T.text, "_action_item"),	"🔨Task",
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
	AS Type,
	choice((T.status != "-"),
		(choice((T.status = "x"),
			"✔️Done",
			"🔜To do")),
		"❌Discard")
	AS Status,
	choice((T.status != "-"),
		(choice((T.status = "x"),
			T.completion,
			T.due)),
		"❌Discard")
	AS Date,
	file.frontmatter.project AS Project
FROM
	-"00_system/05_templates"
FLATTEN
	file.tasks AS T
WHERE
	file.name != this.file.name
	AND !contains(file.path, this.file.folder)
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "task")
	AND (contains(file.frontmatter.file_class, "action_item")
	OR contains(file.frontmatter.file_class, "meeting"))
	AND regextest("(#task)", T.text)
SORT
	T.due ASC
```

---

## Related Notes

### Outgoing Note Links

<!-- Link related notes here -->

### Permanent

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	file.frontmatter.subtype AS Subtype,
	file.frontmatter.status AS Status,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "perm")
SORT
	file.frontmatter.title ASC
```

### Literature

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	file.frontmatter.subtype AS Subtype,
	file.frontmatter.status AS Status,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "lit")
SORT
	file.frontmatter.title ASC
```

### Fleeting

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	file.frontmatter.subtype AS Subtype,
	file.frontmatter.status AS Status,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "fleet")
SORT
	file.frontmatter.title ASC
```

### General

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	file.frontmatter.type AS Type,
	file.frontmatter.status AS Status,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.file_class, "info")
SORT
	file.frontmatter.type,
	file.frontmatter.title ASC
```

---

## Related Directory

### Outgoing Contact Links

<!-- Link related contacts here -->

### Contacts

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Name,
	file.frontmatter.job_title AS "Job Title",
	file.frontmatter.organization AS "Org.",
	file.etags AS Tags
FROM
	"51_contacts"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "dir")
	AND contains(file.frontmatter.type, "contact")
SORT
	file.frontmatter.title ASC
```

### Outgoing Organization Links

<!-- Link related organizations here -->

### Organizations

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Name,
	file.frontmatter.website AS Website,
	file.frontmatter.linkedin AS LinkedIn,
	file.frontmatter.about AS About,
	file.etags AS Tags
FROM
	"52_organizations"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "dir")
	AND contains(file.frontmatter.type, "organization")
SORT
	file.frontmatter.title ASC
```

---

## Related Resources

### Outgoing Resource Links

<!-- Link related resources here -->

### Library

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	file.frontmatter.author AS Author,
	choice(contains(file.frontmatter.type, "book"), file.frontmatter.year_published, file.frontmatter.date_published) AS "Date Published",
	file.frontmatter.type AS Type,
	file.frontmatter.status AS Status,
	file.etags AS Tags
FROM
	-"00_system/05_templates"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "lib")
SORT
	file.frontmatter.type,
	file.frontmatter.title ASC
```
