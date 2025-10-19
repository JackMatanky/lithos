---
title: 10_pillar_file_name_title_suggester
aliases:
  - Pillar File Name and Title Suggester
  - pillar file name and title suggester
  - Pillar File and Full Name
  - pillar_file_and_full_name
  - pillar_file_and_full_name_suggester
  - suggester_pillar_file_full_name
  - pillar_file_name_title_suggester
plugin: templater
language:
  - javascript
module:
  - system
  - user
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T15:53
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Pillar File and Full Name Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a pillar's file name and the pillar's main title from `alias[0]`.

---

## Snippet

```javascript
// Template file to include
const pillar_name_alias = `20_00_pillar_name_alias`;

//---------------------------------------------------------
// SET PILLAR FILE AND FULL NAME
//---------------------------------------------------------
// Retrieve the Pillar File Name template and content
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar_value = include_arr[0];
const pillar_name = include_arr[1];
const pillar_link = `${pillar_value}|${pillar_name}`;
const pillar_value_link = `${new_line}${ul_yaml}"${pillar_link}"`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET PILLAR FILE AND FULL NAME
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar_value = include_arr[0];
const pillar_name = include_arr[1];
const pillar_link = `${pillar_value}|${pillar_name}`;
const pillar_value_link = `${new_line}${ul_yaml}"${pillar_link}"`;
```

#### Referenced Template

```javascript
//---------------------------------------------------------
// SET PILLAR FILE NAME AND ALIAS
//---------------------------------------------------------
// Pillar Files Directory
const pillars_dir = `20_pillars/`;

// Retrieve all files in the Pillars directory
const pillars_obj_arr = await tp.user.file_by_status({
  dir: pillars_dir,
  status: "active",
});

const pillar_obj = await tp.system.suggester(
  (item) => item.key,
  pillars_obj_arr,
  false,
  `Pillar?`
);

const pillar_value = pillar_obj.value;
const pillar_name = pillar_obj.key;

tR += pillar_value
tR += ","
tR += pillar_name
```

### Old Snippet

```javascript
// Pillar Files Directory
const pillars_dir = `20_pillars/`;

//---------------------------------------------------------
// SET PILLAR
//---------------------------------------------------------
// Retrieve all files in the Pillars directory
const pillars = await tp.user.file_by_status({
  dir: pillars_dir,
  status: `active`,
});

const pillar = await tp.system.suggester(
  pillars,
  pillars,
  false,
  `Pillar?`
);

// Split the pillar file name
const pillar_arr = pillar.split(`_`);

// Initialize the pillar's name variable
let pillar_name = ``;

// Loop through pillar array
for (var i = 0; i < pillar_arr.length; i++) {
  // Extract and capitalize each word's first letter.
  first_letter = pillar_arr[i].charAt(0).toUpperCase();
  // Extract the rest of the word.
  substring = pillar_arr[i].substring(1);
  // Concatenate both values with a trailing whitespace.
  pillar_name += `${first_letter}${substring} `;
}

// Trim the pillar name
pillar_name.trim();
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[20_00_journal]]
2. [[25_10_daily_reflection]]
3. [[50_00_project|General Project Template]]
4. [[50_01_project_parent_tasks|General Project with Parent Tasks Template]]
5. [[50_10_proj_personal|Personal Project Template]]
6. [[50_20_proj_habit_ritual|Habits and Rituals Project Template]]
7. [[50_21_proj_habit_ritual_month|Monthly Habits and Rituals Project Template]]
8. [[50_30_proj_education|Education Project Template]]
9. [[50_31_proj_ed_course(X)|Education Course Project Template]]
10. [[50_32_proj_ed_book|Education Book Project Template]]
11. [[50_40_proj_professional|Professional Project Template]]
12. [[50_50_proj_work|Work Project Template]]
13. [[51_00_parent_task|General Parent Task Template]]
14. [[52_00_task_event|General Tasks and Events Template]]
15. [[53_00_action_item|Action Item Template]]
16. [[54_00_meeting|Meeting Template]]
17. [[55_10_habit|Habit Task Template]]
18. [[55_31_daily_work_start_rit]]
19. [[80_00_pkm_tree|General Knowledge Tree Template]]
20. [[80_01_tree_category|Knowledge Tree Category Template]]
21. [[80_02_tree_branch|Knowledge Tree Branch Template]]
22. [[80_03_tree_field|Knowledge Tree Field Template]]
23. [[80_04_tree_subject|Knowledge Tree Subject Template]]
24. [[80_05_tree_topic|Knowledge Tree Topic Template]]
25. [[80_06_tree_subtopic|Knowledge Tree Subtopic Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[20_00_pillar_name_alias]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[get_file_by_status|Markdown File Names Filtered by Frontmatter Status]]
2. [[file_name_alias_by_class_type|Markdown File Names Filtered by Directory, File Class, and Type]]
3. [[61_contact_file_name_title_suggester|Contact File Name and Title]]
4. [[62_organization_file_name_title_suggester|Organization File Name and Title]]

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

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]
2. [[tp.file.include Templater Function|The Templater tp.file.include() Function]]

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
