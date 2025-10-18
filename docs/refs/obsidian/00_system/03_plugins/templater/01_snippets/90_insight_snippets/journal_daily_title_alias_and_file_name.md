---
title: journal_daily_title_alias_and_file_name
aliases:
  - Daily Journal Titles, Aliases, and File Name
  - daily journal titles, aliases, and file name
  - daily journal title alias and file name
  - daily_journal_title_alias_and_file_name
  - journal daily title alias and file name
  - journal_daily_title_alias_and_file_name
plugin: templater
language:
  - javascript
module: 
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T14:02
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Daily Journal Titles, Aliases, and File Name

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Assign the journal's titles and alias based on full type name, full type value, type name, type, and date.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// TODO: Place after journal type and subtype variables
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
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
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
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

1. [[25_10_daily_reflection|Daily Reflection Journal Template]]
2. [[25_12_daily_reflection_today_preset|Preset Daily Reflection Journal Template]]
3. [[26_10_daily_gratitude_today|Daily Gratitude Journal Template]]
4. [[26_11_daily_gratitude_today_preset|Preset Daily Gratitude Journal Template]]
5. [[27_10_daily_detachment_today|Daily Detachment Journal Template]]
6. [[27_11_daily_detachment_today_preset|Preset Daily Detachment Journal Template]]

#### In Conjunction

1. [[journal_daily_type_subtype_file_class|Daily Journal Type, Subtype, and File Class]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

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
