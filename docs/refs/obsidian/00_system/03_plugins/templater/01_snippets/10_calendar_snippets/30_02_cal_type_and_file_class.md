---
title: 30_02_cal_type_and_file_class
aliases:
  - Calendar Type and File Class
  - Type and File Class for Calendar Files
  - day_type_and_file_class
  - cal_type_and_file_class
plugin: templater
language:
  - javascript
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T17:19
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Calendar Type and File Class

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Assign the daily, weekly, monthly, quarterly, or yearly calendar type, moment variable, and file class.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Possible type_name assignments:
// Year, Quarter, Month, Week, Day
//---------------------------------------------------------  
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//---------------------------------------------------------  
const type_name = `calendar_variable`;
const type_value = type_name.toLowerCase();
const moment_var = `${type_value}s`;
const file_class = `cal_${type_value}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//---------------------------------------------------------  
const type_name = `Year, Quarter, Month, Week, Day`;
const type_value = type_name.toLowerCase();
const moment_var = `${type_value}s`;
const file_class = `cal_${type_value}`;
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
3. [[32_00_week]]
4. [[32_01_week_periodic]]
5. [[33_00_month]]
6. [[33_01_month_periodic]]
7. [[34_00_quarter]]
8. [[34_01_quarter_periodic]]
9. [[35_00_year]]
10. [[35_01_year_periodic]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
2. [[30_01_cal_date_suggester|Calendar Date Suggester]]
3. [[31_02_cal_day_titles_alias_and_file_name|Daily Calendar Titles, Alias, and File Name]]
4. [[32_01_cal_week_date_variables|Weekly Calendar Date Variables]]
5. [[32_02_cal_week_titles_alias_and_file_name|Weekly Calendar Titles, Alias, and File Name]]
6. [[33_01_cal_month_date_variables|Monthly Calendar Date Variables]]
7. [[33_02_cal_month_titles_alias_and_file_name|Monthly Calendar Titles, Alias, and File Name]]

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
