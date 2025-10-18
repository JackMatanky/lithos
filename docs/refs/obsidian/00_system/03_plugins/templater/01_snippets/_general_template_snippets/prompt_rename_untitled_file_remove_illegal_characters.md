---
title: prompt_rename_untitled_file_remove_illegal_characters
aliases:
  - Prompt Rename Untitled File and Remove Illegal Characters
  - Untitled File Prompt Rename and Remove Illegal Characters
plugin: templater
language:
  - javascript
module:
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Prompt Rename Untitled File and Remove Illegal Characters

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description::

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET FILE'S TITLE
//---------------------------------------------------------
const has_title = !tp.file.title.startsWith(`Untitled`);
let title;

if (!has_title) {
  title = await tp.system.prompt(`Title`, null, true, false);
} else {
  title = tp.file.title;
}

title = title.trim();

//---------------------------------------------------------
// CONTENT TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
let subtitle;
let full_title;
let file_name;
let alias;

if (title.includes(`:`)) {
  title_split = title.split(`:`);
  title = title_split[0].trim();
  subtitle = title_split[1].trim();
  full_title = `${title}: ${subtitle}`;
  file_name = `${title
    .replaceAll(/[#:\*<>\|\\/-]/g, "_")
    .replaceAll(/\?/g, "")
    .replaceAll(/"/g, "'")}_${subtitle
    .replaceAll(/[#:\*<>\|\\/-]/g, "_")
    .replaceAll(/\?/g, "")
    .replaceAll(/"/g, "'")}`;
  alias = file_name.toLowerCase();
} else {
  full_title = title;
  file_name = `${full_title
    .replaceAll(/[#:\*<>\|\\/-]/g, "_")
    .replaceAll(/\?/g, "")
    .replaceAll(/"/g, "'")}`;
  alias = file_name.toLowerCase();
}

const alias_arr = `${new_line}${ul_yaml}${file_name}${ul_yaml}${alias}${new_line}${ul_yaml}"${full_title}"`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

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
