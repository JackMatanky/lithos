---
title: rename_untitled_file_prompt
aliases:
  - Rename Untitled File Prompt
  - Untitled File Prompt Rename
  - Prompt Rename Untitled File
  - prompt_rename_untitled_file
plugin: templater
language:
  - javascript
module:
  - system
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-01T10:01
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Rename Untitled File Prompt

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Rename an untitled file based on a prompt.

---

## Snippet

```javascript
//---------------------------------------------------------
// SET FILE'S TITLE
//---------------------------------------------------------
// Check if note already has title
const has_title = !tp.file.title.startsWith(`Untitled`);
let title;

// If note does not have title, prompt for title
if (!has_title) {
  title = await tp.system.prompt(
    `Title`,
    null,
    true,
    false
  );
} else {
  title = tp.file.title;
};

// Trim title input
title = title.trim();

// Title Case the title
title = await tp.user.title_case(title);
```

### Templater

```javascript
//---------------------------------------------------------
// SET FILE'S TITLE
//---------------------------------------------------------
const has_title = !tp.file.title.startsWith(`Untitled`);
let title;
if (!has_title) {
  title = await tp.system.prompt(`Title`, null, true, false);
} else {
  title = tp.file.title;
};
title = title.trim();
title = await tp.user.title_case(title);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[28_00_journal_prompt]]
2. [[61_contact|Contact Template]]
3. [[62_organization|Organization Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[title_case|Title Case]]

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

---

## Flashcards
