---
title: note_type_subtype_file_class 
aliases:
  - Note Type, Subtype, and File Class Suggester
  - note type, subtype, and file class suggester
  - note type subtype and file class suggester
  - note type subtype file class suggester
  - note type subtype file class
plugin: templater
language:
  - javascript
module:
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-20T13:08
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Note Type, Subtype, and File Class Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input:: Object Array  
> Output:: String  
> Description:: Return a note's type, subtype, and file class with a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET NOTE TYPE, SUBTYPE, AND FILE CLASS
//---------------------------------------------------------
const note_type_obj_arr = [
  {
    key: "Quote (Fleeting)",
    subtype: "quote",
    type: "fleeting",
    file_class: "pkm_zettel",
  },
  {
    key: "Idea (Fleeting)",
    subtype: "idea",
    type: "fleeting",
    file_class: "pkm_zettel",
  },
  {
    key: "Question (LIT-QEC)",
    subtype: "question",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Evidence (LIT-QEC)",
    subtype: "evidence",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Conclusion (LIT-QEC)",
    subtype: "conclusion",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Problem (LIT-PSA)",
    subtype: "question",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Steps (LIT-PSA)",
    subtype: "steps",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Answer (LIT-PSA)",
    subtype: "answer",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Concept",
    subtype: "null",
    type: "concept",
    file_class: "pkm_info",
  },
  {
    key: "Definition",
    subtype: "null",
    type: "definition",
    file_class: "pkm_info",
  },
  {
    key: "General",
    subtype: "null",
    type: "general",
    file_class: "pkm",
  },
  {
    key: "Permanent",
    value: "permanent",
    file_class: "pkm_zettel",
  },
];

const note_type_obj = await tp.system.suggester(
  (item) => item.key,
  note_type_obj_arr,
  false,
  "Note type?"
);

const type_name = note_type_obj.key;
const type_value = note_type_obj.type;
const subtype_value = note_type_obj.subtype;
const file_class = note_type_obj.file_class;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET NOTE TYPE, SUBTYPE, AND FILE CLASS
//---------------------------------------------------------
const note_type_obj_arr = [
  {
    key: "Quote (Fleeting)",
    subtype: "quote",
    type: "fleeting",
    file_class: "pkm_zettel",
  },
  {
    key: "Idea (Fleeting)",
    subtype: "idea",
    type: "fleeting",
    file_class: "pkm_zettel",
  },
  {
    key: "Question (LIT-QEC)",
    subtype: "question",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Evidence (LIT-QEC)",
    subtype: "evidence",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Conclusion (LIT-QEC)",
    subtype: "conclusion",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Problem (LIT-PSA)",
    subtype: "question",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Steps (LIT-PSA)",
    subtype: "steps",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Answer (LIT-PSA)",
    subtype: "answer",
    type: "literature",
    file_class: "pkm_zettel",
  },
  {
    key: "Concept",
    subtype: "null",
    type: "concept",
    file_class: "pkm_info",
  },
  {
    key: "Definition",
    subtype: "null",
    type: "definition",
    file_class: "pkm_info",
  },
  {
    key: "General",
    subtype: "null",
    type: "general",
    file_class: "pkm",
  },
  {
    key: "Permanent",
    value: "permanent",
    file_class: "pkm_zettel",
  },
];
const note_type_obj = await tp.system.suggester(
  (item) => item.key,
  note_type_obj_arr,
  false,
  "Note type?"
);
const type_name = note_type_obj.key;
const type_value = note_type_obj.type;
const subtype_value = note_type_obj.subtype;
const file_class = note_type_obj.file_class;
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

1. [[90_00_note]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

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
