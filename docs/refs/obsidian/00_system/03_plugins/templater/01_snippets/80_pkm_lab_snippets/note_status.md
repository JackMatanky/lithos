---
title: note_status
aliases:
  - Note Status Suggester
  - note status suggester
  - note status
plugin: templater
language:
  - javascript
module:
  - system
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-20T13:29
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Note Status Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input:: Object Array
> Output:: String
> Description:: Return the note status from a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const note_status = `80_note_status`;

//---------------------------------------------------------
// SET NOTE STATUS
//---------------------------------------------------------
// Retrieve the Note Status template and content
temp_file_path = `${sys_temp_include_dir}${note_status}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const status_name = include_arr[0];
const status_value = include_arr[1];
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET NOTE STATUS
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${note_status}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const status_name = include_arr[0];
const status_value = include_arr[1];
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// SET NOTE STATUS
//---------------------------------------------------------
const status_obj_arr = [
  { key: "🌱️Review", value: "review" },
  { key: "🌿️Clarify", value: "clarify" },
  { key: "🪴Develop", value: "develop" },
  { key: "🌳Evergreen", value: "evergreen" },
  { key: "🗃️Resource", value: "resource" },
];

const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  "Status?"
);

const status_name = status_obj.key;
const status_value = status_obj.value;

tR += status_name
tR += ","
tR += status_value
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[90_00_note|General Note Template]]
2. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
3. [[90_11_note_quote|Quote Fleeting Note Template]]
4. [[90_12_note_idea|Idea Fleeting Note Template]]
5. [[90_20_note_literature(X)|General Literature Note Template]]
6. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
7. [[90_31_note_question|QEC Question Note Template]]
8. [[90_32_note_evidence|QEC Evidence Note Template]]
9. [[90_33_note_conclusion|QEC Conclusion Note Template]]
10. [[90_40_note_lit_psa(X)|PSA Note Template]]
11. [[90_41_note_problem|PSA Problem Note Template]]
12. [[90_42_note_steps|PSA Steps Note Template]]
13. [[90_43_note_answer|PSA Answer Note Template]]
14. [[90_50_note_info(X)|General Info Note Template]]
15. [[90_51_note_concept|Concept Note Template]]
16. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[80_note_status]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[note_type_subtype_file_class|Note Type, Subtype, and File Class Suggester]]

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
