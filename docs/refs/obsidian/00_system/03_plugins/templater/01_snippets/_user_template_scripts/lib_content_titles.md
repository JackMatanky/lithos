---
title: lib_content_titles
aliases:
  - Library Content Titles
  - library content titles
  - Content Titles, Alias, and File Name
  - Titles, Alias, and File Name for Content
  - content_titles_alias_and_file_name
  - titles_alias_and_file_name_for_content
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-26T15:42
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Library Content Titles

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return the full title name, full title value, main title, and subtitle of a library file in order to assign the content's titles, alias, and file name, which has the illegal characters removed.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
async function lib_content_titles(str) {
  const initial_title = `${str}`;

  // EXP: Check the initial title for a colon
  const colon_index = initial_title.indexOf(":");

  let title;
  let subtitle;
  let full_lib_title_name;
  let full_lib_title_value;

  // EXP: If the title includes a colon, split into title and subtitle
  if (colon_index === -1) {
    // EXP: If no colon is found,
    // EXP: assign the full title to the initial title
    title = initial_title;
    full_lib_title_name = title;
    full_lib_title_value = `${full_lib_title_name
      .replaceAll(/[#:\*<>\|\\/-]/g, "_")
      .replaceAll(/\?/g, "")
      .replaceAll(/"/g, "'")}`;
  } else {
    // EXP: If a colon is found,
    // EXP: split the initial title at the colon
    // EXP: into main and secondary titles
    title = initial_title.split(`:`)[0].trim();
    subtitle = initial_title.split(`:`)[1].trim();
    full_lib_title_name = `${title}: ${subtitle}`;
    full_lib_title_value = `${title
      .replaceAll(/[#:\*<>\|\\/-]/g, "_")
      .replaceAll(/\?/g, "")
      .replaceAll(/"/g, "'")}_${subtitle
      .replaceAll(/[#:\*<>\|\\/-]/g, "_")
      .replaceAll(/\?/g, "")
      .replaceAll(/"/g, "'")}`;
  }

  const title_obj = {
    full_title_name: `${full_lib_title_name}`,
    full_title_value: `${full_lib_title_value}`,
    main_title: `${title}`,
    sub_title: `${subtitle}`,
  };

  return title_obj;
}

module.exports = lib_content_titles;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// CONTENT TITLES, ALIAS, AND FILE NAME
//---------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const full_title_name = lib_content_titles.full_title_name;
const full_title_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sub_title;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[71_00_book|Book Template]]
2. [[71_20_book_chapter|Book Chapter Template]]
3. [[72_journal|Journal Article Template]]
4. [[73_report|Report Article Template]]
5. [[75_webpage|Webpage Template]]
6. [[76_10_video_youtube|YouTube Video Template]]
7. [[78_course_OpenU|OpenU Course Template]]
8. [[78_course_OpenU 1]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[rename_untitled_file_prompt|Prompt Rename Untitled File]]

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[lib_content_titles.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[book_title_alias_file_name_directory|Book Title, Alias, File Name, and Directory]]
2. [[book_chapter_title_alias_file_name|Book Chapter Title, Alias, File Name, and Directory]]
3. [[journal_titles_alias_and_file_name|Journal Titles, Alias, and File Name]]
4. [[journal_daily_gratitude_titles_alias_and_file_name|Daily Gratitude Titles, Alias, and File Name]]
5. [[52_00_task_titles_alias_file_name|Task Titles, Alias, and File Name]]

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
