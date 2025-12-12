---
title: brother_uncle
uuid: 49d9fc47-5a2b-4c97-afa1-e7a66ac57a42
aliases:
  - Brother and Uncle
  - brother and uncle
status: active
type: interpersonal
file_class: pillar
date_created: 2023-05-21T16:02
date_modified: 2023-09-05T21:00
tags:
---
# Brother and Uncle

---

## Knowledge

```dataview
LIST
	rows.file.link
FROM -"00_system/05_templates"
WHERE
	((file.frontmatter.pillar) = "brother_uncle")
	AND (contains(file.frontmatter.file_class, "pkm"))
GROUP BY file.frontmatter.file_class
```

## Journal Entries

```dataview
LIST
FROM -"00_system/05_templates"
WHERE
	(file.frontmatter.pillar = "brother_uncle")
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
WHERE any(file.frontmatter.pillar = "brother_uncle")
SORT T.time_start ASC
```
