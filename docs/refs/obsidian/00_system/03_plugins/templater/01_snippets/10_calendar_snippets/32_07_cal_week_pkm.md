---
title: 32_07_cal_week_pkm
aliases:
  - Weekly PKM Dataview Tables
  - weekly pkm dataview tables
  - Weekly PKM files
  - weekly pkm files
  - cal week pkm
plugin: templater
language:
  - javascript
module:
  - momentjs
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-24T07:50
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs, obsidian/tp/file/include
---
# Weekly PKM Dataview Tables

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
// PKM FILES DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
const week_note_perm = await tp.user.dv_pkm_type_status_dates({
  type: "permanent",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_lit = await tp.user.dv_pkm_type_status_dates({
  type: "literature",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_fleet = await tp.user.dv_pkm_type_status_dates({
  type: "type",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_info = await tp.user.dv_pkm_type_status_dates({
  type: "info",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_tree = await tp.user.dv_pkm_type_status_dates({
  type: "tree",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_review = await tp.user.dv_pkm_type_status_dates({
  type: "",
  status: "review",
  start_date: "",
  end_date: "",
  md: "false",
});

const week_note_clarify = await tp.user.dv_pkm_type_status_dates({
  type: "",
  status: "clarify",
  start_date: "",
  end_date: "",
  md: "false",
});

const week_note_develop = await tp.user.dv_pkm_type_status_dates({
  type: "",
  status: "develop",
  start_date: "",
  end_date: "",
  md: "false",
});
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// PKM FILES DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
const week_note_perm = await tp.user.dv_pkm_type_status_dates({
  type: "permanent",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_lit = await tp.user.dv_pkm_type_status_dates({
  type: "literature",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_fleet = await tp.user.dv_pkm_type_status_dates({
  type: "fleeting",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_info = await tp.user.dv_pkm_type_status_dates({
  type: "info",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_tree = await tp.user.dv_pkm_type_status_dates({
  type: "tree",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_review = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "review",
  start_date: "",
  end_date: "",
  md: "false",
});

const week_note_clarify = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "clarify",
  start_date: "",
  end_date: "",
  md: "false",
});

const week_note_develop = await tp.user.dv_pkm_type_status_dates({
  type: "not_tree",
  status: "develop",
  start_date: "",
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
2. [[32_09_cal_week_task_event|Weekly Tasks and Events Dataview Tables]]
3. [[32_06_cal_week_library|Weekly Library Dataview Tables]]
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
