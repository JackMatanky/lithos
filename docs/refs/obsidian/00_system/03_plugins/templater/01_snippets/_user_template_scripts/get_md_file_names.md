---
title: get_md_file_names
aliases:
  - Markdown File Names for Suggester
  - MD File Names
  - Markdown File Names
  - MD File Names for Templater Suggester
  - Markdown File Names for Templater Suggester
  - md_file_names_suggester
plugin: templater
language:
  - javascript
module:
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-01T09:36
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Markdown File Names

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a list of markdown files' basename, primarily used for a suggester.

---

## Snippet

```javascript
async function get_md_file_names(directory) {
  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((f) =>
    f.path.includes(directory)
  );

  const file_names = file_paths.map((f) => f.basename).sort();

  const files_arr = ["null"];

  files_arr.push(file_names);

  const files = files_arr.flat();

  return files;
};

module.exports = get_md_file_names;

```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// MARKDOWN FILE NAME BY DIRECTORY
//---------------------------------------------------------
const file_name = await tp.user.md_file_name(directory);
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

### Script Link

<!-- Link the user template script here -->

1. [[md_file_name.js]]

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
