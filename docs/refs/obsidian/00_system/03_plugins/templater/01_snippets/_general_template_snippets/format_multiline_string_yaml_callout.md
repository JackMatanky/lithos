---
title: format_multiline_string_yaml_callout
aliases:
  - Format Multiline String for YAML Frontmatter and Callouts From Prompt
  - format multiline string for yaml frontmatter and callouts from prompt
  - format multiline string yaml callout
plugin: templater
language:
  - javascript
module:
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-28T15:29
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Format Multiline String for YAML Frontmatter and Callouts From Prompt

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return multiline strings for frontmatter and a callout from user input.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET MULTILINE YAML STRING VALUE AND CALLOUT
//---------------------------------------------------------
const multiline_string_input = await tp.system.prompt(
  "Multiline string?",
  null,
  false,
  true
);

const multiline_string_value = multiline_string_input
  .replaceAll(/(\n\s)+(\w)/g, "\n $2")
  .replaceAll(/(\n)+(\w)/g, "\n $2");

const multiline_string_callout = multiline_string_input
  .replaceAll(/(\n\s)+(\w)/g, "  \n> $2")
  .replaceAll(/(\n)+(\w)/g, "  \n> $2");
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET MULTILINE YAML STRING VALUE AND CALLOUT
//---------------------------------------------------------
const multiline_string_input = await tp.system.prompt(
  "Multiline string?",
  null,
  false,
  true
);
const multiline_string_value = multiline_string_input
  .replaceAll(/(\n\s)+(\w)/g, "\n $2")
  .replaceAll(/(\n)+(\w)/g, "\n $2");
const multiline_string_callout = multiline_string_input
  .replaceAll(/(\n\s)+(\w)/g, "  \n> $2")
  .replaceAll(/(\n)+(\w)/g, "  \n> $2");
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

1. [[62_organization|Organization Template]]
2. [[71_00_book|Book Template]]

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

1. [[tp.system.prompt Templater Function|The Templater tp.system.prompt() Function]]

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

[[problem_format_multiline_yaml_header_with_regular_expression|Format Multiline YAML Header with Regular Expression]]  
[[step_format_multiline_yaml_header_with_regular_expression|Steps to Format Multiline YAML Header with Regular Expression]]

---

## Flashcards
