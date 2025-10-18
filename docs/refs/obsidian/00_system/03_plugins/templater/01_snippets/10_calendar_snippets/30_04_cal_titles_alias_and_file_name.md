---
title: 30_04_cal_titles_alias_and_file_name
aliases:
  - Calendar Titles, Alias, and File Name
  - cal_titles_alias_and_file_name
plugin: templater
language:
  - javascript
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Daily Calendar Titles, Alias, and File Name

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Assign the daily calendar's titles, alias, and file name based on daily date variables.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// TODO: Define <date> variable
// TODO: Define <long_date> variable
// TODO: Define <short_date> variable
// TODO: Define <weekday_name> variable
//---------------------------------------------------------  
// DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const day_full_title_name = `"${weekday_name}, ${long_date_day}"`;
const day_short_title_name = `"${long_date_day}"`;
const day_full_title_value = `${med_date_day}`;
const day_short_title_value = `${short_date_day}`;

const day_alias_arr = `${new_line}${ul_yaml}${day_full_title_name}${ul_yaml}${day_short_title_name}${new_line}${ul_yaml}${day_short_title_value}`;

const day_file_name = `${med_date_day}`;

//---------------------------------------------------------  
// WEEKLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const week_full_title_name = `${long_date_week}`;
const week_short_title_value = `${short_date_week}`;

const week_alias_arr = `${new_line}${ul_yaml}"${week_full_title_name}"${ul_yaml}"${week_short_title_value}"`;

const week_file_name = `${week_short_title_value}`;

//---------------------------------------------------------  
// MONTHLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const month_full_title_name = `${long_date_month}`;
const month_short_title_name = `"${med_date_month}"`;
const month_short_title_value = `${short_date_month}`;

const month_alias_arr = `${new_line}${ul_yaml}${month_name_full}${ul_yaml}${month_full_title_name}${new_line}${ul_yaml}${month_short_title_name}${ul_yaml}${month_short_title_value}`;

const month_file_name = `${short_date_month}`;

//---------------------------------------------------------  
// QUARTERLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const quarter_full_title_name = `${long_date_quarter}`;
const quarter_short_title_name = `"${med_date_quarter}"`;
const quarter_short_title_value = `"${short_date_quarter}"`;

const quarter_alias_arr = `${new_line}${ul_yaml}${quarter_full_title_name}${ul_yaml}${quarter_short_title_name}${new_line}${ul_yaml}${quarter_short_title_value}`;

const quarter_file_name = `${quarter_short_title_name}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const day_full_title_name = `"${weekday_name}, ${long_date_day}"`;
const day_short_title_name = `"${long_date_day}"`;
const day_full_title_value = `${med_date_day}`;
const day_short_title_value = `${short_date_day}`;

const day_alias_arr = `${new_line}${ul_yaml}${day_full_title_name}${ul_yaml}${day_short_title_name}${new_line}${ul_yaml}${day_short_title_value}`;

const day_file_name = `${med_date_day}`;

//---------------------------------------------------------  
// WEEKLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const week_full_title_name = `${long_date_week}`;
const week_short_title_value = `${short_date_week}`;

const week_alias_arr = `${new_line}${ul_yaml}"${week_full_title_name}"${ul_yaml}"${week_short_title_value}"`;

const week_file_name = `${week_short_title_value}`;

//---------------------------------------------------------  
// MONTHLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const month_full_title_name = `${long_date_month}`;
const month_short_title_name = `"${med_date_month}"`;
const month_short_title_value = `${short_date_month}`;

const month_alias_arr = `${new_line}${ul_yaml}${month_name_full}${ul_yaml}${month_full_title_name}${new_line}${ul_yaml}${month_short_title_name}${ul_yaml}${month_short_title_value}`;

const month_file_name = `${short_date_month}`;

//---------------------------------------------------------  
// QUARTERLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const quarter_full_title_name = `${long_date_quarter}`;
const quarter_short_title_name = `"${med_date_quarter}"`;
const quarter_short_title_value = `"${short_date_quarter}"`;

const quarter_alias_arr = `${new_line}${ul_yaml}${quarter_full_title_name}${ul_yaml}${quarter_short_title_name}${new_line}${ul_yaml}${quarter_short_title_value}`;

const quarter_file_name = `${quarter_short_title_name}`;
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

1. [[30_03_cal_date_variables|Calendar Date Variables]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[31_02_cal_day_titles_alias_and_file_name|Daily Calendar Titles, Alias, and File Name]]
2. [[32_02_cal_week_titles_alias_and_file_name|Weekly Calendar Titles, Alias, and File Name]]
3. [[33_02_cal_month_titles_alias_and_file_name|Monthly Calendar Titles, Alias, and File Name]]
4. [[34_02_cal_quarter_titles_alias_and_file_name|Quarterly Calendar Titles, Alias, and File Name]]

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
