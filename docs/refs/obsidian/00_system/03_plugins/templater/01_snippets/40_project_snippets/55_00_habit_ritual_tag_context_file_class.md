---
title: 55_00_habit_ritual_tag_context_file_class
aliases:
  - Habits and Rituals Task Tag, Context, and File Class
  - habits and rituals task tag, context, and file class
  - habit ritual tag context file class
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
# Habits and Rituals Task Tag, Context, and File Class

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return the habits and/or rituals task tag, context, and file class.

---

## Snippet

```javascript
//---------------------------------------------------------  
// HABIT'S AND RITUAL'S TASK TAG, CONTEXT, AND FILE CLASS
//---------------------------------------------------------
const task_tag = `#task`;
const context_name = `Habits and Rituals`;
const context = context_name.replaceAll(/s\sand\s/g, "_").replaceAll(/s$/g, "").toLowerCase();
const file_class = `task_${context}`;
```

### Templater

```javascript
//---------------------------------------------------------  
// HABIT'S AND RITUAL'S TASK TAG, CONTEXT, AND FILE CLASS
//---------------------------------------------------------
const task_tag = `#task`;
const context_name = `Habits and Rituals`;
const context = context_name.replaceAll(/s\sand\s/g, "_").replaceAll(/s$/g, "").toLowerCase();
const file_class = `task_${context}`;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[55_10_habit]]
2. [[55_21_daily_morn_rit]]
3. [[55_22_today_morn_rit]]
4. [[55_23_tomorrow_morn_rit]]
5. [[53_24_daily_morn_rit_quickadd]]
6. [[55_31_daily_work_start_rit]]
7. [[55_32_today_work_start_rit]]
8. [[55_33_tomorrow_work_start_rit]]
9. [[53_34_daily_work_start_rit_quickadd]]
10. [[55_41_daily_work_shut_rit]]
11. [[55_42_today_work_shut_rit]]
12. [[55_43_tomorrow_work_shut_rit]]
13. [[53_44_daily_work_shut_rit_quickadd(X)]]
14. [[55_51_daily_eve_rit]]
15. [[55_52_today_eve_rit]]
16. [[55_53_tomorrow_eve_rit]]
17. [[53_54_daily_eve_rit_quickadd]]

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
