---
title: book_file_name_title_directory_suggester
aliases:
  - Book File Name, Title Name, and Directory Suggester
  - book file name, title, and directory suggester
  - Book File, Full Name, and Directory
  - book_file_full_name_directory
  - book_file_full_name_directory_suggester
  - book file name title directory suggester
plugin: templater
language:
  - javascript
module:
  - system
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-27T15:27
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Book File Name, Title Name, and Directory Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a book's file name and title from `alias[0]` and assign a chapter's book directory with the book's file name.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Library books directory
const lib_books_dir = `60_library/71_books/`;

// Template file to include
const book_name_alias = `71_book_name_alias`;

//---------------------------------------------------------
// SET BOOK AND DIRECTORY
//---------------------------------------------------------
// Retrieve the Book File Name and Alias template and content
temp_file_path = `${sys_temp_include_dir}${book_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const book_value = include_arr[0];
const book_name = include_arr[1];
const book_link = `${book_value}|${book_name}`;

const book_dir = `${lib_books_dir}${book_value}/`;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET BOOK AND DIRECTORY
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${book_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const book_value = include_arr[0];
const book_name = include_arr[1];
const book_link = `${book_value}|${book_name}`;

const book_dir = `${lib_books_dir}${book_value}/`;
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// SET BOOK
//---------------------------------------------------------
// Books directory
const books_dir = `60_library/71_books/`;

// Filter array to only include books in the Book Directory
const books_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: books_dir,
    file_class: "lib",
    type: "book",
  });
  
const books_obj = await tp.system.suggester(
  (item) => item.key,
  books_obj_arr,
  false,
  "Book?"
  );
  
const book_value = books_obj.value;
const book_name = books_obj.key;

tR += book_value
tR += ","
tR += book_name
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

1. [[file_name_alias_by_class_type|Markdown File Names Filtered by Directory, File Class, and Type Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[10_pillar_file_name_title_suggester|Pillar File Name and Title Name Suggester]]
2. [[related_project_suggester|Related Project Suggester]]
3. [[related_parent_task_suggester|Related Parent Task Suggester]]

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
