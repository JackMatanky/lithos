---
title: 54_meeting_subtype
aliases:
  - Meeting Subtype Suggester
  - meeting subtype suggester
  - meeting subtype
plugin: templater
language:
  - javascript
module:
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-13T17:12
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Meeting Subtype Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return the type of meeting from a suggester

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET MEETING SUBTYPE
//---------------------------------------------------------  
const meeting_obj_arr = [
  { key: `Meeting`, value: `meeting` },
  { key: `Phone Call`, value: `phone_call` },
  { key: `Interview`, value: `interview` },
  { key: `Appointment`, value: `appointment` },
  { key: `Event`, value: `event` },
  { key: `Gathering`, value: `gathering` },
  { key: `Hangout`, value: `hangout` },
];

const meeting_obj = await tp.system.suggester(
  (item) => item.key,
  meeting_obj_arr,
  false,
  `Meeting Type?`
);

const subtype_name = meeting_obj.key;
const subtype_value = meeting_obj.value;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET MEETING SUBTYPE
//---------------------------------------------------------  
const meeting_obj_arr = [
  { key: `Meeting`, value: `meeting` },
  { key: `Phone Call`, value: `phone_call` },
  { key: `Interview`, value: `interview` },
  { key: `Appointment`, value: `appointment` },
  { key: `Event`, value: `event` },
  { key: `Gathering`, value: `gathering` },
  { key: `Hangout`, value: `hangout` },
];
const meeting_obj = await tp.system.suggester(
  (item) => item.key,
  meeting_obj_arr,
  false,
  `Meeting Type?`
);
const subtype_name = meeting_obj.key;
const subtype_value = meeting_obj.value;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[54_00_meeting]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

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
