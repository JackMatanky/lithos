---
title: nl_date_and_time
aliases:
  - Natural Language Date and Time
  - NL Date and Time
  - nl date and time
plugin: templater
language:
  - javascript
module:
  - user
  - momentjs
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-31T15:51
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs
---
# Natural Language Date and Time

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a full datetime from a date parsed from natural language into [[iso_8601|ISO 8601]] YYYY-MM-DD format and the time parsed from [[Natural Language Dates]].

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET DATE AND TIME
//---------------------------------------------------------
// TYPE OPTIONS: "start", "end", ""
// Choose the date and type
const date = await tp.user.nl_date(tp, "");

// Choose the time for the action item
const time = await tp.user.nl_time(tp, "");

// Parse full date with Natural Language Dates
const full_date_time = moment(`${date}T${time}`);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET DATE AND TIME
//---------------------------------------------------------
const date = await tp.user.nl_date(tp, "");
const time = await tp.user.nl_time(tp, "");
const full_date_time = moment(`${date}T${time}`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[52_00_task_event]]
2. [[53_00_action_item]]
3. [[54_00_meeting]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[nl_date|Natural Language Date Suggester]]
2. [[nl_time|Natural Language Time Suggester]]

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
	file.frontmatter.definition AS Definition
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
