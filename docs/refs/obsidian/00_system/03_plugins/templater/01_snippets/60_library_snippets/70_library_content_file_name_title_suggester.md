---
title: 60_library_content_file_name_title_suggester
aliases:
  - Library Content File Name and Title Suggester
  - Library Content File Name and Title
  - library_content_file_name_title_suggester
  - suggester_library_content_file_name_and_title
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
date_created: 2023-07-17T10:20
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Library Content File Name and Title Suggester

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output:: A basename of the chosen contact file
> Description:: Return a library file's name and main title from `alias[0]`.

---

## Snippet

```javascript
// Template file to include
const lib_name_alias = "60_library_content_file_name_alias";

//---------------------------------------------------------
// SET LIBRARY CONTENT FILE NAME AND ALIAS
//---------------------------------------------------------
// Retrieve the Library Content File Name and Alias template and content
temp_file_path = `${sys_temp_include_dir}${lib_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const lib_resource_value = include_arr[0];
const lib_resource_name = include_arr[1];
const lib_resource_link = include_arr[2];
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET LIBRARY CONTENT FILE NAME AND ALIAS
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${lib_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const lib_resource_value = include_arr[0];
const lib_resource_name = include_arr[1];
const lib_resource_link = include_arr[2];
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// SET LIBRARY CONTENT FILE NAME AND ALIAS
//---------------------------------------------------------
// Library Files Directory
const library_dir = "60_library/";

const lib_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: library_dir,
  file_class: "lib",
  type: "",
});
const lib_obj = await tp.system.suggester(
  (item) => item.key,
  lib_obj_arr,
  false,
  "Library resource?"
);

let lib_resource_value = lib_obj.value;
let lib_resource_name = lib_obj.key;

let lib_resource_link = "";
if (lib_obj.value == "_user_input") {
  lib_resource_name = await tp.system.prompt(
    "Resource title?",
    null,
    false,
    false
  );
  lib_resource_value = await tp.system.prompt(
    "Resource link?",
    null,
    false,
    false
  );
  lib_resource_link = `[${lib_resource_name}](${lib_resource_value})`;
} else {
  lib_resource_link = `[[${lib_resource_value}|${lib_resource_name}]]`;
};

tR += lib_resource_value;
tR += ";";
tR += lib_resource_name;
tR += ";";
tR += lib_resource_link;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[80_00_pkm_tree|General Knowledge Tree Template]]
2. [[80_01_tree_category|Knowledge Tree Category Template]]
3. [[80_02_tree_branch|Knowledge Tree Branch Template]]
4. [[80_03_tree_field|Knowledge Tree Field Template]]
5. [[80_04_tree_subject|Knowledge Tree Subject Template]]
6. [[80_05_tree_topic|Knowledge Tree Topic Template]]
7. [[80_06_tree_subtopic|Knowledge Tree Subtopic Template]]
8. [[90_00_note|General Note Template]]
9. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
10. [[90_11_note_quote|Quote Fleeting Note Template]]
11. [[90_12_note_idea|Idea Fleeting Note Template]]
12. [[90_20_note_literature(X)|General Literature Note Template]]
13. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
14. [[90_31_note_question|QEC Question Note Template]]
15. [[90_32_note_evidence|QEC Evidence Note Template]]
16. [[90_33_note_conclusion|QEC Conclusion Note Template]]
17. [[90_40_note_lit_psa(X)|PSA Note Template]]
18. [[90_41_note_problem|PSA Problem Note Template]]
19. [[90_42_note_steps|PSA Steps Note Template]]
20. [[90_43_note_answer|PSA Answer Note Template]]
21. [[90_50_note_info(X)|General Info Note Template]]
22. [[90_51_note_concept|Concept Note Template]]
23. [[90_52_note_definition|Definition Note Template]]
24. [[53_30_act_education]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[60_library_content_file_name_alias]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[get_md_file_names|Markdown File Names for Suggester]]
2. [[61_contact_file_name_title_suggester|Contact File Name and Title Suggester]]
3. [[62_organization_file_name_title_suggester|Organization File Name and Title Suggester]]
4. [[10_pillar_file_name_title_suggester|Pillar File Name and Title Suggester]]

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
