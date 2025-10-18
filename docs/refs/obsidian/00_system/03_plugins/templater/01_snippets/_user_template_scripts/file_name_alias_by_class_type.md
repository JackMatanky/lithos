---
title: file_name_alias_by_class_type
aliases:
  - Markdown File Names and Alias by Directory, File Class, and Type Suggester
  - Markdown File Names and Alias by Directory, File Class, and Type
  - markdown file names and alias by directory file class and type suggester
  - markdown file names and alias by directory file class and type
  - file name and alias by dir class type
  - file name alias by class type
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-18T11:29
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Markdown File Names Filtered by Directory, File Class, and Type Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input:: Directory Path, File Class YAML Value, Type YAML Value  
> Output:: Array  
> Description:: Return a Markdown file name and alias from a suggester filtered by directory, file class and type in the YAML metadata.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
async function file_name_alias_by_class_type({
  dir: directory,
  file_class: yaml_class,
  type: yaml_type,
}) {
  const class_arg = `${yaml_class}`;
  const type_arg = `${yaml_type}`;

  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((file) =>
    file.path.includes(directory)
  );

  const mapped_file_promises = file_paths.map(async (file) => {
    const file_cache = await app.metadataCache.getFileCache(file);

    const class_frontmatter = file_cache?.frontmatter?.file_class;
    const type_frontmatter = file_cache?.frontmatter?.type;

    // If the file class and type frontmatter values
    // equal type_arg and start with class_arg
    // , mark it to be included
    if (type_arg == "") {
      file.shouldInclude =
        class_frontmatter && class_frontmatter.startsWith(class_arg);
    } else {
      file.shouldInclude =
        type_frontmatter == type_arg && class_frontmatter.startsWith(class_arg);
    }
    return file;
  });

  // Wait for all files to be processed
  // because getting frontmatter is asynchronous
  const mapped_files = await Promise.all(mapped_file_promises);

  // Filter out files that shouldn't be included
  const filtered_files = mapped_files.filter((file) => file.shouldInclude);

  // Create an array for the filtered files
  const filtered_files_arr = [];

  // Append the filtered files to the array
  filtered_files_arr.push(filtered_files);

  // Flatten the array from two dimensions to one
  const file_arr = filtered_files_arr.flat();

  // const file_obj_arr = file_arr;

  let file_name;
  let file_alias;
  let file_obj_arr = [];

  for (let i = 0; i < file_arr.length; i++) {
    // Retrieve the file alias
    const file_cache = await app.metadataCache.getFileCache(file_arr[i]);
    file_alias = file_cache.frontmatter.aliases[0];
    // Retrieve the file name
    file_name = file_arr[i].basename;
    // Push the key-value object into the file object array
    file_obj_arr.push({ key: file_alias, value: file_name });
  }

  // Sort the array by file name
  file_obj_arr.sort((a, b) => {
    let value_a = a.value.toLowerCase(),
      value_b = b.value.toLowerCase();

    if (value_a < value_b) {
      return -1;
    }
    if (value_a > value_b) {
      return 1;
    }
    return 0;
  });

  // Add an object for null values
  const obj_arr = [{ key: "Null", value: "null" }];

  // Append the file array object to the null array object
  obj_arr.push(file_obj_arr);

  // Reassign the flattened null array object to the file array object
  file_obj_arr = obj_arr.flat();

  return file_obj_arr;
}

module.exports = file_name_alias_by_class_type;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// MD FILE NAME AND ALIAS BY DIR, FILE CLASS, AND TYPE
//---------------------------------------------------------
const file_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: directory,
  file_class: yaml_class,
  type: yaml_type,
})

const file_obj = await tp.system.suggester(
  (item) => item.key,
  file_obj_arr,
  false,
  'File?'
)

const file_name = file_obj.value;
const file_alias = file_obj.key;
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

1. [[related_project_suggester|Related Project Suggester]]
2. [[task_context_project_by_path_or_suggester|Project by Path or Suggester]]
3. [[related_parent_task_suggester|Related Parent Task Suggester]]
4. [[task_parent_task_by_path_or_suggester|Parent Task by Path or Suggester]]

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[file_name_alias_by_class_type.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[get_file_by_status|Markdown File Names Filtered by Frontmatter Status]]
2. [[get_md_file_titles_names|Markdown File Names and Titles for Suggester]]

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

See [origin](https://discord.com/channels/686053708261228577/875720842443649045/1088272864500789308)

---

## Flashcards
