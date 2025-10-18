---
title: 55_00_ritual_daily_type_names
aliases:
  - Daily Ritual Type Names
  - daily ritual type names
  - ritual daily type names
plugin: templater
language:
  - javascript
module: 
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-05T08:00
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Daily Ritual Type Names

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Assign the daily ritual's type names.

---

## Snippet

```javascript
//---------------------------------------------------------
// RITUAL TYPE NAMES
//---------------------------------------------------------
// FULL TYPE NAME OPTIONS: Daily Morning Rituals, Daily Workday Startup Rituals, Daily Workday Shutdown Rituals, Daily Evening Rituals
const full_type_name = `<daily_ritual_name>`;
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ").splice(1, full_type_name.split(" ").length).join(" ");
const type_lower = type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_value = type_lower.slice(0, -1);
```

### Templater

```javascript
//---------------------------------------------------------
// RITUAL TYPE NAMES
//---------------------------------------------------------
// FULL TYPE NAME OPTIONS: Daily Morning Rituals, Daily Workday Startup Rituals, Daily Workday Shutdown Rituals, Daily Evening Rituals
const full_type_name = `<daily_ritual_name>`;
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ").splice(1, full_type_name.split(" ").length).join(" ");
const type_lower = type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_value = type_lower.slice(0, -1);
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
3. [[55_23_tomorrow_morn_rit]]
4. [[53_24_daily_morn_rit_quickadd]]
5. [[55_31_daily_work_start_rit]]
6. [[55_32_today_work_start_rit]]
7. [[55_33_tomorrow_work_start_rit]]
8. [[53_34_daily_work_start_rit_quickadd]]
9. [[55_41_daily_work_shut_rit]]
10. [[55_42_today_work_shut_rit]]
11. [[55_43_tomorrow_work_shut_rit]]
12. [[53_44_daily_work_shut_rit_quickadd(X)]]
13. [[55_51_daily_eve_rit]]
14. [[55_52_today_eve_rit]]
15. [[55_53_tomorrow_eve_rit]]
16. [[53_54_daily_eve_rit_quickadd]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

1. [[54_meeting_tag_type_file_class|Meeting Tag, Type, and File Class]]

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
