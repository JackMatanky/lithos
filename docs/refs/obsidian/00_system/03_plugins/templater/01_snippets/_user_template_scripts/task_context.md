---
title: task_context
aliases:
  - Task Context Suggester
  - task context suggester
  - task_context_suggester
  - Task Context
  - task_context
plugin: templater
language:
  - javascript
module:
  - user
  - system
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-01T09:36
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Task Context Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return the task's context name, value, and directory with a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
const task_context_obj_arr = [
  {
    key: "Personal",
    value: "personal",
    directory: "41_personal/",
  },
  {
    key: "Habits and Rituals",
    value: "habit_ritual",
    directory: "45_habit_ritual/",
  },
  {
    key: "Education",
    value: "education",
    directory: "42_education/",
  },
  {
    key: "Professional",
    value: "professional",
    directory: "43_professional/",
  },
  {
    key: "Work",
    value: "work",
    directory: "44_work/",
  },
];

async function task_context(tp) {
  const task_context_obj = await tp.system.suggester(
    (item) => item.key,
    task_context_obj_arr,
    false,
    "Task Context?"
  );

  return task_context_obj;
}

module.exports = task_context;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET TASK CONTEXT
//---------------------------------------------------------
const task_context_obj = await tp.user.task_context(tp);
const task_context_name = task_context_obj.key;
const task_context_value = task_context_obj.value;
const task_context_dir = task_context_obj.directory;
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

1. [[task_context_by_path_or_suggester|Task Context by Path or Suggester]]

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[task_context.js]]

### Outgoing Snippet Links

<!-- Link related snippets here  -->

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
