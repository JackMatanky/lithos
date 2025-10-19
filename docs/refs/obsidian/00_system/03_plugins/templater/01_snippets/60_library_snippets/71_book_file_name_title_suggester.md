---
title: 71_book_file_name_title_suggester
aliases:
  - Book File Name and Title Suggester
  - Book File Name and Title
  - book_file_name_and_title_suggester
  - suggester_book_file_name_and_title
language:
  - javascript
plugin: templater
module:
  - system
  - user
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-05T13:40
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Organization File Name and Title Suggester

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output:: A basename of the chosen organization file
> Description:: Return an organization's file name and the organization's main title from `alias[0]`.

---

## Snippet

```javascript
// Template file to include
const book_name_alias = "61_book_name_alias";

//---------------------------------------------------------
// SET BOOK FILE NAME AND TITLE
//---------------------------------------------------------
// Retrieve the Book File Name and Alias template and content
temp_file_path = `${sys_temp_include_dir}${book_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const book_value = include_arr[0];
const book_name = include_arr[1];
const book_link = `${book_value}|${book_name}`;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET BOOK FILE NAME AND TITLE
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${book_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const book_value = include_arr[0];
const book_name = include_arr[1];
const book_link = `${book_value}|${book_name}`;
```

#### Referenced Template

```javascript
//---------------------------------------------------------
// SET BOOK
//---------------------------------------------------------
// Books directory
const books_dir = "60_library/71_books/";

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

1. [[50_32_proj_ed_book|Education Book Project Template]]
2. [[71_00_book|Book Template]]
3. [[71_20_book_chapter|Book Chapter Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[71_book_name_alias]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[get_md_file_names|Markdown File Names for Suggester]]
2. [[get_md_file_titles_names|Markdown File Names and Titles for Suggester]]
3. [[61_contact_file_name_title_suggester|Contact File Name and Title Suggester]]
4. [[62_organization_file_name_title_suggester|Organization File Name and Title Suggester]]
5. [[10_pillar_file_name_title_suggester|Pillar File Name and Title Suggester]]

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

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]
2. [[tp.file.include Templater Function|The Templater tp.file.include() Function]]

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
