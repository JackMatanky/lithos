---
title: Contact Gender Suggester
aliases:
  - Contact Gender Suggester
  - Suggester for Contact Gender
  - contact_gender_suggester
  - suggester_contact_gender
plugin: templater
language:
  - javascript
module:
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-08T12:46
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Contact Gender Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Set the source of connection with a contact using a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET GENDER
//---------------------------------------------------------
const gender_obj_arr = [
  { name: `Female`, value: `female` },
  { name: `Male`, value: `male` },
  { name: `Other`, value: `other` },
];

const gender_obj = await tp.system.suggester(
  (item) => item.name,
  gender_obj_arr,
  false,
  `What is the contact's gender?`
);

const gender_name = gender_obj.name;
const gender_value = gender_obj.value;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET GENDER
//---------------------------------------------------------
const gender_obj_arr = [
  { name: `Female`, value: `female` },
  { name: `Male`, value: `male` },
  { name: `Other`, value: `other` },
];
const gender_obj = await tp.system.suggester(
  (item) => item.name,
  gender_obj_arr,
  false,
  `What is the contact's gender?`
);
const gender_name = gender_obj.name;
const gender_value = gender_obj.value;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[61_contact]]

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

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]

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
