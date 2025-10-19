---
title: template_file_include
aliases:
  - Include a Template File in a Template
  - include template file in template
  - include template file
  - template include file
plugin: templater
language:
  - javascript
module:
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-15T11:12
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Include a Template File in a Template

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input:: Template tFile
> Output::
> Description:: Return a single value from a template

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// >>>TODO: DEFINE <FILE_NAME> VARIABLE<<<
// >>>TODO: DEFINE <TEMPLATE_CONTENT_VAR> VARIABLE<<<
//---------------------------------------------------------
// RETRIEVE EXTERNAL TEMPLATE CONTENT
//---------------------------------------------------------
let temp_file_name = `FILE_NAME`;
let temp_tfile = await tp.file.find_tfile(temp_file_name);
let temp_content = await tp.file.include(temp_tfile);
const TEMPLATE_CONTENT_VAR = temp_content;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// RETRIEVE EXTERNAL TEMPLATE CONTENT
//---------------------------------------------------------
let temp_file_name = `FILE_NAME`;
let temp_tfile = await tp.file.find_tfile(temp_file_name);
let temp_content = await tp.file.include(temp_tfile);
const TEMPLATE_CONTENT_VAR = temp_content;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[10_pillar_file_name_title_suggester|Pillar File and Full Name Suggester]]
2. [[62_organization_file_name_title_suggester|Organization File Name and Title Suggester]]
3. [[61_contact_file_name_title_suggester|Contact File Name and Title Suggester]]

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
