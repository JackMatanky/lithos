---
title: journal_titles_alias_and_file_name
aliases:
  - Journal Titles, Alias, and File Name
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
# Journal Titles, Alias, and File Name

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
// TODO: Define <full_type_name> variable
// TODO: Define <full_type_value> variable
// TODO: Define <type_name> variable
// TODO: Define <type> variable
// TODO: Define <date> variable
//---------------------------------------------------------  
// JOURNAL TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${date} ${full_type_name}`;
const short_title_name = `${date} ${type_name}`;
const full_title_value = `${date} ${full_type_value}`;
const short_title_value = `${date} ${type_value}`;
const alias = `${date}_${full_type_value}`;

const alias_arr = `${new_line}${ul_yaml}"${type_name}"${ul_yaml}"${full_type_name}"${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}"${full_title_value}"${new_line}${ul_yaml}"${short_title_value}"${ul_yaml}"${alias}"`;

const file_name = `${date}_${type_value}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// JOURNAL TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = `${date} ${full_type_name}`;
const short_title_name = `${date} ${type_name}`;
const full_title_value = `${date} ${full_type_value}`;
const short_title_value = `${date} ${type_value}`;
const alias = `${date}_${full_type_value}`;

const alias_arr = `${new_line}${ul_yaml}"${full_type_name}"${ul_yaml}"${type_name}"${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}"${full_title_value}"${new_line}${ul_yaml}"${short_title_value}"${ul_yaml}"${alias}"`;

const file_name = `${date}_${type_value}`;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

1. [[25_00_journal_reflection|General Reflection Journal Template]]
2. [[26_00_gratitude_journal|Gratitude Journal Template]]
3. [[27_00_detachment_journal|Detachment Journal Template]]

#### In Conjunction

1. [[journal_daily_reflection_type_and_file_class|Daily Reflection Journal Type and File Class]]
2. [[nl_date|Natural Language Date Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[journal_daily_gratitude_titles_alias_and_file_name|Daily Gratitude Titles, Alias, and File Name]]
2. [[journal_daily_detachment_titles_alias_and_file_name|Daily Detachment Titles, Alias, and File Name]]

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
