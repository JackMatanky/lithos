---
title: 31_01_cal_day_date_variables
aliases:
  - Daily Calendar Date Variables
  - Calendar Day Date Variables
  - Day Calendar Date Variables
  - Day Date Variables
  - day_cal_date_variables
  - cal_day_date_variables
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
# Daily Calendar Date Variables

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Date variables used in the daily calendar file.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATE VARIABLES
//---------------------------------------------------------
const date = moment(full_date).format(`YYYY-MM-DD`);
const long_date = moment(full_date).format(`MMMM D, YYYY`);
const short_date = moment(full_date).format(`YY-M-D`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const year_day = moment(full_date).format(`DDDD`);
const quarter = moment(full_date).format(`Q`);
const month_full_name = moment(full_date).format(`MMMM`);
const month_short_name = moment(full_date).format(`MMM`);
const month_number = moment(full_date).format(`MM`);
const month_day = moment(full_date).format(`DD`);
const week_number = moment(full_date).format(`ww`);
const weekday_name = moment(full_date).format(`dddd`);
const weekday_number = moment(full_date).format(`[0]e`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`YYYY-MM-DD`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`YYYY-MM-DD`);
const two_weeks = moment(full_date)
  .add(14, moment_var)
  .format(`YYYY-MM-DD`);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATE VARIABLES
//---------------------------------------------------------
const date = moment(full_date).format(`YYYY-MM-DD`);
const long_date = moment(full_date).format(`MMMM D, YYYY`);
const short_date = moment(full_date).format(`YY-M-D`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const year_day = moment(full_date).format(`DDDD`);
const quarter = moment(full_date).format(`Q`);
const month_full_name = moment(full_date).format(`MMMM`);
const month_short_name = moment(full_date).format(`MMM`);
const month_number = moment(full_date).format(`MM`);
const month_day = moment(full_date).format(`DD`);
const week_number = moment(full_date).format(`ww`);
const weekday_name = moment(full_date).format(`dddd`);
const weekday_number = moment(full_date).format(`[0]e`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`YYYY-MM-DD`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`YYYY-MM-DD`);
const two_weeks = moment(full_date)
  .add(14, moment_var)
  .format(`YYYY-MM-DD`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[31_00_day]]
2. [[31_01_day_periodic]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[30_02_cal_type_and_file_class|Daily Calendar Type and File Class]]
2. [[nl_date|Natural Language Date Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[31_02_cal_day_titles_alias_and_file_name|Daily Calendar Titles, Alias, and File Name]]
2. [[32_01_cal_week_date_variables|Weekly Calendar Date Variables]]
3. [[33_01_cal_month_date_variables|Monthly Calendar Date Variables]]
4. [[34_01_cal_quarter_date_variables|Quarterly Calendar Date Variables]]

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
