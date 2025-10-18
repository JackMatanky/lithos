---
title: 60_library_status_suggester
aliases:
  - Library Status
  - Library Status Suggester
  - suggester_library_status
  - library status suggester
plugin: templater
language:
  - javascript
module:
  - system
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T16:16
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Library Status Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Set the library resource's status.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript 
// Template file to include
const library_status = "60_library_status";

//---------------------------------------------------------  
// SET LIBRARY STATUS
//---------------------------------------------------------
// Retrieve the Library Status template and content
temp_file_path = `${sys_temp_include_dir}${library_status}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const status_value = include_arr[0];
const status_name = include_arr[1];
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET LIBRARY STATUS
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${library_status}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const status_value = include_arr[0];
const status_name = include_arr[1];
```

#### Referenced Template

```javascript
//---------------------------------------------------------
// SET LIBRARY STATUS
//---------------------------------------------------------
const status_obj_arr = [
  { key: "❓Undetermined", value: "undetermined" },
  { key: "🔜To do", value: "to_do" },
  { key: "👟In progress", value: "in_progress" },
  { key: "✔️Done", value: "done" },
  { key: "🗃️Resource", value: "resource" },
  { key: "📅Schedule", value: "schedule" },
  { key: "🤌On hold", value: "on_hold" },
];

const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  "Resource status?"
);

const status_value = status_obj.value;
const status_name = status_obj.key;

tR += status_value
tR += ","
tR += status_name
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
2. [[71_10_book_search|Book Search Template]]
3. [[71_20_book_chapter|Book Chapter Template]]
4. [[72_journal|Journal Article Template]]
5. [[73_report|Report Article Template]]
6. [[75_webpage|Webpage Template]]
7. [[76_10_video_youtube|YouTube Video Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[60_library_status]]

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

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]

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
