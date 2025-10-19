---
title: 34_01_cal_quarter_date_variables
aliases:
  - Quarterly Calendar Date Variables
  - Calendar Quarter Date Variables
  - Quarter Calendar Date Variables
  - Quarter Date Variables
  - quarter_cal_date_variables
  - cal_quarter_date_variables
plugin: templater
language:
  - javascript
module:
  - momentjs
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T17:27
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs
---
# Quarterly Calendar Date Variables

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Date variables used in the quarterly calendar file.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATE VARIABLES
//---------------------------------------------------------
const long_date = moment(full_date).format(`Qo [ Quarter of ] YYYY`);
const med_date = moment(full_date).format(`[Q]Q YYYY`);
const short_date = moment(full_date).format(`[Q]Q [']YY`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const quarter = moment(full_date).format(`Q`);
const date_start = moment(full_date)
  .startOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const date_end = moment(full_date)
  .endOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`[Q]Q [']YY`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`[Q]Q [']YY`);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATE VARIABLES
//---------------------------------------------------------
const long_date = moment(full_date).format(`Qo [ Quarter of ] YYYY`);
const med_date = moment(full_date).format(`[Q]Q YYYY`);
const short_date = moment(full_date).format(`[Q]Q [']YY`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const quarter = moment(full_date).format(`Q`);
const date_start = moment(full_date)
  .startOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const date_end = moment(full_date)
  .endOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`[Q]Q [']YY`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`[Q]Q [']YY`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[34_00_quarter]]
2. [[34_01_quarter_periodic]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[nl_date|Natural Language Date Suggester]]
2. [[30_02_cal_type_and_file_class|Calendar Type and File Class]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[34_02_cal_quarter_titles_alias_and_file_name|Quarterly Calendar Titles, Alias, and File Name]]

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
