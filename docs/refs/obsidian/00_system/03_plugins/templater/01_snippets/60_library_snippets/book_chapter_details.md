---
title: book_chapter_details
aliases:
  - Book Chapter Details
  - book chapter details
plugin: templater
language:
  - javascript
module:
  - user
  - file
cssclasses: null
type: snippet
file_class: pkm_code
date_created: 2023-06-28T11:04
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Book Chapter Details

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a book's chapters' number, title, start page, and end page.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// BOOK CHAPTER DETAILS
//---------------------------------------------------------
const chapter_details_input = (
  await tp.system.prompt("Chapter Details Object Array", null, false, true)
).split(";");

const number_regex = /(.+_number:\s")(\d{1,4})(",\s.+)/g;
const title_regex = /(.+title:\s")(.+?)(",\s.+)/g;
const page_start_regex = /(.+page_start:\s")(\d{1,4}|[xvi].+?)(",\s.+)/g;
const page_end_regex = /(.+page_end:\s")(\d{1,4}|[xvi].+?)("\})/g;

let chapter_details_obj_arr = [];
for (var i = 0; i < chapter_details_input.length - 1; i++) {
  chapter_details = chapter_details_input[i];
  chapter_number = chapter_details
    .replace(number_regex, "$2")
    .replaceAll(/[\n\s]/g, "");
  chapter_title = chapter_details
    .replace(title_regex, "$2")
    .replaceAll(/^[\n\s]|[\n\s]$/g, "");
  chapter_page_start = chapter_details
    .replace(page_start_regex, "$2")
    .replaceAll(/[\n\s]/g, "");
  chapter_page_end = chapter_details
    .replace(page_end_regex, "$2")
    .replaceAll(/[\n\s]/g, "");
  chapter_obj = {
    number: chapter_number,
    title: chapter_title,
    page_start: chapter_page_start,
    page_end: chapter_page_end,
  };
  chapter_details_obj_arr.push(chapter_obj);
}
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// BOOK CHAPTER DETAILS
//---------------------------------------------------------
const chapter_details_input = (
  await tp.system.prompt("Chapter Details Object Array", null, false, true)
).split(";");

const number_regex = /(.+_number:\s")(\d{1,4})(",\s.+)/g;
const title_regex = /(.+title:\s")(.+?)(",\s.+)/g;
const page_start_regex = /(.+page_start:\s")(\d{1,4}|[xvi].+?)(",\s.+)/g;
const page_end_regex = /(.+page_end:\s")(\d{1,4}|[xvi].+?)("\})/g;

let chapter_details_obj_arr = [];
for (var i = 0; i < chapter_details_input.length - 1; i++) {
  chapter_details = chapter_details_input[i];
  chapter_number = chapter_details
    .replace(number_regex, "$2")
    .replaceAll(/[\n\s]/g, "");
  chapter_title = chapter_details
    .replace(title_regex, "$2")
    .replaceAll(/^[\n\s]|[\n\s]$/g, "");
  chapter_page_start = chapter_details
    .replace(page_start_regex, "$2")
    .replaceAll(/[\n\s]/g, "");
  chapter_page_end = chapter_details
    .replace(page_end_regex, "$2")
    .replaceAll(/[\n\s]/g, "");
  chapter_obj = {
    number: chapter_number,
    title: chapter_title,
    page_start: chapter_page_start,
    page_end: chapter_page_end,
  };
  chapter_details_obj_arr.push(chapter_obj);
}
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

1. [[71_01_book_all_chapters|Book and Chapters Template]]
2. [[71_21_all_book_chapter|All Book Chapters Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[lib_content_titles|Library Content Titles]]
2. [[book_file_name_title_directory_suggester|Book File Name, Title Name, and Directory Suggester]]
3. [[book_chapter_number_suggester|Book Chapter Number Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[book_title_alias_file_name_directory|Book Title, Alias, File Name, and Directory]]

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
