---
title: dv_lib_linked
aliases:
  - Linked Library Files Dataview Table
  - Dataview Table for Linked Library Files
  - dv lib linked
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-12T13:46
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Linked Library Files Dataview Table

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a dataview table or markdown table for linked library files based on specific file class and type.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// Title
const yaml_title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias}) AS Title`;

// Title link for DV markdown query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]" AS Title`;

// File type
const yaml_type = `file.frontmatter.type`;
const file_type = `choice(${yaml_type} = "book", "📚Book", 
	choice(${yaml_type} = "book_chapter", "📑Book Chapter", 
	choice(${yaml_type} = "journal", "📜️Journal", 
	choice(${yaml_type} = "report", "📈Report", 
	choice(${yaml_type} = "news", "🗞️News", 
	choice(${yaml_type} = "magazine", "📰️Magazine", 
	choice(${yaml_type} = "webpage", "🌐Webpage", 
	choice(${yaml_type} = "blog", "💻Blog", 
	choice(${yaml_type} = "video", "🎥️Video", 
	choice(${yaml_type} = "youtube", "▶YouTube", 
	choice(${yaml_type} = "documentary", "🖼️Documentary", 
	choice(${yaml_type} = "audio", "🔉Audio", 
	choice(${yaml_type} = "podcast", "🎧️Podcast", "📃Documentation")))))))))))))
	AS Type`;

// Status
const yaml_status = `file.frontmatter.status`;
const status = `choice(${yaml_status} = "undetermined", "❓Undetermined",
	choice(${yaml_status} = "to_do", "🔜To do",
	choice(${yaml_status} = "in_progress", "👟In progress",
	choice(${yaml_status} = "done", "✔️Done",
	choice(${yaml_status} = "resource", "🗃️Resource",
	choice(${yaml_status} = "schedule", "📅Schedule", "🤌On hold"))))))
	AS Status`;

// Author
const author = `Author AS Author`;

// Date published
const date_publish = `choice(contains(file.frontmatter.type, "book"), file.frontmatter.year_published, file.frontmatter.date_published) AS "Date Published"`;

// Tags
const tags = `file.etags AS Tags`;

//---------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//---------------------------------------------------------
// Library directory
const lib_dir = `"60_library"`;

// Inbox directory
const inbox_dir = `"inbox"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTER <<<<<
//---------------------------------------------------------
// Current file filter
const current_file_filter = `file.name != this.file.name`;

// File inlinks and outlinks
const in_out_link_filter = `(contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))`;

// File class filter
const class_filter = `contains(file.frontmatter.file_class, "lib")`;

//---------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//---------------------------------------------------------
const lib_status_sort = `choice(${yaml_status} = "undetermined" OR ${yaml_status} = "schedule", 
		1, 
		choice(${yaml_status} = "to_do", 
			2, 
			choice(${yaml_status} = "in_progress", 
				3, 
				choice(${yaml_status} = "done", 
					4, 
					5))))`;

//---------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE:
// VAR SUBTYPE:

async function dv_lib_linked(type, subtype, md) {
  const type_arg = `${type}`;
  const subtype_arg = `${subtype}`;
  const md_arg = `${md}`;

  const type_filter = `contains(file.frontmatter.file_class, "${type_arg}")`;
  let subtype_filter = `contains(${yaml_type}, "${subtype_arg}")`;

  let dataview_query;

  if (type_arg != "") {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link} AS Title,
	${author},
	${date_publish},
	${file_type},
	${status},
	${tags}
FROM
	${lib_dir}
	OR ${inbox_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
SORT 
	${yaml_type},	
	${yaml_title} ASC
${three_backtick}`;
  } else {
    // Table for linked LIBRARY content
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link} AS Title,
	${author},
	${date_publish},
	${file_type},
	${status},
	${tags}
FROM
	${lib_dir}
	OR ${inbox_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	${yaml_type},	
	${yaml_title} ASC
${three_backtick}`;
  }

  if (md_arg == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    const md_query = String(
      dataview_query
        .replace(dataview_block_start_regex, "")
        .replace(dataview_block_end_regex, "")
        .replaceAll(/\n\s+/g, " ")
        .replaceAll(/\n/g, " ")
        .replace(title_link, md_title_link)
    );

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
  }

  return dataview_query;
}

module.exports = dv_lib_linked;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// LINKED LIBRARY FILES DATAVIEW TABLE
//---------------------------------------------------------
// MD: "true", "false"
// TYPE:
// SUBTYPE:
const linked_lib_file_table = await tp.user.dv_lib_linked(type, subtype, md)
```

#### Examples

```javascript
//---------------------------------------------------------
// LINKED LIBRARY FILES DATAVIEW TABLE
//---------------------------------------------------------
// ALL RELATED LIBRARY FILES TABLES
const linked_lib_content = await tp.user.dv_lib_linked("", "", "false");

```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_00_project|General Project Template]]
2. [[51_00_parent_task|General Parent Task Template]]
3. [[90_00_note|General Note Template]]
4. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
5. [[90_11_note_quote|Quote Fleeting Note Template]]
6. [[90_12_note_idea|Idea Fleeting Note Template]]
7. [[90_20_note_literature(X)|General Literature Note Template]]
8. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
9. [[90_31_note_question|QEC Question Note Template]]
10. [[90_32_note_evidence|QEC Evidence Note Template]]
11. [[90_33_note_conclusion|QEC Conclusion Note Template]]
12. [[90_40_note_lit_psa(X)|PSA Note Template]]
13. [[90_41_note_problem|PSA Problem Note Template]]
14. [[90_42_note_steps|PSA Steps Note Template]]
15. [[90_43_note_answer|PSA Answer Note Template]]
16. [[90_50_note_info(X)|General Info Note Template]]
17. [[90_51_note_concept|Concept Note Template]]
18. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->  

1. [[dv_lib_linked.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_linked_file(X)|Linked File Dataview Table]]
2. [[dv_task_linked|Linked Tasks and Events Files Dataview Table]]
3. [[dv_dir_linked|Linked Directory Files Dataview Table]]
4. [[dv_pkm_linked|Linked Personal Knowledge Files Dataview Table]]
5. [[dv_lib_linked|Linked Library Files Dataview Table]]

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
