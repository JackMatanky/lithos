---
title: 32_01_cal_week_date_variables
aliases:
  - Weekly Calendar Date Variables
  - Calendar Week Date Variables
  - Week Calendar Date Variables
  - Week Date Variables
  - week_cal_date_variables
  - cal_week_date_variables
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
# Weekly Calendar Date Variables

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Date variables used in the weekly calendar.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// GENERAL DATE VARIABLES
//---------------------------------------------------------
const long_date = moment(full_date).format("[Week #]ww YYYY");
const short_date = moment(full_date).format("YYYY-[W]ww");
const year_full = moment(full_date).format("YYYY");
const year_short = moment(full_date).format("YY");
const quarter = moment(full_date).format("Q");
const month_full_name = moment(full_date).format("MMMM");
const month_short_name = moment(full_date).format("MMM");
const month_number = moment(full_date).format("MM");

//---------------------------------------------------------
// WEEK DATE VARIABLES
//---------------------------------------------------------
const week_number = moment(full_date).format("ww");
const date_start = moment(full_date)
  .startOf(type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const date_end = moment(full_date)
  .endOf(type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format("YYYY-[W]ww");
const next_date = moment(full_date)
  .add(1, moment_var)
  .format("YYYY-[W]ww");

//---------------------------------------------------------
// WEEKDAY DATE VARIABLES
//---------------------------------------------------------
const sunday = moment(full_date).day(0).format("YYYY-MM-DD");
const monday = moment(full_date).day(1).format("YYYY-MM-DD");
const tuesday = moment(full_date).day(2).format("YYYY-MM-DD");
const wednesday = moment(full_date).day(3).format("YYYY-MM-DD");
const thursday = moment(full_date).day(4).format("YYYY-MM-DD");
const friday = moment(full_date).day(5).format("YYYY-MM-DD");
const saturday = moment(full_date).day(6).format("YYYY-MM-DD");

//---------------------------------------------------------
// WEEKDAY FILE LINK EMBEDS
//---------------------------------------------------------
const sunday_link = `${sunday}\|Sunday`;
const monday_link = `${monday}\|Monday`;
const tuesday_link = `${tuesday}\|Tuesday`;
const wednesday_link = `${wednesday}\|Wednesday`;
const thursday_link = `${thursday}\|Thursday`;
const friday_link = `${friday}\|Friday`;
const saturday_link = `${saturday}\|Saturday`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// GENERAL DATE VARIABLES
//---------------------------------------------------------
const long_date = moment(full_date).format("[Week ]ww[,] YYYY");
const short_date = moment(full_date).format("YYYY-[W]ww");
const year_full = moment(full_date).format("YYYY");
const year_short = moment(full_date).format("YY");
const quarter = moment(full_date).format("Q");
const month_full_name = moment(full_date).format("MMMM");
const month_short_name = moment(full_date).format("MMM");
const month_number = moment(full_date).format("MM");

//---------------------------------------------------------
// WEEK DATE VARIABLES
//---------------------------------------------------------
const week_number = moment(full_date).format("ww");
const date_start = moment(full_date)
  .startOf(type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const date_end = moment(full_date)
  .endOf(type_value)
  .format("YYYY-MM-DD[T]HH:mm");
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format("YYYY-[W]ww");
const next_date = moment(full_date)
  .add(1, moment_var)
  .format("YYYY-[W]ww");

//---------------------------------------------------------
// WEEKDAY DATE VARIABLES
//---------------------------------------------------------
const sunday = moment(full_date).day(0).format("YYYY-MM-DD");
const monday = moment(full_date).day(1).format("YYYY-MM-DD");
const tuesday = moment(full_date).day(2).format("YYYY-MM-DD");
const wednesday = moment(full_date).day(3).format("YYYY-MM-DD");
const thursday = moment(full_date).day(4).format("YYYY-MM-DD");
const friday = moment(full_date).day(5).format("YYYY-MM-DD");
const saturday = moment(full_date).day(6).format("YYYY-MM-DD");

//---------------------------------------------------------
// WEEKDAY FILE LINK EMBEDS
//---------------------------------------------------------
const sunday_link = `${sunday}\|Sunday`;
const monday_link = `${monday}\|Monday`;
const tuesday_link = `${tuesday}\|Tuesday`;
const wednesday_link = `${wednesday}\|Wednesday`;
const thursday_link = `${thursday}\|Thursday`;
const friday_link = `${friday}\|Friday`;
const saturday_link = `${saturday}\|Saturday`;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
2. [[32_00_week|Weekly Calendar Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[nl_date|Natural Language Date Suggester]]
2. [[30_02_cal_type_and_file_class|Calendar Type and File Class]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[32_02_cal_week_titles_alias_and_file_name|Weekly Calendar Titles, Alias, and File Name]]
2. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
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
