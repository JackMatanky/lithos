---
title: book_chapter_number_suggester
aliases:
  - Book Chapter Number Suggester
  - book chapter number suggester
plugin: templater
language:
  - javascript
module:
  - file
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-28T11:04
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include, obsidian/tp/system/suggester
---
# Book Chapter Number Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin:: [[Templater]]
> Language:: [[JavaScript]]
> Input::
> Output::
> Description:: Assign a book chapter's number.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const book_chapter_number = `71_book_chapter_number`;

//---------------------------------------------------------
// SET CHAPTER NUMBER
//---------------------------------------------------------
// Retrieve the Book Chapter Number template and content
temp_file_path = `${sys_temp_include_dir}${book_chapter_number}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const chapter_number = include_arr;
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET CHAPTER NUMBER
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${book_chapter_number}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const chapter_number = include_arr;
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// SET CHAPTER NUMBER
//---------------------------------------------------------
const chapter_number_arr = [
  "00",
  "01",
  "02",
  "03",
  "04",
  "05",
  "06",
  "07",
  "08",
  "09",
  "10",
  "11",
  "12",
  "13",
  "14",
  "15",
  "16",
  "17",
  "18",
  "19",
  "20",
  "21",
  "22",
  "23",
  "24",
  "25",
  "26",
  "27",
  "28",
  "29",
  "30",
];

const chapter_number = await tp.system.suggester(
  chapter_number_arr,
  chapter_number_arr,
  false,
  "Chapter number?"
);

tR += chapter_number;
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

1. [[71_book_chapter_number]]
2. [[book_chapter_title_alias_file_name|Book Chapter Title, Alias, and File Name]]

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
