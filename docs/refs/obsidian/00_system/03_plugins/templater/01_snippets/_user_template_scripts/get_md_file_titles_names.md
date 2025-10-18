---
title: get_md_file_titles_names
aliases:
  - Markdown File Names and Titles for Suggester
  - MD File Names and Titles
  - Markdown File Names
  - MD File Names for Templater Suggester
  - Markdown File Names for Templater Suggester
  - md_file_names_suggester
plugin: templater
language:
  - javascript
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-14T08:11
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Markdown File Names and Titles for Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a Markdown file's name and alias from a suggester.

---

## Snippet

```javascript
async function get_md_file_titles_names(directory) {
  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((f) =>
    f.path.includes(directory)
  );

  let file_name;
  let file_alias;
  let file_obj_arr = [];

  for (let i = 0; i < file_paths.length; i++) {
    // Retrieve the file alias
    const file_cache = await app.metadataCache.getFileCache(file_paths[i]);
    file_alias = file_cache.frontmatter.aliases[0];
    // Retrieve the file name
    file_name = file_paths[i].basename;
    // Push the key-value object into the file object array
    file_obj_arr.push({ key: file_alias, value: file_name });
  }

  // Sort the array by key
  file_obj_arr.sort((a, b) => {
    let key_a = a.key.toLowerCase(),
      key_b = b.key.toLowerCase();

    if (key_a < key_b) {
      return -1;
    }
    if (key_a > key_b) {
      return 1;
    }
    return 0;
  });

  // Add an object for null and user input values
  const obj_arr = [
    { key: "Null", value: "null" },
    { key: "User Input", value: "_user_input" },
  ];
  // Append the file array object to the null array object
  obj_arr.push(file_obj_arr);
  // Reassign the flattened null array object to the file array object
  file_obj_arr = obj_arr.flat();

  return file_obj_arr;
}

module.exports = get_md_file_titles_names;
```

### Templater

```javascript
//---------------------------------------------------------  
// MARKDOWN FILE NAME AND ALIAS BY DIRECTORY
//---------------------------------------------------------
const file_obj_arr = await tp.user.md_file_name_alias(directory);
const file_obj = await tp.system.suggester(
  (item) => item.key,
  file_obj_arr,
  false,
  'File?'
)
const file_alias = file_obj.key;
const file_name = file_obj.value;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[62_organization_file_name_title_suggester|Organization File Name and Title Suggester]]
2. [[61_contact_file_name_title_suggester|Contact File Name and Title Suggester]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->  

1. [[md_file_name_alias.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[file_name_alias_by_class_type|Markdown File Names Filtered by Directory, File Class, and Type]]
2. [[get_file_by_status|Markdown File Names Filtered by Frontmatter Status]]

### Incoming Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
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
