---
title: "Contact Full, First, Last, and Maiden Names"
aliases:
  - Contact Full, First, Last, and Maiden Names
  - contact_full_first_last_maiden_names
plugin: templater
language:
  - javascript
module:
  - 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Contact Full, First, Last, and Maiden Names

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a contact's first, last, and maiden name from their full name.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET THE CONTACT'S NAME AND FILE'S TITLE
//---------------------------------------------------------
const full_name = await tp.system.prompt(
  "What is the contact's name?",
  "",
  true,
  false
);
const names = full_name.split(" ");
const name_first = names[0].trim();
let name_last;
let name_last_maiden;

if (names.length >= 3) {
  name_last = `${names[1].trim()} ${names[2].trim()}`;
  name_last_maiden = names[1].trim();
} else if (names.length >= 2) {
  name_last = names[1].trim();
  if (name_last.split(`-`).length == 2) {
    name_last_maiden = name_last.split(`-`)[0].trim();
  }
}
const file_name = `${name_last
  .replaceAll(/[^\w]/g, "_")
  .toLowerCase()}_${name_first.replaceAll(/[^\w]/g, "_").toLowerCase()}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET THE CONTACT'S NAME AND FILE'S TITLE
//---------------------------------------------------------
const full_name = await tp.system.prompt(
  `What is the contact's name?`,
  ``,
  true,
  false
);
const names = full_name.split(` `);
const name_first = names[0].trim();
let name_last;
let name_last_maiden;

if (names.length >= 3) {
  name_last = `${names[1].trim()} ${names[2].trim()}`;
  name_last_maiden = names[1].trim();
} else if (names.length >= 2) {
  name_last = names[1].trim();
  if (name_last.split(`-`).length == 2) {
    name_last_maiden = name_last.split(`-`)[0].trim();
  }
}
const file_name = `${name_last
  .replaceAll(/[^\w]/g, "_")
  .toLowerCase()}_${name_first.replaceAll(/[^\w]/g, "_").toLowerCase()}`;
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

1. [[tp.system.prompt Templater Function|The Templater tp.system.prompt() Function]]

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition, 
	file.frontmatter.syntax AS Syntax
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
