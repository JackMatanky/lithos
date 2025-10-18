---
title: get_file_by_status
aliases:
  - MD File Names Filtered by Frontmatter Status
  - Markdown File Names Filtered by Frontmatter Status
  - MD File Names Filtered by Frontmatter Status for Templater Suggester
  - Markdown File Names Filtered by Frontmatter Status for Templater Suggester
  - filtered_md_file_names_suggester
plugin: templater
language:
  - javascript
module: 
description: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-18T11:29
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Markdown File Names Filtered by Frontmatter Status

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input:: Directory Path, YAML key  
> Output:: Array  
> Description:: Return a markdown file name from a suggester filtered by directory and file status in the YAML metadata.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
async function get_file_by_status({ dir: directory, status: yaml_value }) {
  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((file) =>
    file.path.includes(directory)
  );

  const mapped_file_promises = file_paths.map(async (file) => {
    const file_cache = await app.metadataCache.getFileCache(file);
    
    // If the status frontmatter value
    // equals yaml_value, mark it to be included
    file.shouldInclude = file_cache?.frontmatter.status === yaml_value;

    return file;
  });

  // Wait for all files to be processed (have to wait because getting frontmatter is asynchronous)
  const mapped_files = await Promise.all(mapped_file_promises);

  // Filter out files that shouldn't be included
  const filtered_files = mapped_files.filter((file) => file.shouldInclude);

  // Convert list of files into list of links
  const file_names = filtered_files.map((file) => file.basename).sort();

  // Create an array for the filtered files 
  const files_arr = ["null"];
  
  // Append the filtered files to the array
  files_arr.push(file_names);

  // Flatten the array from two dimensions to one
  const files = files_arr.flat();

  return files;
}

module.exports = get_file_by_status;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// MD FILE BY DIRECTORY AND STATUS
//---------------------------------------------------------
const file = await tp.user.file_by_status({
  dir: directory,
  status: yaml_value,
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

1. [[20_00_pillar_name_alias]]

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[file_by_status.js]]

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

See [origin](https://discord.com/channels/686053708261228577/875720842443649045/1088272864500789308)

---

## Flashcards
