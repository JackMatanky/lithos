---
title: Contact Connection Type Suggester
aliases:
  - Contact Connection Type Suggester
  - Suggester for Contact Connection Type
  - contact_connection_type_suggester
  - suggester_contact_connection_type
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
# Contact Connection Type Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Set the type of connection with a contact using a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET CONNECTION TYPE
//---------------------------------------------------------
const connection_obj_arr = [
  { name: `Personal`, value: `personal` },
  { name: `Professional`, value: `professional` },
  { name: `Academic`, value: `academic` },
  { name: `None`, value: `none` },
];

const connection_obj = await tp.system.suggester(
  (item) => item.name,
  connection_obj_arr,
  false,
  `What is the connection type between the contact and me?`
);

const connection_name = connection_obj.name;
const connection_value = connection_obj.value;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET CONNECTION TYPE
//---------------------------------------------------------
const connection_obj_arr = [
  { name: `Personal`, value: `personal` },
  { name: `Professional`, value: `professional` },
  { name: `Academic`, value: `academic` },
  { name: `None`, value: `none` },
];
const connection_obj = await tp.system.suggester(
  (item) => item.name,
  connection_obj_arr,
  false,
  `What is the connection type between the contact and me?`
);
const connection_name = connection_obj.name;
const connection_value = connection_obj.value;
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
