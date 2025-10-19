---
title: get_file
aliases:
  - Get Vault Files
  - Get Vault Files for Templater Suggester
  - vault_file_suggester
plugin: templater
language:
  - javascript
module:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-23T16:46
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/api
---
# Vault Files

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input:: Directory Path
> Output:: Array of File Names with Extensions
> Description:: Return a vault file name with extension from a suggester filtered by a directory.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
async function get_file(directory) {
  const obsidian_files = app.vault.getFiles();

  const full_file_paths = obsidian_files.filter((f) =>
    f.path.includes(directory)
  );

  const file_names = full_file_paths.map((f) => f.name).sort();

  const null_arr = ["null", "_user_input"];

  null_arr.push(file_names);

  const files = null_arr.flat();

  return files;
}

module.exports = get_file;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// FILE NAME AND EXTENSION BY DIRECTORY
//---------------------------------------------------------
const full_file_name = await tp.user.vault_file(directory);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[61_contact|Contact Template]]
2. [[62_organization|Organization Template]]
3. [[71_00_book|Book Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[vault_file.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[book_cover_suggester|Book Cover Suggester]]

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
