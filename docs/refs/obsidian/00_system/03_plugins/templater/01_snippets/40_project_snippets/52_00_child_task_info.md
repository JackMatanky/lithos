---
title: 52_00_child_task_info
aliases:
  - Child Task Info Callout
  - Child Task Info
  - child task info
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-22T14:38
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Child Task Info Callout

## Description

> [!snippet] Snippet Details
>
> Plugin:: [[Templater]]
> Language:: [[JavaScript]]
> Input::
> Output::
> Description:: Return a callout for Child Task Info callout without a date info.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const child_task_info_callout = "42_child_task_info_callout";

//---------------------------------------------------------
// CHILD TASK INFO CALLOUT
//---------------------------------------------------------
// Retrieve the Child Task Info template and content
temp_file_path = `${sys_temp_include_dir}${child_task_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const child_task_info = include_arr;
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// CHILD TASK INFO CALLOUT
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${child_task_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const child_task_info = include_arr;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_00_project|General Project Template]]
2. [[50_01_project_parent_tasks|General Project with Parent Tasks Template]]
3. [[50_10_proj_personal|Personal Project Template]]
4. [[50_20_proj_habit_ritual|Habits and Rituals Project Template]]
5. [[50_21_proj_habit_ritual_month|Monthly Habits and Rituals Project Template]]
6. [[50_30_proj_education|Education Project Template]]
7. [[50_31_proj_ed_course(X)|Education Course Project Template]]
8. [[50_32_proj_ed_book|Education Book Project Template]]
9. [[50_33_proj_ed_book_parent_chapter|Education Book Project and Chapter Parent Tasks Template]]
10. [[50_40_proj_professional|Professional Project Template]]
11. [[50_50_proj_work|Work Project Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[52_child_task_info_callout]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[50_00_proj_review_kiss|Project Review KISS Framework]]
2. [[53_00_action_item_preview|Before Action Preview]]
3. [[53_00_action_item_review|After Action Review]]

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
