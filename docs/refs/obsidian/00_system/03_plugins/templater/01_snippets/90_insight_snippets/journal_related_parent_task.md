---
title: journal_related_parent_task
aliases:
  - Journal Related Parent Task Suggester
  - Journal Related Parent Task
  - journal_related_parent_task_suggester
  - suggester_journal_related_parent_task
plugin: templater
language:
  - javascript
module:
  - system
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T14:20
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Journal Related Parent Task Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Filter parent task folder paths by related project and set related parent task.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// TODO: Define <project>
//---------------------------------------------------------
// SET RELATED PARENT TASK
//---------------------------------------------------------
let parent_task;
if (project !== "null") {
  // Filter array to only include parent task folder paths matching the chosen project
  const parent_tasks = await tp.user.folder_name({
    dir: project,
    index: 3,
  });

  // Choose a parent task if the journal is related,
  // otherwise choose "null"
  parent_task = await tp.system.suggester(
    parent_tasks,
    parent_tasks,
    false,
    `Related Parent Task to the ${full_type_name}?`
  );
} else {
  parent_task = `null`;
};
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET RELATED PARENT TASK
//---------------------------------------------------------
let parent_task;
if (project !== `null`) {
  const parent_tasks = await tp.user.folder_name({
    dir: project,
    index: 3,
  });

  parent_task = await tp.system.suggester(
    parent_tasks,
    parent_tasks,
    false,
    `Related Parent Task to the ${full_type_name}?`
  );
} else {
  parent_task = `null`;
};
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[25_10_daily_reflection]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[journal_related_project|Journal Related Project Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

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
