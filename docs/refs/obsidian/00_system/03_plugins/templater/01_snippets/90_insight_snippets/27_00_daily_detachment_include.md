---
title: 27_00_daily_detachment_include
aliases:
  - Daily Detachment File Include
  - daily detachment file include
  - daily_detachment_file_include
  - daily detachment include
  - daily_detachment_include
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
# Daily Detachment File Include

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return the daily detachment file with preset information.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const daily_detachment_journal = "97_daily_detachment_journal";

//---------------------------------------------------------  
// DETACHMENT JOURNAL
//---------------------------------------------------------
// Retrieve the Detachment Journal Info template
temp_file_path = `${sys_temp_include_dir}${daily_detachment_journal}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// DETACHMENT JOURNAL
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${daily_detachment_journal}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// FOLDER PATH VARIABLES
//---------------------------------------------------------
const sys_temp_include_dir = "00_system/06_template_include/";
const gratitude_journals_dir = "80_insight/96_gratitude";

//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const hr_line = hyphen.repeat(3);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const colon = String.fromCodePoint(0x3a);
const two_colon = colon.repeat(2);
const two_percent = String.fromCodePoint(0x25).repeat(2);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

//Text Formatting
const head_one = `${hash}${space}`;
const head_two = `${hash.repeat(2)}${space}`;
const head_three = `${hash.repeat(3)}${space}`;
const head_four = `${hash.repeat(4)}${space}`;
const cmnt_ob_start = `${two_percent}${space}`;
const cmnt_ob_end = `${space}${two_percent}`;
const cmnt_html_start = `${less_than}${excl}${two_hyphen}${space}`;
const cmnt_html_end = `${space}${two_hyphen}${great_than}`;
const tbl_start =`${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end =`${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${hyphen}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${great_than}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_start}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;

//---------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//---------------------------------------------------------
const pdev_journal_info_callout = "90_pdev_journal_info_callout";

//---------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//---------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//---------------------------------------------------------
// JOURNAL WRITING DATE AND PREVIOUS DATE
//---------------------------------------------------------
const date = moment().format("YYYY-MM-DD");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const prev_date = moment().subtract(1, "days").format("YYYY-MM-DD");
const full_date_time = moment(`${date}T00:00`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

//---------------------------------------------------------
// JOURNAL TYPE AND FILE CLASS
//---------------------------------------------------------
const full_type_name = "Daily Gratitude Journal";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const long_type_name = `${full_type_name.split(" ")[0]} ${full_type_name.split(" ")[1]}`;
const long_type_value = long_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[1];
const type_value = full_type_value.split("_")[1];
const subtype_name = full_type_name.split(" ")[0];
const subtype_value = full_type_value.split("_")[0];
const file_class = `pdev_${full_type_value.split("_")[2]}`;

//---------------------------------------------------------
// JOURNAL TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${date} ${full_type_name}`;
const partial_title_name = `${date} ${long_type_name}`;
const short_title_name = `${date} ${type_name}`;
const full_title_value = `${short_date_value}_${full_type_value}`;
const partial_title_value = `${short_date_value}_${long_type_value}`;
const short_title_value = `${short_date_value}_${type_value}`;

const alias_arr = [long_type_name, full_type_name, full_title_name, partial_title_name, short_title_name, full_title_value, partial_title_value, short_title_value];
let file_alias = "";
for (let i = 0; i < alias_arr.length; i++) {
  alias = `${new_line}${ul_yaml}"${alias_arr[i]}"`;
  file_alias += alias;
};

const file_name = partial_title_value;

//---------------------------------------------------------
// PILLAR FILE AND FULL NAME
//---------------------------------------------------------
const pillar_name = "Mental Health";
const pillar_value = pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const pillar_link = `[[${pillar_value}|${pillar_name}]]`;
const pillar_value_link = `${new_line}${ul_yaml}"${pillar_link}"`;

//---------------------------------------------------------
// SET GOAL
//---------------------------------------------------------
const goal = "null";

//---------------------------------------------------------
// RELATED PROJECT FILE NAME, ALIAS, AND LINK
//---------------------------------------------------------
const year_month_short = moment().format("YYYY-MM");
const year_month_long = moment().format("MMMM [']YY");

const context_name = "Habits and Rituals";
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();

const project_name = `${year_month_long} ${context_name}`;
const project_value = `${year_month_short}_${context_value}`;
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = `${new_line}${ul_yaml}"${project_link}"`;

//---------------------------------------------------------
// RELATED PARENT TASK FILE NAME, ALIAS, AND LINK
//---------------------------------------------------------
const habit_ritual_order = "02";
const habit_ritual_name = "Morning Rituals";
const parent_task_name = `${year_month_long} ${habit_ritual_name}`;
const parent_task_value = `${year_month_short}_${habit_ritual_order}_${habit_ritual_name.replaceAll(/\s/g, "_").toLowerCase()}`;
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = `${new_line}${ul_yaml}"${parent_task_link}"`;

//---------------------------------------------------------  
// PDEV JOURNAL INFO CALLOUT
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pdev_journal_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const journal_info = include_arr;

//---------------------------------------------------------
// FILE DETAILS CALLOUT
//---------------------------------------------------------
const info_title = `${call_start}[!${type_value}]${space}${full_type_name}${space}Details${new_line}${call_start}${new_line}`;

const info = `${info_title}${journal_info}`;

//---------------------------------------------------------  
// GRATITUDE HEADINGS AND INLINE DATA
//---------------------------------------------------------
heading = "I Am Grateful For…";
const head_gratitude = `${head_two}${heading}${two_new_line}`;
const inline_gratitude = `1.${space}**Gratitude**::${new_line}2.${space}**Gratitude**::${new_line}3.${space}**Gratitude**::${new_line}`;
const gratitude = `${head_gratitude}${inline_gratitude}`;

heading = "I Can Thank Myself For…";
const head_gratitude_self = `${head_two}${heading}${two_new_line}`;
const inline_gratitude_self = `1.${space}**Self${space}Gratitude**::${new_line}2.${space}**Self${space}Gratitude**::${new_line}3.${space}**Self${space}Gratitude**::${new_line}`;
const gratitude_self = `${head_gratitude_self}${inline_gratitude_self}`;

//---------------------------------------------------------  
// FILE FRONTMATTER AND CONTENT
//---------------------------------------------------------
yaml_title = `title:${space}${file_name}${new_line}`;
yaml_aliases = `aliases:${space}${file_alias}${new_line}`;
yaml_date = `date:${space}${date_value_link}${new_line}`;
yaml_pillar = `pillar:${space}${pillar_value_link}${new_line}`;
yaml_goal = `goal:${space}${goal}${new_line}`;
yaml_project = `project:${space}${project_value_link}${new_line}`;
yaml_parent_task = `parent_task:${space}${parent_task_value_link}${new_line}`;
yaml_subtype = `subtype:${space}${subtype_value}${new_line}`;
yaml_type = `type:${space}${type_value}${new_line}`;
yaml_file_class = `file_class:${space}${file_class}${new_line}`;
yaml_date_created = `date_created:${space}${date_created}${new_line}`;
yaml_date_modified = `date_modified:${space}${date_modified}${new_line}`;
yaml_tags = `tags:${new_line}`;

const frontmatter = `${hr_line}${yaml_title}${yaml_aliases}${yaml_date}${yaml_pillar}${yaml_goal}${yaml_project}${yaml_parent_task}${yaml_subtype}${yaml_type}${yaml_file_class}${yaml_date_created}${yaml_date_modified}${yaml_tags}${hr_line}`;

const file_content = `${frontmatter}
${head_one}${full_type_name}${new_line}
${info}
${new_line}${hr_line}${new_line}
${gratitude}
${hr_line}${new_line}
${gratitude_self}
${hr_line}${new_line}
${head_two}Related${new_line}
${prev_date}`;

//---------------------------------------------------------
// CREATE FILE IN DIRECTORY
//---------------------------------------------------------
const directory = gratitude_journals_dir;
await tp.file.create_new(
  file_content,
  file_name,
  false,
  app.vault.getAbstractFileByPath(directory)
);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[20_01_daily_journals_preset]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[90_pdev_journal_info_callout]]

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
