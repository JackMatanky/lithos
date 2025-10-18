---
title: organization_title_alias_file_name
aliases:
  - Organization Title, Alias, and File Name
  - organization title, alias, and file name
  - organization title alias file name
plugin: templater
language:
  - javascript
module:
  - 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-27T17:00
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Organization Title, Alias, and File Name

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Assign an organization's title, alias, and file name.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>TODO: DEFINE <title> VARIABLE<<<
// >>>TODO: DEFINE <context_dir> VARIABLE<<<
//---------------------------------------------------------  
// ORGANIZATION TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = title;
const short_title_name = full_title_name.toLowerCase();
const short_title_value = short_title_name.replaceAll(/\s/g, "_");

const alias_arr = `["${full_title_name}", "${short_title_name}", ${short_title_value}]`

const file_name = short_title_value;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// ORGANIZATION TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const full_title_name = title;
const short_title_name = full_title_name.toLowerCase();
const short_title_value = short_title_name
  .replaceAll(/,/g, "")
  .replaceAll(/\s/g, "_")
  .replaceAll(/[\/\+]/g, "-")
  .replaceAll(/&/g, "and");

const alias_arr = `["${full_title_name}", "${short_title_name}", ${short_title_value}]`

const file_name = short_title_value;
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript

```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[62_organization|Organization Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[rename_untitled_file_prompt|Rename Untitled File Prompt]]

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
