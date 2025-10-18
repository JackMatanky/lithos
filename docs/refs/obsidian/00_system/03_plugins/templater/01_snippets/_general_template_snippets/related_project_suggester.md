---
title: related_project_suggester
aliases:
  - Related Project Suggester
  - Related Project
  - suggester_related_project
  - related project suggester
plugin: templater
language:
  - javascript
module:
  - system
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T14:16
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Related Project Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a related project's file name, alias, and link.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const related_project = "40_related_project";

//---------------------------------------------------------
// SET RELATED PROJECT
//---------------------------------------------------------
// Retrieve the Related Projects template and content
temp_file_path = `${sys_temp_include_dir}${related_project}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const project_value = include_arr[0];
const project_name = include_arr[1];
const project_link = `${project_value}|${project_name}`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET RELATED PROJECT
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_project}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const project_value = include_arr[0];
const project_name = include_arr[1];
const project_link = `${project_value}|${project_name}`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;
```

#### Referenced Template

```javascript
//---------------------------------------------------------
// SET PROJECT
//---------------------------------------------------------
// Projects directory
const projects_dir = "40_projects/";

// Filter array to only include projects in the Projects Directory
const projects_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: projects_dir,
    file_class: "task",
    type: "project",
  });
  
const project_obj = await tp.system.suggester(
  (item) => item.key,
  projects_obj_arr,
  false,
  "Is this file related to a project?"
  );
  
const project_value = project_obj.value;
const project_name = project_obj.key;

tR += project_value
tR += ","
tR += project_name
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[28_00_journal_prompt|Prompt Journal Template]]
2. [[80_00_pkm_tree|General Knowledge Tree Template]]
3. [[80_01_tree_category|Knowledge Tree Category Template]]
4. [[80_02_tree_branch|Knowledge Tree Branch Template]]
5. [[80_03_tree_field|Knowledge Tree Field Template]]
6. [[80_04_tree_subject|Knowledge Tree Subject Template]]
7. [[80_05_tree_topic|Knowledge Tree Topic Template]]
8. [[80_06_tree_subtopic|Knowledge Tree Subtopic Template]]
9. [[90_00_note|General Note Template]]
10. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
11. [[90_11_note_quote|Quote Fleeting Note Template]]
12. [[90_12_note_idea|Idea Fleeting Note Template]]
13. [[90_20_note_literature(X)|General Literature Note Template]]
14. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
15. [[90_31_note_question|QEC Question Note Template]]
16. [[90_32_note_evidence|QEC Evidence Note Template]]
17. [[90_33_note_conclusion|QEC Conclusion Note Template]]
18. [[90_40_note_lit_psa(X)|PSA Note Template]]
19. [[90_41_note_problem|PSA Problem Note Template]]
20. [[90_42_note_steps|PSA Steps Note Template]]
21. [[90_43_note_answer|PSA Answer Note Template]]
22. [[90_50_note_info(X)|General Info Note Template]]
23. [[90_51_note_concept|Concept Note Template]]
24. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[50_related_project]]
2. [[related_parent_task_suggester|Related Parent Task Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[task_parent_task_by_path_or_suggester|Parent Task by Path or Suggester]]
2. [[task_context_by_path_or_suggester|Task Context by Path or Suggester]]
3. [[task_context_project_by_path_or_suggester|Project by Path or Suggester]]
4. [[journal_related_project|Journal Related Project Suggester]]
5. [[journal_related_parent_task|Journal Related Parent Task Suggester]]

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
