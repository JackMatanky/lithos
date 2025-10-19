---
title: book_title_alias_file_name_directory
aliases:
  - Book Title, Alias, File Name, and Directory
  - book title, alias, file name, and directory
  - book title alias file name and directory
  - book title alias file name directory
plugin: templater
language:
  - javascript
module:
  -
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-28T11:04
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Book Title, Alias, File Name, and Directory

## Description

> [!snippet] Snippet Details
>
> Plugin:: [[Templater]]
> Language:: [[JavaScript]]
> Input::
> Output::
> Description:: Assign a book's title, alias, file name, and directory based on author, year published, and full title.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Library books directory
const lib_books_dir = `60_library/71_books/`;

//---------------------------------------------------------
// BOOK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//---------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const full_title_name = lib_content_titles.full_title_name;
const full_title_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sub_title;

const main_title_value = main_title.replaceAll(/\s/g, "_").toLowerCase();

const file_name = `${contact_value.split("_")[0]}_${date_published}_${main_title.replaceAll(/\s/g, "_").toLowerCase()}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}${full_title_value}${new_line}${ul_yaml}${main_title_value}${ul_yaml}${file_name}`;

const book_dir = `${lib_books_dir}${file_name}/`;
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// BOOK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//---------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const full_title_name = lib_content_titles.full_title_name;
const full_title_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sec_title;

const main_title_value = main_title.replaceAll(/\s/g, "_");

const file_name = `${contact_value.split("_")[0]}_${date_published}_${main_title.replaceAll(/\s/g, "_").toLowerCase()}`;

const alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}${full_title_value}${new_line}${ul_yaml}${main_title_value}${ul_yaml}${file_name}`;

const book_dir = `${lib_books_dir}${file_name}/`;
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

1. [[71_00_book|Book Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[lib_content_titles|Library Content Titles]]

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
