---
title: 32_02_cal_week_titles_alias_and_file_name
aliases:
  - Weekly Calendar Titles, Alias, and File Name
  - Calendar Week Titles, Alias, and File Name
  - Week Titles, Alias, and File Name
  - week_titles_alias_and_file_name
  - cal_week_titles_alias_and_file_name
plugin: templater
language:
  - javascript
module:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T18:16
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Weekly Calendar Titles, Alias, and File Name

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Assign the weekly calendar's titles, alias, and file name based on daily date variables.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// TODO: Define <long_date> variable
// TODO: Define <short_date> variable
//---------------------------------------------------------
// WEEKLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${long_date}`;
const short_title_value = `${short_date}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_value}"`;

const file_name = `${short_date}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// WEEKLY CALENDAR TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${long_date}`;
const short_title_value = `${short_date}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_value}"`;

const file_name = `${short_date}`;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_00_week]]
2. [[32_01_week_periodic]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[32_01_cal_week_date_variables|Weekly Calendar Date Variables]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[30_02_cal_type_and_file_class|Calendar Type and File Class]]
2. [[31_02_cal_day_titles_alias_and_file_name|Daily Calendar Titles, Alias, and File Name]]
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
