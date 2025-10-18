---
title: 32_06_cal_week_library
aliases:
  - Weekly Library Dataview Tables
  - weekly library dataview tables
  - Weekly Library
  - weekly library
  - cal week library
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
tags: obsidian/templater, javascript, js/momentjs, obsidian/tp/file/include
---
# Weekly Library Dataview Tables

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a week's calendar file's library dataview tables.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------  
// WEEKLY LIBRARY DATAVIEW TABLE
//---------------------------------------------------------
// COMPLETED OPTIONS: "done", "completed",
// ACTIVE OPTIONS: "active", "to_do", "in_progress"
// SCHEDULE OPTIONS: "schedule", "on_hold"
// CREATED OPTIONS: "new", "created"
// DETERMINE OPTIONS: "undetermined", "determine"
const week_lib_done = await tp.user.dv_lib_status_dates({
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_active = await tp.user.dv_lib_status_dates({
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_new = await tp.user.dv_lib_status_dates({
  status: "new",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_schedule = await tp.user.dv_lib_status_dates({
  status: "schedule",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_undetermined = await tp.user.dv_lib_status_dates({
  status: "determine",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// WEEKLY LIBRARY DATAVIEW TABLE
//---------------------------------------------------------
const week_lib_done = await tp.user.dv_lib_status_dates({
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_active = await tp.user.dv_lib_status_dates({
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_new = await tp.user.dv_lib_status_dates({
  status: "new",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_schedule = await tp.user.dv_lib_status_dates({
  status: "schedule",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_undetermined = await tp.user.dv_lib_status_dates({
  status: "determine",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_00_week|Weekly Calendar Template]]
2. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[31_00_days_of_week]]
2. [[30_01_cal_date_suggester|Calendar Date Suggester]]
3. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
4. [[31_02_cal_day_titles_alias_and_file_name|Daily Calendar Titles, Alias, and File Name]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_lib_status_dates|Library Content By Status and Dates Dataview Table]]
2. [[32_05_cal_week_journal|Weekly Journal Dataview Tables]]
3. [[32_09_cal_week_task_event|Weekly Tasks and Events Dataview Tables]]
4. [[32_07_cal_week_pkm|Weekly PKM Dataview Tables]]
5. [[32_08_cal_week_habit_ritual|Weekly Habits and Rituals Dataview Tables]]

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
