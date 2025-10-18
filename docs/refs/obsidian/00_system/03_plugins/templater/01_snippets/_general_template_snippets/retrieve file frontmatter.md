---
title: retrieve file frontmatter 
aliases:
  - "Retrieve a File's Frontmatter"
  - retrieve_file_frontmatter
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-12T18:22
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/api, obsidian/tp/file/find_tfile
---
# Retrieve a File's Frontmatter

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Retrieve a file's frontmatter data.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>TODO: DEFINE <FILE_NAME> VARIABLE<<<
// >>>TODO: DEFINE <YAML_KEY> VARIABLE<<<
//---------------------------------------------------------
// RETURN A FILE'S FRONTMATTER DATA
//---------------------------------------------------------
let file = tp.file.find_tfile(`<FILE_NAME>.md`);
let fileCache = await app.metadataCache.getFileCache(file);
const data_field = fileCache?.frontmatter?.<YAML_KEY>;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// RETURN A FILE'S FRONTMATTER DATA
//---------------------------------------------------------
let file = tp.file.find_tfile(`<FILE_NAME>.md`);
let file_cache = await app.metadataCache.getFileCache(file);
const data_field = file_cache?.frontmatter?.<YAML_KEY>;
```

#### Example

```javascript
//---------------------------------------------------------
// RETURN A BOOK'S FRONTMATTER DATA
//---------------------------------------------------------
let file = tp.file.find_tfile(`${book_value}.md`);
let file_cache = await app.metadataCache.getFileCache(file);
const contact_value = file_cache?.frontmatter?.author;
const date_published = file_cache?.frontmatter?.date_published;
const organization_value = file_cache?.frontmatter?.publisher;
const url = file_cache?.frontmatter?.url;
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

1. [[10_pillar_file_name_title_suggester|Pillar File Name and Title Suggester]]

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

1. [[tp.file.find_tfile Templater Function|The Templater tp.file.find_tfile() Function]]
2. [[MetadataCache]]

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
