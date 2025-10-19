---
title: nl_start_end_date
aliases:
  - Natural Language Start and End Date
  - NL Start and End Date
  - nl start and end date
  - nl start end date
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
date_created: 2023-06-13T13:49
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs
---
# Natural Language Start and End Date

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a start and end date parsed from natural language by the [[Natural Language Dates]] plugin into [[iso_8601|ISO 8601]] YYYY-MM-DD format.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET START AND END DATES
//---------------------------------------------------------
// Choose the start date
const date_start = await tp.user.nl_date(tp);

// Choose the end date
const date_end = await tp.user.nl_date(tp);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET START AND END DATES
//---------------------------------------------------------
const date_start = await tp.user.nl_date(tp);
const date_end = await tp.user.nl_date(tp);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_00_project]]
2. [[50_10_proj_personal]]
3. [[50_20_proj_habit_ritual]]
4. [[50_30_proj_education]]
5. [[50_50_proj_work]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[nl_date|Natural Language Date Suggester]]
2. [[nl_date_and_time|Natural Language Date and Time]]

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
