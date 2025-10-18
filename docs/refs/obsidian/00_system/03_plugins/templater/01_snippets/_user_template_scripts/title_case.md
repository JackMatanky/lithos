---
title: title_case
aliases:
  - Title Case
  - title case
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-14T08:11
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Title Case

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a title cased string.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
const lower_arr = [
  "a",
  "an",
  "and",
  "as",
  "at",
  "but",
  "by",
  "for",
  "for",
  "from",
  "in",
  "into",
  "near",
  "no",
  "nor",
  "of",
  "on",
  "onto",
  "or",
  "the",
  "to",
  "with",
];

const upper_arr = ["AI", "ID", "TV", "YAML"];

async function title_case(str) {
  // EXP: Split the initial string by spaces.
  const initial_title = str
    .split(" ")
    .map((w) =>
      // EXP: If a word is in lower_arr, change it to lowercase
      // EXP: If a word is in upper_arr, change it to uppercase
      // EXP: Otherwise, title case the word
      lower_arr.includes(w.replaceAll(/^,|[,:]$/g, "").toLowerCase())
        ? w.toLowerCase()
        : upper_arr.includes(w.replaceAll(/^,|[,:]$/g, "").toUpperCase())
        ? w.toUpperCase()
        : w[0].toUpperCase() + w.substring(1).toLowerCase()
    )
    .join(" ");

  // EXP: Check the initial title for a colon
  const colon_index = initial_title.indexOf(":");

  let full_title;
  let title;
  let subtitle;
  if (colon_index === -1) {
    // EXP: If no colon is found,
    // EXP: assign the full title to the split initial title
    full_title = initial_title.split(" ");

    // EXP: Ensure first word is capitalized
    full_title[0] = lower_arr.includes(full_title[0])
      ? full_title[0].charAt(0).toUpperCase() + full_title[0].substring(1)
      : full_title[0];

    // EXP: Reassemble the full title as a string
    full_title = full_title.join(" ");
  } else {
    // EXP: If a colon is found,
    // EXP: split the initial title at the colon
    // EXP: into main and secondary titles
    // EXP: and follow the same procedure as above
    title = initial_title.split(":")[0].trim();
    title_arr = title.split(" ");
    title_arr[0] = lower_arr.includes(title_arr[0])
      ? title_arr[0].charAt(0).toUpperCase() + title_arr[0].substring(1)
      : title_arr[0];
    title = title_arr.join(" ");

    subtitle = initial_title.split(":")[1].trim();
    subtitle_arr = subtitle.split(" ");
    subtitle_arr[0] = lower_arr.includes(subtitle_arr[0])
      ? subtitle_arr[0].charAt(0).toUpperCase() + subtitle_arr[0].substring(1)
      : subtitle_arr[0];
    subtitle = subtitle_arr.join(" ");

    // EXP: Assign the full title to the rejoined title and subtitle
    full_title = `${title}: ${subtitle}`;
  }
  return full_title;
}

module.exports = title_case;
```

### Old Snippet

```javascript
lower_arr = [
  "a",
  "an",
  "and",
  "as",
  "at",
  "but",
  "by",
  "for",
  "for",
  "from",
  "in",
  "into",
  "near",
  "no",
  "nor",
  "of",
  "on",
  "onto",
  "or",
  "the",
  "to",
  "with",
];

upper_arr = ["AI", "ID", "TV"];

async function title_case(str) {
  // EXP: Split the initial string by spaces
  // EXP: title case any word not found in lower_arr
  const initial_title_arr = str
    .split(" ")
    .map((w) =>
      lower_arr.includes(w.replaceAll(/^,|,$/g, "").toLowerCase())
        ? w.toLowerCase()
        : upper_arr.includes(w.replaceAll(/^,|,$/g, "").toUpperCase())
        ? w.toUpperCase()
        : w[0].toUpperCase() + w.substring(1).toLowerCase()
    );

  const index = initial_title_arr.findIndex((i) => i.includes(":"));

  let full_title;
  let title;
  let subtitle;

  if (index == -1) {
    // EXP: If no colon is found, return the full title
    full_title = initial_title_arr.join(" ");
  } else {
    // EXP: If a colon is found, assign the main title
    // EXP: to the initial title spliced to the index plus one
    title_arr = initial_title_arr.splice(0, index + 1);

    // EXP: If present, remove the colon in title array's last element
    title_arr[index] = title_arr[index].includes(":")
      ? title_arr[index].substring(0, -1)
      : title_arr[index];

    // EXP: If lower_arr includes the first title
    // EXP: array element, convert it to title case
    title_arr[0] = lower_arr.includes(title_arr[0])
      ? title_arr[0].charAt(0).toUpperCase() + title_arr[0].substring(1)
      : title_arr[0];

    title = title_arr.join(" ").trim();

    // EXP: assign the subtitle to the spliced initial
    // EXP: title from the index plus on onward
    subtitle_arr = initial_title_arr;

    // EXP: If lower_arr includes the first subtitle
    // EXP: array element, convert it to title case
    subtitle_arr[0] = lower_arr.includes(subtitle_arr[0])
      ? subtitle_arr[0].charAt(0).toUpperCase() + subtitle_arr[0].substring(1)
      : subtitle_arr[0];

    subtitle = subtitle_arr.join(" ").trim();

    full_title = `${title}: ${subtitle}`;
  }
  return full_title;
}

module.exports = title_case;
```

## Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// CONVERT TITLE TO TITLE CASE
//---------------------------------------------------------
const title_case = await tp.user.title_case(str)
```

## Language Reference

<!-- Recreate the code with links to files  -->

## Explanation

```javascript

```

## Use Cases

### Files

<!-- Files containing the snippet  -->

### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[rename_untitled_file_prompt|Rename Untitled File Prompt]]

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[title_case.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dir_contact_names|Contact Names Prompt]]

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
