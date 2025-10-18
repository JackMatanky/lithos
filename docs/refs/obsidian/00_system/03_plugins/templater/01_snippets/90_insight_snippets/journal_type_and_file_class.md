---
title: journal_type_and_file_class
aliases:
  - Journal Type and File Class
  - journal type and file class
  - journal_type_and_file_class
plugin: templater
language:
  - javascript
module: 
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T13:52
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Journal Type and File Class

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Assign the journal's full type name, full type value, type name, type, and file class.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// FULL_TYPE_NAME OPTIONS:
// - Reflection Journal
// - Gratitude Journal
// - Detachment Journal
//---------------------------------------------------------  
// JOURNAL TYPE, SUBTYPE AND FILE CLASS
//--------------------------------------------------------- 
const full_type_name = ">>>FULL_TYPE_NAME<<<";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[0];
const type_value = full_type_value.split("_")[0];
const subtype_name = "Null";
const subtype_value = "null";
const file_class = `pdev_${full_type_value.split("_")[1]}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// JOURNAL TYPE, SUBTYPE AND FILE CLASS
//--------------------------------------------------------- 
const full_type_name = ">>>FULL_TYPE_NAME<<<";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[0];
const type_value = full_type_value.split("_")[0];
const subtype_name = "Null";
const subtype_value = "null";
const file_class = `pdev_${full_type_value.split("_")[1]}`;
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

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here  -->

1. [[53_00_action_item_tag_type_file_class|Action Item Tag, Type, and File Class]]
2. [[54_meeting_tag_type_file_class|Meeting Tag, Type, and File Class]]
3. [[journal_daily_reflection_type_and_file_class|Daily Reflection Journal Type and File Class]]
4. [[journal_gratitude_type_and_file_class|Gratitude Journal Type and File Class]]
5. [[journal_daily_gratitude_type_and_file_class|Daily Gratitude Journal Type and File Class]]

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
