---
title: dv_dir_linked
aliases:
  - Linked Directory Files Dataview Table
  - Dataview Table for Linked Directory Files
  - dv dir linked
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
# Linked Directory Files Dataview Table

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a dataview table or markdown table for linked contact or organization files based on specific file class.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// file title name and link
const title_link = `link(file.name, file.frontmatter.aliases[0])`;

// Tags
const tags = `file.etags AS Tags`;

// CONTACT
const job_title = `file.frontmatter.job_title AS "Job Title"`;

const organization = `Organization AS Organization`;

// ORGANIZATION
const website = `file.frontmatter.website AS Website`;

const linkedin = `file.frontmatter.linkedin AS LinkedIn`;

const org_about = `file.frontmatter.about AS About`;

//---------------------------------------------------------
// SECT: >>>>> DATA SOURCE
//---------------------------------------------------------
// Template directory
const template_dir = `-"00_system/05_templates"`;

// Organization and Contacts directory
const directory_dir = `"50_directory"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTER
//---------------------------------------------------------
// Current file filter
const current_file_filter = `file.name != this.file.name`;

// File inlinks and outlinks
const in_out_link_filter = `(contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))`;

// File class filter
const class_filter = `contains(file.frontmatter.file_class, "dir")`;

//---------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

// VAR MD: "true", "false"
// VAR TYPE: "contact", "organization"

async function dv_dir_linked(type, md) {
  const type_arg = `${type}`;
  const md_arg = `${md}`;

  const type_filter = `contains(file.frontmatter.file_class, "${type_arg}")`;

  let dataview_query;
  if (md_arg != "true") {
    if (type_arg.startsWith("cont")) {
      // Table for linked CONTACTS
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Name,
	${job_title},
	${organization},
	${tags}
FROM
	${template_dir}
	AND ${directory_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${in_out_link_filter}
SORT
	file.frontmatter.title ASC
${three_backtick}`;
    } else {
      // Table for linked ORGANIZATIONS
      dataview_query = `${three_backtick}dataview
TABLE WITHOUT ID
	${title_link} AS Name,
	${website},
	${linkedin},
	${org_about},
	${tags}
FROM
	${template_dir}
	AND ${directory_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${in_out_link_filter}
SORT
	file.frontmatter.title ASC
${three_backtick}`;
    }
  }
  return dataview_query;
}

module.exports = dv_dir_linked;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// LINKED DIRECTORY FILES DATAVIEW TABLE
//---------------------------------------------------------
// MD: "true", "false"
// TYPE: "contact", "organization"
const linked_dir_file_table = await tp.user.dv_dir_linked(type, md)
```

#### Examples

```javascript
//---------------------------------------------------------
// LINKED DIRECTORY FILES DATAVIEW TABLE
//---------------------------------------------------------
// RELATED CONTACT FILES DATAVIEW TABLE
const linked_dir_contact = await tp.user.dv_dir_linked("contact", "false");

// RELATED ORGANIZATION FILES DATAVIEW TABLE
const linked_dir_org = await tp.user.dv_dir_linked("organization", "false");
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

1. [[dv_dir_linked.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_linked_file(X)|Linked File Dataview Table]]
2. [[dv_task_linked|Linked Tasks and Events Files Dataview Table]]
3. [[dv_lib_linked|Linked Library Files Dataview Table]]
4. [[dv_pkm_linked|Linked Personal Knowledge Files Dataview Table]]

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
