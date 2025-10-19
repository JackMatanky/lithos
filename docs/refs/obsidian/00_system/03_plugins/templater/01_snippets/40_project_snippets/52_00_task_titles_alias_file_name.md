---
title: 52_00_task_titles_alias_file_name
aliases:
  - Task Titles, Alias, and File Name
  - Titles, Alias, and File Name for Tasks
  - task titles alias and file name
  - titles alias and file name for tasks
  - task titles alias file name
plugin: templater
language:
  - javascript
module:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T14:02
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Task Titles, Alias, and File Name

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Assign a task's titles, alias, and file name based on title and date.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>TODO: DEFINE <title> VARIABLE<<<
// >>>TODO: DEFINE <date> VARIABLE<<<
//---------------------------------------------------------
// TASK TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${date} ${title}`;
const short_title_name = `${title.toLowerCase()}`;
const short_title_value = short_title_name
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/'/g, "");
const full_title_value = `${date}_${short_title_value}`;

const alias_arr = `${new_line}${ul_yaml}"${title}"${ul_yaml}"${full_title_name}"${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}"${short_title_value}"${new_line}${ul_yaml}"${full_title_value}"`;

const file_name = full_title_value;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// TASK TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${date} ${title}`;
const short_title_name = `${title.toLowerCase()}`;
const short_title_value = short_title_name
  .replaceAll(/[\s-]/g, "_")
  .replaceAll(/'/g, "");
const full_title_value = `${date}_${short_title_value}`;

const alias_arr = `${new_line}${ul_yaml}"${title}"${ul_yaml}"${full_title_name}"${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}"${short_title_value}"${new_line}${ul_yaml}"${full_title_value}"`;

const file_name = full_title_value;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[52_00_task_event|General Tasks and Events Template]]
2. [[53_00_action_item|Action Item Template]]
3. [[54_00_meeting|Meeting Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[rename_untitled_file_prompt|Prompt Rename Untitled File]]
2. [[nl_date_and_time|NL Date and Time]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[50_00_proj_titles_alias_file_name_dir|Project Titles, Alias, and File Name]]
2. [[journal_titles_alias_and_file_name|Journal Titles, Alias, and File Name]]
3. [[journal_daily_gratitude_titles_alias_and_file_name|Daily Gratitude Titles, Alias, and File Name]]

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
