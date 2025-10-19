---
title: 54_meeting_tag_type_file_class
aliases:
  - Meeting Tag, Type, and File Class
  - meeting tag, type, and file class
  - meeting tag type and file class
  - meeting tag type file class
plugin: templater
language:
  - javascript
module:
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T16:19
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Meeting Tag, Type, and File Class

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Assign the meeting's full type name, short type name, and file class.

---

## Snippet

```javascript
//---------------------------------------------------------
// MEETING TAG, TYPE, AND FILE CLASS
//---------------------------------------------------------
const task_tag = `#task`;
const type_name = `Meeting`;
const type_value = type_name.toLowerCase();
const file_class = `task_${type_value}`;
```

### Templater

```javascript
//---------------------------------------------------------
// MEETING TAG, TYPE, AND FILE CLASS
//---------------------------------------------------------
const task_tag = `#task`;
const type_name = `Meeting`;
const type_value = type_name.toLowerCase();
const file_class = `task_${type_value}`;
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

1. [[54_meeting_subtype|Meeting Subtype Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[53_00_action_item_tag_type_file_class|Action Item Tag, Type, and File Class]]

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
