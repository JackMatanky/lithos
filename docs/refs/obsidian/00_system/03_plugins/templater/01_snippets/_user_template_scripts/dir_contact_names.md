---
title: dir_contact_names
aliases:
  - Contact Names Prompt
  - contact names prompt
  - prompt_contact_name
  - dir contact names
plugin: templater
language:
  - javascript
module:
  - user
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-14T08:11
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Contact Names Prompt

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a contact's full, first, last, and maiden names from a prompt.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
const surname_prefix_arr = [
  `da`,
  `das`,
  `de`,
  `del`,
  `dele`,
  `della`,
  `den`,
  `der`,
  `des`,
  `di`,
  `dos`,
  `du`,
  `het`,
  `la`,
  `le`,
  `van`,
  `von`,
];

async function dir_contact_names(tp) {
  const name = await tp.system.prompt(`Full Name?`);

  const names = name.split(" ");
  const name_first = names[0];
  let name_last_maiden = `null`;
  let name_last_prefix = `null`;
  let name_last = `null`;
  let name_last_first = `null`;

  if (names.length < 3) {
    name_last = names[1];
    if (name_last.split(`-`).length == 2) {
      name_last_maiden = name_last.split(`-`)[0];
    }
    name_last_first = `${name_last}, ${name_first}`;
  } else if (names.length < 4) {
    if (surname_prefix_arr.includes(names[1].toLowerCase())) {
      name_last_prefix = names[1];
      name_last = names[2];
      name_last_first = `${name_last}, ${name_first} ${name_last_prefix}`;
    } else {
      name_last = names[2];
      name_last_maiden = names[1];
      name_last_first = `${name_last_maiden} ${name_last}, ${name_first}`;
    }
  } else if (names.length < 5) {
    if (
      surname_prefix_arr.includes(names[1].toLowerCase()) &&
      surname_prefix_arr.includes(names[2].toLowerCase())
    ) {
      name_last_prefix = `${names[1]} ${names[2]}`;
      name_last = names[3];
      name_last_first = `${name_last}, ${name_first} ${name_last_prefix}`;
    } else if (surname_prefix_arr.includes(names[1].toLowerCase())) {
      name_last_prefix = names[1];
      name_last = names[3];
      name_last_maiden = names[2];
      name_last_first = `${name_last_maiden} ${name_last}, ${name_first} ${name_last_prefix}`;
    } else if (surname_prefix_arr.includes(names[2].toLowerCase())) {
      name_last_prefix = names[2];
      name_last = names[3];
      name_last_maiden = names[1];
      name_last_first = `${name_last}, ${name_first} ${name_last_prefix}`;
    }
  } else if (names.length < 6) {
    if (
      surname_prefix_arr.includes(names[1].toLowerCase()) &&
      surname_prefix_arr.includes(names[2].toLowerCase())
    ) {
      name_last_prefix = `${names[1]} ${names[2]}`;
      name_last = names[4];
      name_last_maiden = names[3];
      name_last_first = `${name_last}, ${name_first} ${name_last_prefix} ${name_last_maiden}`;
    } else if (
      surname_prefix_arr.includes(names[2].toLowerCase()) &&
      surname_prefix_arr.includes(names[3].toLowerCase())
    ) {
      name_last_prefix = `${names[2]} ${names[3]}`;
      name_last = names[4];
      name_last_maiden = names[1];
      name_last_first = `${name_last}, ${name_first} ${name_last_maiden} ${name_last_prefix}`;
    }
  }

  const name_obj = {
    full_name: `${name}`,
    first_name: `${name_first}`,
    last_name: `${name_last}`,
    maiden_name: `${name_last_maiden}`,
    surname_prefix: `${name_last_prefix}`,
    last_first_name: `${name_last_first}`,
  };

  return name_obj;
}

module.exports = dir_contact_names;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET CONTACT NAMES
//---------------------------------------------------------
const contact_names = await tp.user.dirContactNames(tp);
const full_name = contact_names.full_name;
const first_name = contact_names.first_name;
const last_name = contact_names.last_name;
const maiden_name = contact_names.maiden_name;
const surname_prefix = contact_names.surname_prefix;
const last_first_name = contact_names.last_first_name;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[61_contact]]
2. [[71_00_book]]
3. [[72_journal]]
4. [[73_report]]
5. [[75_webpage]]
6. [[76_10_video_youtube]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[contact_names.js]]

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
