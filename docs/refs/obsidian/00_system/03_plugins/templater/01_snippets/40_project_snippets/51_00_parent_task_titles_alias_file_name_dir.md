---
title: 51_00_parent_task_titles_alias_file_name_dir
aliases:
  - Parent Task Titles, Alias, File Name, and Directory
  - Titles, Alias, File Name, and Directory for Parent Tasks
  - parent task titles alias file name and directory
  - titles alias file name and directory for parent tasks
  - parent task titles alias file name dir
plugin: templater
language:
  - javascript
module:
  - 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-26T11:35
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Parent Task Titles, Alias, File Name, and Directory

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Assign a parent task's titles, alias, file name, and directory based on title.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>TODO: DEFINE <title> VARIABLE<<<
// >>>TODO: DEFINE <context_dir> VARIABLE<<<
//---------------------------------------------------------  
// PARENT TASK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//---------------------------------------------------------
const full_title_name = title;
const short_title_name = full_title_name.toLowerCase();
const short_title_value = short_title_name.replaceAll(/\s/g, "_");

const alias_arr = `["${full_title_name}", "${short_title_name}", ${short_title_value}]`

const file_name = short_title_value;

const parent_task_dir = `${project_dir}${file_name}/`;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// PARENT TASK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//---------------------------------------------------------
const full_title_name = title;
const short_title_name = full_title_name.toLowerCase();
const short_title_value = short_title_name.replaceAll(/\s/g, "_");

const alias_arr = `["${full_title_name}", "${short_title_name}", ${short_title_value}]`

const file_name = short_title_value;

const parent_task_dir = `${project_dir}${file_name}/`;
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript

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

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[rename_untitled_file_prompt|Prompt Rename Untitled File]]
2. [[task_context_project_by_path_or_suggester|Project by Path or Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[52_00_task_titles_alias_file_name|Task Titles, Alias, and File Name]]
2. [[50_00_proj_titles_alias_file_name_dir|Project Titles, Alias, File Name, and Directory]]
3. [[journal_titles_alias_and_file_name|Journal Titles, Alias, and File Name]]
4. [[journal_daily_gratitude_titles_alias_and_file_name|Daily Gratitude Titles, Alias, and File Name]]

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
