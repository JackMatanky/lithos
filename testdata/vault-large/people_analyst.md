---
title: people_analyst
uuid: cac517f5-44d2-4c23-ba91-09b0741dda82
aliases:
  - People Analyst
  - people_analyst
status: active
type: professional
file_class: pillar
date_created: 2023-05-21T16:10
date_modified: 2023-09-05T21:00
tags:
---
# People Analyst

---

## Knowledge

```dataview
LIST
	rows.file.link
FROM -"00_system/05_templates"
WHERE
	((file.frontmatter.pillar) = "people_analyst")
	AND (contains(file.frontmatter.file_class, "pkm"))
GROUP BY file.frontmatter.file_class
```

## Journal Entries

```dataview
LIST
FROM -"00_system/05_templates"
WHERE
	(file.frontmatter.pillar = "people_analyst")
	AND (contains(file.frontmatter.file_class, "journal"))
```

## Tasks and Events

```dataview
TABLE WITHOUT ID
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)", "") AS Task,
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "^[A-Za-z0-9\'\-\s]*_", "") AS Type,
	T.completion AS Completed,
	T.time_start AS Start,
	T.time_end AS End,
	(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) -
	date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)) AS Duration,
	T.section AS Link
FROM -"00_system/05_templates" AND #task
FLATTEN file.tasks AS T
WHERE any(file.frontmatter.pillar = "people_analyst")
SORT T.time_start ASC
```
