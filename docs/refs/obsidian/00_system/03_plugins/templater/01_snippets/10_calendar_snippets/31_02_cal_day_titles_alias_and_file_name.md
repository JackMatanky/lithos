---
title: 012_cal_day_titles_alias_and_file_name
aliases:
  - Daily Calendar Titles, Alias, and File Name
  - Calendar Day Titles, Alias, and File Name
  - Day Titles, Alias, and File Name
  - day_titles_alias_and_file_name
  - cal_day_titles_alias_and_file_name
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
const full_title_name = `${weekday_name}, ${long_date}`;
const short_title_name = `${long_date}`;
const full_title_value = `${date}`;
const short_title_value = `${short_date}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}${short_title_value}`;

const file_name = `${date}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${weekday_name}, ${long_date}`;
const short_title_name = `${long_date}`;
const full_title_value = `${date}`;
const short_title_value = `${short_date}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}${short_title_value}`;

const file_name = `${date}`;
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

1. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[30_02_cal_type_and_file_class|Daily Calendar Type and File Class]]
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
