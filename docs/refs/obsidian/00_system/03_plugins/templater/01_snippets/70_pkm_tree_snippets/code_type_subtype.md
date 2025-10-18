---
title: code_type_subtype
aliases:
  - Code Note Type and Subtype
  - Code Note Type and Subtype Suggester
  - code_note_type_and_subtype
  - code_note_type_and_subtype_suggester
  - code type subtype
plugin: templater
language:
  - javascript
module:
  - system
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-20T13:15
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Code Note Type and Subtype Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input:: Object Array  
> Output:: String  
> Description:: Return the knowledge tree objects' name and link from a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const code_type_subtype = "71_code_type_subtype";

//---------------------------------------------------------
// SET CODE FILE TYPE AND SUBTYPE
//---------------------------------------------------------
// Retrieve the Code Type and Subtype template and content
temp_file_path = `${sys_temp_include_dir}${code_type_subtype}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const type_value = include_arr[0];
const type_name = include_arr[1];
const subtype_value = include_arr[2];
const subtype_name = include_arr[3];
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET CODE FILE TYPE AND SUBTYPE
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${code_type_subtype}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const type_value = include_arr[0];
const type_name = include_arr[1];
const subtype_value = include_arr[2];
const subtype_name = include_arr[3];
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// SET TYPE
//---------------------------------------------------------
const type_obj_arr = [
  { key: "Data type", value: "data_type" },
  { key: "Error", value: "error" },
  { key: "Function", value: "function" },
  { key: "Keyword", value: "keyword" },
  { key: "Method", value: "method" },
  { key: "Operator", value: "operator" },
  { key: "Snippet", value: "snippet" },
  { key: "Statement", value: "statement" },
];

const type_obj = await tp.system.suggester(
  (item) => item.key,
  type_obj_arr,
  false,
  "Code note type?"
);

const type_value = type_obj.value;
const type_name = type_obj.key;

//---------------------------------------------------------
// SET SUBTYPE
//---------------------------------------------------------
const subtype_obj_arr = [
  { key: "Null", value: "null" },
  { key: "Array", value: "array" },
  { key: "Boolean", value: "boolean" },
  { key: "DataFrame", value: "dataframe" },
  { key: "Dictionary", value: "dictionary" },
  { key: "File", value: "file" },
  { key: "List", value: "list" },
  { key: "Numeric", value: "numeric" },
  { key: "Object", value: "object" },
  { key: "Series", value: "series" },
  { key: "Set", value: "set" },
  { key: "String", value: "string" },
  { key: "Tuple", value: "tuple" },
  { key: "RegEx", value: "regex" },
  { key: "General", value: "general" },
];

const subtype_obj = await tp.system.suggester(
  (item) => item.key,
  subtype_obj_arr,
  false,
  `${type_name} subtype?`
);

const subtype_value = subtype_obj.value;
const subtype_name = subtype_obj.key;

tR += type_value;
tR += ";";
tR += type_name;
tR += ";";
tR += subtype_value;
tR += ";";
tR += subtype_name;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[84_01_code_keyword|General Code Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[71_code_type_subtype]]

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
