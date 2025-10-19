---
title: 32_08_cal_week_habit_ritual
aliases:
  - Weekly Habits and Rituals Dataview Tables
  - weekly habits and rituals dataview tables
  - Weekly Habits and Rituals
  - weekly habits and rituals
  - cal week habit ritual
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
# Weekly Habits and Rituals Dataview Tables

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a week's calendar day files.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const week_days = "31_00_days_of_week";

//---------------------------------------------------------
// WEEKDAY CALENDAR FILES
//---------------------------------------------------------
// Retrieve the Weekday Calendar Files template and content
template = await tp.file.find_tfile(week_days);
content = await tp.file.include(template);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// WEEKDAY CALENDAR FILES
//---------------------------------------------------------
template = await tp.file.find_tfile(week_days);
content = await tp.file.include(template);
```

#### Referenced Template

```javascript
//---------------------------------------------------------
// HABIT AND RITUALS TABLES
//---------------------------------------------------------
// TYPES: "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// STATUS OPTIONS: "due", "done"
const week_habit_due = await tp.user.dv_task_type_status_dates({
  type: "habit",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_morn_rit_due = await tp.user.dv_task_type_status_dates({
  type: "morning_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_work_start_due = await tp.user.dv_task_type_status_dates({
  type: "workday_startup_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_work_shut_due = await tp.user.dv_task_type_status_dates({
  type: "workday_shutdown_ritual",
  status: "due",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_eve_rit_due = await tp.user.dv_task_type_status_dates({
  type: "evening_ritual",
  status: "due",
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

1. [[32_05_cal_week_journal|Weekly Journal Dataview Tables]]
2. [[32_06_cal_week_library|Weekly Library Dataview Tables]]
3. [[32_07_cal_week_pkm|Weekly PKM Dataview Tables]]

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
