---
title: 40_goal_file_suggester
aliases:
  - Goal File Suggester
  - Goal File
  - goal_file_suggester
language:
  - javascript
plugin: templater
module:
  - system
  - user
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T15:57
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Goal File Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output:: A basename of the chosen goal file
> Description:

---

## Snippet

```javascript
// Goal Files Directory
const goals_dir = `30_goals/`;

//---------------------------------------------------------
// SET GOAL
//---------------------------------------------------------
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal?`
);
```

### Templater

```javascript
//---------------------------------------------------------
// SET GOAL
//---------------------------------------------------------
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(goals, goals, false, `Goal?`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

---

## Related

### Outgoing Snippet Links

1. [[get_md_file_names|Markdown File Names for Suggester]]

### Incoming Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	file.link AS Snippet,
	file.frontmatter.description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND contains(file.outlinks, this.file.link)
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### Incoming Function Links

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
