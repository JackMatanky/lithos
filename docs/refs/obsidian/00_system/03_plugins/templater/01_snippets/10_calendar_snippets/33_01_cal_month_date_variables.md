---
title: 33_01_cal_month_date_variables
aliases:
  - Monthly Calendar Date Variables
  - Calendar Month Date Variables
  - Month Calendar Date Variables
  - Month Date Variables
  - month_cal_date_variables
  - cal_month_date_variables
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
# Monthly Calendar Date Variables

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Date variables used in the monthly calendar.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATE VARIABLES
//---------------------------------------------------------
const long_date = moment(full_date).format(`MMMM YYYY`);
const med_date = moment(full_date).format(`MMM [']YY`);
const short_date = moment(full_date).format(`YYYY-MM`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const quarter = moment(full_date).format(`Q`);
const month_full_name = moment(full_date).format(`MMMM`);
const month_number = moment(full_date).format(`MM`);
const date_start = moment(full_date)
  .startOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const date_end = moment(full_date)
  .endOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`MMM [']YY`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`MMM [']YY`);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// DATE VARIABLES
//---------------------------------------------------------
const long_date = moment(full_date).format(`MMMM YYYY`);
const med_date = moment(full_date).format(`MMM [']YY`);
const short_date = moment(full_date).format(`YYYY-MM`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const quarter = moment(full_date).format(`Q`);
const month_full_name = moment(full_date).format(`MMMM`);
const month_number = moment(full_date).format(`MM`);
const date_start = moment(full_date)
  .startOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const date_end = moment(full_date)
  .endOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`MMM [']YY`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`MMM [']YY`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[33_00_month]]
2. [[33_01_month_periodic]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[nl_date|Natural Language Date Suggester]]
2. [[30_02_cal_type_and_file_class|Calendar Type and File Class]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[33_02_cal_month_titles_alias_and_file_name|Monthly Calendar Titles, Alias, and File Name]]
2. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
3. [[32_01_cal_week_date_variables|Weekly Calendar Date Variables]]
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
