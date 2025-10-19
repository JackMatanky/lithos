---
title: book_chapter_title_alias_file_name
aliases:
  - Book Chapter Title, Alias, and File Name
  - book chapter title, alias, and file name
  - book chapter title alias and file name
  - book chapter title alias file name
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-28T11:04
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Book Chapter Title, Alias, and File Name

## Description

> [!snippet] Snippet Details
>
> Plugin:: [[Templater]]
> Language:: [[JavaScript]]
> Input::
> Output::
> Description:: Assign a book chapter's title, alias, and file name based on the chapter title, chapter number, and book main title.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// BOOK CHAPTER TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const full_title_name = lib_content_titles.full_title_name;
const full_title_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sub_title;

const main_title_value = main_title.replaceAll(/\s/g, "_").toLowerCase();

const book_chapter_title_name = `${book_main_title}: ${main_title}`;
const book_chapter_title_value = `${book_main_title_value}_${main_title_value}`;

const file_name = `${chapter_number}_${main_title_value}_${book_main_title_value}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}${full_title_value}${new_line}${ul_yaml}${main_title_value}${ul_yaml}"${book_chapter_title_name}"${new_line}${ul_yaml}${book_chapter_title_value}${ul_yaml}${file_name}`;

```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// BOOK CHAPTER TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const full_title_name = lib_content_titles.full_title_name;
const full_title_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sub_title;

const main_title_value = main_title.replaceAll(/\s/g, "_").toLowerCase();

const book_chapter_title_name = `${book_main_title}: ${main_title}`;
const book_chapter_title_value = `${book_main_title_value}_${main_title_value}`;

const file_name = `${chapter_number}_${main_title_value}_${book_main_title_value}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}${full_title_value}${new_line}${ul_yaml}${main_title_value}${ul_yaml}"${book_chapter_title_name}"${new_line}${ul_yaml}${book_chapter_title_value}${ul_yaml}${file_name}`;

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

1. [[71_20_book_chapter|Book Chapter Template]]

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
