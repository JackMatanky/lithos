---
title: yaml_multiline
aliases:
  - Multiline YAML Field
  - multiline yaml field
  - Prompt for Multiline YAML Field
  - prompt for multiline yaml field
  - yaml multiline
  - yaml_multiline
plugin: templater
language:
  - javascript
module:
  - system
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-29T11:24
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Multiline YAML Field

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a multiline YAML field, using a prompt.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET MULTILINE YAML FIELD
//---------------------------------------------------------
const about = await tp.system.prompt(
  "Description?",
  null,
  false,
  true
);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, "$2")
  .replaceAll(/(\s*)\n/g, "\n")
  .replaceAll(/([^\s])(\s*)$/g, "$1")
  .replaceAll(/\n\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n/g, "<new_line>")
  .replaceAll(/\n/g, "<new_line>")
  .replaceAll(/(<new_line>)(\w)/g, "\n\n $2")
  .replaceAll(/(<new_line>)^(-\s|\d\.\s)/g, "\n $2");
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET MULTILINE YAML FIELD
//---------------------------------------------------------
const about = await tp.system.prompt(
  "Description?",
  null,
  false,
  true
);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, "$2")
  .replaceAll(/(\s*)\n/g, "\n")
  .replaceAll(/([^\s])(\s*)$/g, "$1")
  .replaceAll(/\n\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n/g, "<new_line>")
  .replaceAll(/\n/g, "<new_line>")
  .replaceAll(/(<new_line>)(\w)/g, "\n\n $2")
  .replaceAll(/(<new_line>)^(-\s|\d\.\s)/g, "\n $2");
```

#### Examples

```javascript
//---------------------------------------------------------
// SET THE ORGANIZATION'S LINKEDIN ABOUT
//---------------------------------------------------------
const about = await tp.system.prompt(
  "Organization's LinkedIn 'about'?",
  null,
  false,
  true
);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, "$2")
  .replaceAll(/(\s*)\n/g, "\n")
  .replaceAll(/([^\s])(\s*)$/g, "$1")
  .replaceAll(/\n\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n/g, "<new_line>")
  .replaceAll(/\n/g, "<new_line>")
  .replaceAll(/(<new_line>)(\w)/g, "\n\n $2")
  .replaceAll(/(<new_line>)^(-\s|\d\.\s)/g, "\n $2");
```

```javascript
//---------------------------------------------------------
// SET THE BOOK'S DESCRIPTION
//---------------------------------------------------------
const about = await tp.system.prompt(
  "Book Description?",
  null,
  false,
  true
);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, "$2")
  .replaceAll(/(\s*)\n/g, "\n")
  .replaceAll(/([^\s])(\s*)$/g, "$1")
  .replaceAll(/\n\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n\n/g, "<new_line>")
  .replaceAll(/\n\n/g, "<new_line>")
  .replaceAll(/\n/g, "<new_line>")
  .replaceAll(/(<new_line>)(\w)/g, "\n\n $2")
  .replaceAll(/(<new_line>)^(-\s|\d\.\s)/g, "\n $2");
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[61_contact|Contact Template]]
2. [[62_organization|Organization Template]]
3. [[71_00_book|Book Template]]

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
