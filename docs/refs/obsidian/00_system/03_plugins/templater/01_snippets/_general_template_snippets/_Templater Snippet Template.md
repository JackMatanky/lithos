---
title: 
aliases:
  - 
plugin: templater
language:
  - javascript
module:
  - 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Title

## Description

> [!snippet] Snippet Details
>  
> Plugin:: [[Templater]]  
> Language:: [[JavaScript]]  
> Input::  
> Output::  
> Description::

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript

```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript

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
