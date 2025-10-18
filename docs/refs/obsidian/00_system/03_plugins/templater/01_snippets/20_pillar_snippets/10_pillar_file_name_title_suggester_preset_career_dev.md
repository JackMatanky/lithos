---
title: 10_pillar_file_name_title_suggester_preset_career_dev
aliases:
  - Pillar File Name and Title Suggester Preset with Career Development
  - pillar file name and title suggester preset with career development
  - Pillar File and Full Name Preset with Career Development
  - pillar_file_and_full_name_preset_career_dev
  - pillar_file_and_full_name_suggester_preset_career_dev
  - suggester_pillar_file_full_name_preset_career_dev
  - pillar_file_name_title_suggester_preset_career_dev
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
# Pillar File Name and Title Suggester Preset with Career Development

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a pillar's file name and the pillar's main title from `alias[0]`. Preset with the Career Development pillar's information.

---

## Snippet

```javascript
// Template file to include
const pillar_name_alias_preset_career = "20_01_pillar_name_alias_preset_career";

//---------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET CAREER DEV.
//---------------------------------------------------------
// Retrieve the Pillar File Name and Title with 
// Career Development Preset template and content
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias_preset_career}.md`;
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
// SET PILLAR FILE AND FULL NAME; PRESET CAREER DEV.
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias_preset_career}.md`;
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
// SET PILLAR FILE AND FULL NAME; PRESET CAREER DEV.
//---------------------------------------------------------
// Pillar Files Directory
const pillars_dir = "20_pillars/";

// CAREER DEVELOPMENT PILLAR FILE AND FULL NAME
const preset_pillar_name = "Career Development";
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

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[20_01_pillar_name_alias_preset_career]]

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
