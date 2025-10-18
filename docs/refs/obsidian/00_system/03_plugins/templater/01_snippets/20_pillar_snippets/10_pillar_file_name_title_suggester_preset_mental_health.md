---
title: 10_pillar_file_name_title_suggester_preset_mental_health
aliases:
  - Pillar File Name and Title Suggester Preset with Mental Health
  - pillar file name and title suggester preset with mental health
  - pillar_file_name_and_title_suggester_preset_mental_health
  - suggester_pillar_file_name_and_title_preset_mental_health
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
# Pillar File Name and Title Suggester Preset with Mental Health

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a pillar's file name and the pillar's main title from `alias[0]`. Preset with the Mental Health pillar's information.

---

## Snippet

```javascript
// Template file to include
const pillar_name_alias_preset_mental = "20_03_pillar_name_alias_preset_mental";

//---------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET MENTAL HEALTH
//---------------------------------------------------------
// Retrieve the Pillar File Name and Title with 
// Mental Health Preset template and content
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias_preset_mental}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar_value = include_arr[0];
const pillar_value_link = include_arr[1];
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET MENTAL HEALTH
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias_preset_mental}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar_value = include_arr[0];
const pillar_value_link = include_arr[1];
```

#### Referenced Template

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const new_line = String.fromCodePoint(0xa);
const ul = `${String.fromCodePoint(0x2d)}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;

//---------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET KNOW. EXPANSION
//---------------------------------------------------------
// Pillar Files Directory
const pillars_dir = "20_pillars/";

// MENTAL HEALTH PILLAR FILE AND FULL NAME
const preset_pillar_name = "Mental Health";
const preset_pillar_value = preset_pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const preset_pillar_link = `[[${preset_pillar_value}|${preset_pillar_name}]]`;
const preset_pillar_value_link = `${new_line}${ul_yaml}"${preset_pillar_link}"`;

// Retrieve all files in the Pillars directory
const pillars_obj_arr = await tp.user.file_by_status({
  dir: pillars_dir,
  status: "active",
});

const pillars_obj_arr_filter = pillars_obj_arr.filter(
  (pillar) => pillar.value != preset_pillar_value
);

const pillar_obj = await tp.system.suggester(
  (item) => item.key,
  pillars_obj_arr_filter,
  false,
  `Pillar (excluding ${preset_pillar_name})?`
);

let pillar_value = pillar_obj.value;
let pillar_name = pillar_obj.key;
let pillar_link = `[[${pillar_value}|${pillar_name}]]`;
let pillar_value_link = `${new_line}${ul_yaml}"${pillar_link}"`;

if (pillar_value != "null") {
  pillar_value = `[${pillar_value}, ${preset_pillar_value}]`;
  pillar_link = `${pillar_link}, ${preset_pillar_link}`;
  pillar_value_link = `${pillar_value}${preset_pillar_value_link}`;
} else {
  pillar_value = `${preset_pillar_value}`;
  pillar_link = `${preset_pillar_link}`;
  pillar_value_link = `${preset_pillar_value_link}`;
}

tR += pillar_value;
tR += ";"
tR += pillar_name;
tR += ";"
tR += pillar_link;
tR += ";"
tR += pillar_value_link;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_30_proj_education|Education Project Template]]
2. [[50_31_proj_ed_course(X)|Education Course Project Template]]
3. [[50_32_proj_ed_book|Education Book Project Template]]
4. [[50_33_proj_ed_book_parent_chapter|Education Book Project and Chapter Parent Tasks Template]]
5. [[51_32_parent_ed_book_chapter|Education Book Chapter Parent Task Template]]
6. [[80_00_pkm_tree|General Knowledge Tree Template]]
7. [[80_01_tree_category|Knowledge Tree Category Template]]
8. [[80_02_tree_branch|Knowledge Tree Branch Template]]
9. [[80_03_tree_field|Knowledge Tree Field Template]]
10. [[80_04_tree_subject|Knowledge Tree Subject Template]]
11. [[80_05_tree_topic|Knowledge Tree Topic Template]]
12. [[80_06_tree_subtopic|Knowledge Tree Subtopic Template]]
13. [[90_00_note|General Note Template]]
14. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
15. [[90_11_note_quote|Quote Fleeting Note Template]]
16. [[90_12_note_idea|Idea Fleeting Note Template]]
17. [[90_20_note_literature(X)|General Literature Note Template]]
18. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
19. [[90_31_note_question|QEC Question Note Template]]
20. [[90_32_note_evidence|QEC Evidence Note Template]]
21. [[90_33_note_conclusion|QEC Conclusion Note Template]]
22. [[90_40_note_lit_psa(X)|PSA Note Template]]
23. [[90_41_note_problem|PSA Problem Note Template]]
24. [[90_42_note_steps|PSA Steps Note Template]]
25. [[90_43_note_answer|PSA Answer Note Template]]
26. [[90_50_note_info(X)|General Info Note Template]]
27. [[90_51_note_concept|Concept Note Template]]
28. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[20_03_pillar_name_alias_preset_mental]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[get_file_by_status|Markdown File Names Filtered by Frontmatter Status]]
2. [[file_name_alias_by_class_type|Markdown File Names Filtered by Directory, File Class, and Type]]
3. [[10_pillar_file_name_title_suggester|Pillar File Name and Title Suggester]]
4. [[61_contact_file_name_title_suggester|Contact File Name and Title]]
5. [[62_organization_file_name_title_suggester|Organization File Name and Title]]

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
