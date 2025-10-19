---
title: 55_10_habit_tag_context_type_file_class
aliases:
  - Habit Task Tag, Context, Type, and File Class
  - habit task tag, context, type, and file class
  - habit task tag context type and file class
  - habit tag context type file class
plugin: templater
language:
  - javascript
module:
  -
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-20T16:58
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Habit Task Tag, Context, Type, and File Class

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a habit's task tag, context, type, and file class

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// HABIT TASK TAG, CONTEXT, TYPE, AND FILE CLASS
//---------------------------------------------------------
const task_tag = `#task`;
const context_name = `Habits and Rituals`;
// Replace the middle "s," space, "and," and space from context_name
// Then replace the final "s" from "Rituals"
const context = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();
const context_dir = habit_ritual_proj_dir;
const type_name = context_name.split(" ")[0].replaceAll(/s$/g, "");
const type_value = type_name.toLowerCase();
const file_class = "task_child";
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// HABIT TASK TAG, CONTEXT, TYPE, AND FILE CLASS
//---------------------------------------------------------
const task_tag = `#task`;
const context_name = `Habits and Rituals`;
const context_value = context_name
  .replaceAll(/s\sand\s/g, "_")
  .replaceAll(/s$/g, "")
  .toLowerCase();
const context_dir = habit_ritual_proj_dir;
const type_name = context_name.split(" ")[0].replaceAll(/s$/g, "");
const type_value = type_name.toLowerCase();
const file_class = "task_child";
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

1. [[55_10_habit]]

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
