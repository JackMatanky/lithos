---
title: Moment Date and Natural Language Time
aliases:
  - Moment Date and Natural Language Time
  - Moment Date and NL Time
  - moment_date_and_nl_time
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
date_created: 2023-06-05T11:28
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs
---
# Moment Date and Natural Language Time

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a full datetime from the current date in [[iso_8601|ISO 8601]] YYYY-MM-DD format and the time parsed from [[Natural Language Dates]].

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET DATE AND TIME
//---------------------------------------------------------
// Today's date in ISO format
const date = moment().format(`YYYY-MM-DD`);

// Choose the time in HH:mm format
const time = await tp.user.nl_time(tp, "");

// Parse full date
const full_date_time = moment(`${date}T${time}`);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET DATE AND TIME
//---------------------------------------------------------
const date = moment().format(`YYYY-MM-DD`);
const time = await tp.user.nl_time(tp, "");
const full_date_time = moment(`${date}T${time}`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[55_21_daily_morn_rit]]
2. [[55_22_today_morn_rit]]
3. [[55_31_daily_work_start_rit]]
4. [[55_32_today_work_start_rit]]
5. [[55_41_daily_work_shut_rit]]
6. [[55_42_today_work_shut_rit]]
7. [[55_51_daily_eve_rit]]
8. [[55_52_today_eve_rit]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[nl_date|Natural Language Date Suggester]]
2. [[nl_time|Natural Language Time Suggester]]

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
