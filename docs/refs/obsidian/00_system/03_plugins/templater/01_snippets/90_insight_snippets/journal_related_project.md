---
title: journal_related_project
aliases:
  - Journal Related Project Suggester
  - Journal Related Project
  - journal_related_project_suggester
  - suggester_journal_related_project
plugin: templater
language:
  - javascript
module:
  - system
  - user
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T14:20
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Journal Related Project Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Filter project folder paths by the project directory and set related project.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Projects directory
const projects_dir = `40_projects/`;

//---------------------------------------------------------
// SET RELATED PROJECT
//---------------------------------------------------------
// Filter array to only include project folder paths based on task context
const projects = await tp.user.folder_name({
  dir: projects_dir,
  index: 2,
});

// Choose a project if the journal is related,
// otherwise choose "null"
const project = await tp.system.suggester(
  projects,
  projects,
  false,
  `Related Project to the ${full_type_name}?`
);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET RELATED PROJECT
//---------------------------------------------------------
const projects = await tp.user.folder_name({
  dir: projects_dir,
  index: 2,
});

const project = await tp.system.suggester(
  projects,
  projects,
  false,
  `Related Project to the ${full_type_name}?`
);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

1. [[25_10_daily_reflection]]

#### In Conjunction

---

## Related

### Outgoing Snippet Links

1. [[related_project_suggester|Project Folder Suggester]]

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	description AS Description,
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
	file.frontmatter.definition AS Definition
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND contains(file.outlinks, this.file.link)
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
