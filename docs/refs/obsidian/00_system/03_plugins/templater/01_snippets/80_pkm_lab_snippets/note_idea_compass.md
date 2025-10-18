---
title: note_idea_compass
aliases:
  - Note Idea Compass Callouts
  - note idea compass callouts
  - note idea compass
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-22T08:04
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Note Idea Compass Callouts

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return the note idea compass section formatted with headings and callouts.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const note_idea_compass = "80_note_idea_compass";

//---------------------------------------------------------
// NOTE IDEA COMPASS
//---------------------------------------------------------
// Retrieve the Note Idea Compass template and content
temp_file_path = `${sys_temp_include_dir}${note_idea_compass}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const idea_compass = include_arr;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// NOTE IDEA COMPASS
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${note_idea_compass}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const idea_compass = include_arr;
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const head = String.fromCodePoint(0x23);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const four_space = space.repeat(4);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const hr_line = String.fromCodePoint(0x2d).repeat(3);
const cmnt_ob_start = `${String.fromCodePoint(37).repeat(2)}${space}`;
const cmnt_ob_end = `${space}${String.fromCodePoint(37).repeat(2)}`;
const colon = String.fromCodePoint(0x3a);
const two_colon = colon.repeat(2);
const ul = `${String.fromCodePoint(0x2d)}${space}`;
const tbl_start =`|${space}`;
const tbl_end =`${space}|`;
const tbl_pipe = `${space}|${space}`;
const call_start = `>${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${four_space}-${space}`;
const call_check = `${call_ul}[${space}]${space}`;
const call_check_indent = `${call_ul_indent}[${space}]${space}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;

const head_one = `${hash}${space}`;
const head_two = `${hash.repeat(2)}${space}`;
const head_three = `${hash.repeat(3)}${space}`;
const head_four = `${hash.repeat(4)}${space}`;

//---------------------------------------------------------
// GENERAL VARIABLES
//---------------------------------------------------------
let heading = "";
let question = "";
let data = "";
let call_title = "";
let call_body = "";

//---------------------------------------------------------  
// IDEA COMPASS
//---------------------------------------------------------
const three_head = `${hash.repeat(3)}${space}`;

heading = "North";
question = "Where is the origin?";
data = "Origin";
call_title = `${call_start}[!${heading.toLowerCase()}]${space}${question}${new_line}${call_start}${new_line}`;
call_body = `${call_start}1.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}${call_start}2.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}`;
const north = `${three_head}${heading}${two_new_line}${call_title}${call_body}${two_new_line}`;

heading = "West";
question = "What is similar?";
data = "Similar";
call_title = `${call_start}[!${heading.toLowerCase()}]${space}${question}${new_line}${call_start}${new_line}`;
call_body = `${call_start}1.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}${call_start}2.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}`;
const west = `${three_head}${heading}${two_new_line}${call_title}${call_body}${two_new_line}`;

heading = "East";
question = "What is opposite?";
data = "Opposite";
call_title = `${call_start}[!${heading.toLowerCase()}]${space}${question}${new_line}${call_start}${new_line}`;
call_body = `${call_start}1.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}${call_start}2.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}`;
const east = `${three_head}${heading}${two_new_line}${call_title}${call_body}${two_new_line}`;

heading = "South";
question = "Where does this lead?";
data = "Destination";
call_title = `${call_start}[!${heading.toLowerCase()}]${space}${question}${new_line}${call_start}${new_line}`;
call_body = `${call_start}1.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}${call_start}2.${space}**${data}**::${new_line}${call_ul_indent}**${data}${space}Explanation**::${new_line}`;
const south = `${three_head}${heading}${two_new_line}${call_title}${call_body}${two_new_line}`;

const idea_compass = `${north}${west}${east}${south}`;

tR += idea_compass;
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

1. [[80_note_idea_compass]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[note_title_alias_file_name|Note Title, Alias, and File Name]]
2. [[note_type_subtype_file_class|Note Type, Subtype, and File Class Suggester]]
3. [[note_status|Note Status Suggester]]

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

1. [[tp.file.include Templater Function|The Templater tp.file.include() Function]]

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
