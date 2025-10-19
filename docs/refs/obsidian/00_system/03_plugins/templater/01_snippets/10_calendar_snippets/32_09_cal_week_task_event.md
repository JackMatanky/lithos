---
title: 32_09_cal_week_task_event
aliases:
  - Weekly Tasks and Events Dataview Tables
  - weekly tasks and events dataview tables
  - Weekly Tasks and Events
  - weekly tasks and events
  - cal week task event
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
# Weekly Tasks and Events Dataview Tables

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
//---------------------------------------------------------
// WEEKLY TASKS AND EVENTS DATAVIEW TABLES
//---------------------------------------------------------
// TASKS: "project", "parent_task", "child_task", "task"
// COMPLETED STATUSES: "completed", "done"
// ACTIVE STATUSES: "active", "to_do", "in_progress"
// SCHEDULE STATUSES: "schedule", "on_hold"
// DETERMINE STATUSES: "undetermined", "determine"
// CREATED STATUSES: "created", "new"
const week_active_proj = await tp.user.dv_task_type_status_dates({
  type: "project",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
const week_active_parent_task = await tp.user.dv_task_type_status_dates({
  type: "parent_task",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
const tasks_due_sunday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: sunday,
  end_date: "week",
  md: "false",
});

const tasks_due_monday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: monday,
  end_date: "week",
  md: "false",
});

const tasks_due_tuesday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: tuesday,
  end_date: "week",
  md: "false",
});

const tasks_due_wednesday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: wednesday,
  end_date: "week",
  md: "false",
});

const tasks_due_thursday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: thursday,
  end_date: "week",
  md: "false",
});

const tasks_due_friday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: friday,
  end_date: "week",
  md: "false",
});

const tasks_due_saturday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: saturday,
  end_date: "week",
  md: "false",
});
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// WEEKLY TASKS AND EVENTS DATAVIEW TABLES
//---------------------------------------------------------
// TASKS: "project", "parent_task", "child_task", "task", "habit", "morning_ritual", "workday_startup_ritual", "workday_shutdown_ritual", "evening_ritual"
// COMPLETED STATUSES: "completed", "done"
// ACTIVE STATUSES: "active", "to_do", "in_progress"
// SCHEDULE STATUSES: "schedule", "on_hold"
// DETERMINE STATUSES: "undetermined", "determine"
// CREATED STATUSES: "created", "new"
const week_active_proj = await tp.user.dv_task_type_status_dates({
  type: "project",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
const week_active_parent_task = await tp.user.dv_task_type_status_dates({
  type: "parent_task",
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});
const tasks_due_sunday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: sunday,
  end_date: "",
  md: "false",
});

const tasks_due_monday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: monday,
  end_date: "",
  md: "false",
});

const tasks_due_tuesday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: tuesday,
  end_date: "",
  md: "false",
});

const tasks_due_wednesday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: wednesday,
  end_date: "",
  md: "false",
});

const tasks_due_thursday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: thursday,
  end_date: "",
  md: "false",
});

const tasks_due_friday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: friday,
  end_date: "",
  md: "false",
});

const tasks_due_saturday = await tp.user.dv_task_type_status_dates({
  type: "child_task",
  status: "due",
  start_date: saturday,
  end_date: "",
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
4. [[32_08_cal_week_habit_ritual|Weekly Habits and Rituals Dataview Tables]]

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
