---
title: 71_01_book_chapter_info
aliases:
  - Book Chapter Info Callout
  - Book Chapter Info
  - book chapter info
plugin: templater
language:
  - javascript
module:
  - file
cssclasses: 
type: snippet
file_class: pkm_code
date_created: 2023-06-22T14:38
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Book Chapter Info Callout

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a callout for Book Chapter Info callout.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const book_chapter_info_callout = "61_01_book_chapter_info_callout";

//---------------------------------------------------------  
// BOOK CHPATER INFO CALLOUT
//---------------------------------------------------------
// Retrieve the Book Chapter Info template and content
temp_file_path = `${sys_temp_include_dir}${book_chapter_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const book_chapter_info = include_arr;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// BOOK CHPATER INFO CALLOUT
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${book_chapter_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const book_chapter_info = include_arr;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[71_00_book]]
2. [[71_01_book_all_chapters]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[71_01_book_chapter_info_callout]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[50_00_proj_review_kiss|Project Review KISS Framework]]
2. [[53_00_action_item_preview|Before Action Preview]]
3. [[53_00_action_item_review|After Action Review]]

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
