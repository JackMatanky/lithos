---
title: 50_task_event_section_habit_rit_proj
aliases:
  - Habits and Rituals Project Tasks and Events Section
  - Habits and Rituals Project Tasks and Events Section Dataview Tables
  - habits and rituals project tasks and events section dataview tables
  - task event section habits and rit proj
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-18T09:27
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Habits and Rituals Project Tasks and Events Section

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a habits and rituals project's tasks and events section formatted with headings and tables.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const habit_rit_related_task_sect_proj = "40_20_habit_rit_project_task_event_section";

//---------------------------------------------------------
// HABITS AND RITUALS PROJECT TASKS AND EVENTS SECTION
//---------------------------------------------------------
// Retrieve the Tasks and Events Section for
// Habits and Rituals Project template and content
temp_file_path = `${sys_temp_include_dir}${habit_rit_related_task_sect_proj}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const project_task_event_section = include_arr;

```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// HABITS AND RITUALS PROJECT TASKS AND EVENTS SECTION
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${habit_rit_related_task_sect_proj}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const project_task_event_section = include_arr;
```

#### Referenced Templates

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// HABITS AND RITUALS PROJECT TASKS AND EVENTS SECTION
//---------------------------------------------------------
const proj_parent_tasks_heading = "### Parent Tasks";
const proj_parent_tasks = await tp.user.dv_task_linked({
  type: "project",
  status: "",
  relation: "child",
  md: "false",
});
const child_task_remaining_heading = "### Remaining Habits and Rituals";
const child_task_remaining = await tp.user.dv_task_linked({
  type: "task",
  status: "due",
  relation: "child",
  md: "false",
});
const child_task_completed_heading = "### Completed Habits and Rituals";
const child_task_completed = await tp.user.dv_task_linked({
  type: "task",
  status: "",
  relation: "child",
  md: "false",
});
const hor_line = "---";

const project_task_event_section = `${proj_parent_tasks_heading}\n
${proj_parent_tasks}\n
${child_task_remaining_heading}\n
${child_task_remaining}\n
${child_task_completed_heading}\n
${child_task_completed}\n
${hor_line}`;

tR += project_task_event_section;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[51_00_parent_task|General Parent Task Template]]
2. [[51_20_parent_habit_ritual_month|Monthly Habits Parent Task Template]]
3. [[51_22_parent_month_morn_rit|Monthly Morning Rituals Parent Task Template]]
4. [[51_23_parent_month_work_start_rit|Monthly Workday Startup Rituals Parent Task Template]]
5. [[51_24_parent_month_work_shut_rit|Monthly Workday Shutdown Rituals Parent Task Template]]
6. [[51_25_parent_month_eve_rit|Monthly Evening Rituals Parent Task Template]]
7. [[51_32_parent_ed_book_chapter|Education Book Chapter Template]]
8. [[51_41_parent_job_application|Job Application Parent Task Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[140_00_related_task_sect_proj]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[related_task_event_section_general|Related Tasks and Events Section]]
2. [[related_task_event_section_proj_suggester|Related Tasks and Events Section with Related Project Suggester]]
3. [[related_dir_sect|Related Directory Section]]
4. [[related_lib_sect|Related Library Section]]
5. [[related_lib_sect_related_file|Related Library Section with Related Content Suggester]]
6. [[related_pkm_section|Related Personal Knowledge Section]]
7. [[related_note_section|Related Note Section]]
8. [[50_00_proj_related_section|Project Related Section]]

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

1. [[tp.file.include Templater Function|The Templater tp.file.include() Function]]

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition
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
