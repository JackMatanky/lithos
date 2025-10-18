---
title: related_parent_task_suggester
aliases:
  - Related Parent Task Suggester
  - Related Parent Task
  - suggester_related_parent_task
  - related parent task suggester
plugin: templater
language:
  - javascript
module:
  - system
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T14:16
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Related Parent Task Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a related parent task's file name, alias, and link.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET RELATED PARENT TASK
//---------------------------------------------------------
let parent_task_value = "null";
let parent_task_name = "null";
let parent_task_link = "Null|null";

if (project_value !== "null") {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type(
    {
      dir: project_value,
      file_class: "task",
      type: "parent_task",
    }
  );
  const parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    "Is this journal entry related to the project's parent tasks?"
  );
}
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET RELATED PARENT TASK
//---------------------------------------------------------
let parent_task_value = "null";
let parent_task_name = "null";
let parent_task_link = "Null|null";

if (project_value !== "null") {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type(
    {
      dir: project_value,
      file_class: "task",
      type: "parent_task",
    }
  );
  const parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    "Is this journal entry related to the project's parent tasks?"
  );
}
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[28_00_journal_prompt|Prompt Journal Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[related_project_suggester|Related Project Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[task_parent_task_by_path_or_suggester|Parent Task by Path or Suggester]]
2. [[task_context_by_path_or_suggester|Task Context by Path or Suggester]]
3. [[task_context_project_by_path_or_suggester|Project by Path or Suggester]]
4. [[journal_related_project|Journal Related Project Suggester]]
5. [[journal_related_parent_task|Journal Related Parent Task Suggester]]

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
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
