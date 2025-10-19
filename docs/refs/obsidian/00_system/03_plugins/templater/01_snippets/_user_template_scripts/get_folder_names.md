---
title: get_folder_names
aliases:
  - Folder Names
  - Folder Name Suggester
  - Folder Names for Templater Suggester
  - folder_names_suggester
plugin: templater
language:
  - javascript
module:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-01T09:36
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/api
---
# Folder Names

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input:: Directory Path, Split Path Index
> Output:: Array
> Description:: Return a folder name from a suggester filtered by directory and the index of the path split by `/`.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
async function get_folder_names({ dir: directory, index: path_index }) {
  const obsidian_folders = app.vault.getAllLoadedFiles();

  const all_directory_paths = obsidian_folders
    .filter((i) => i.children)
    .map((folder) => folder.path);

  const folder_paths = all_directory_paths.filter((folder_path) =>
    folder_path.includes(directory)
  );

  const folder_names = folder_paths
    .map((folder_path) => folder_path.split("/")[path_index])
    .filter((folder_name) => folder_name);

  const folders_set = […new Set(folder_names)];

  folders_set.sort();

  const folders_arr = ["null"];

  folders_arr.push(folders_set);

  const folders = folders_arr.flat();

  return folders;
}

module.exports = get_folder_names;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// FOLDER NAME BY DIRECTORY AND SPLIT PATH INDEX
//---------------------------------------------------------
const folder = await tp.user.folder_name({
  dir: directory,
  index: path_index,
});
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

1. [[folder_name.js]]

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
