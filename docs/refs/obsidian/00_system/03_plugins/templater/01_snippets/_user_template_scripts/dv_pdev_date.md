---
title: dv_pdev_date
aliases:
  - Journals by Date Dataview List
  - Dataview List of Journals by Date
  - journal_date_dv_list
  - dv_journal
plugin:
  - templater
  - dataview
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-11T13:37
date_modified: 2023-10-25T16:23
tags: obsidian/templater, obsidian/dataview, javascript
---
# Journals by Date Dataview List

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]], [[Dataview]]
> Language: [[JavaScript]]
> Input:: Date, String
> Output:: Dataview Table
> Description:: Return a dataview list of journals according to date.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);

// Journal name and link
const link = `link(file.link, file.frontmatter.aliases[0])`;

// Journal source
const source = `-"00_system/05_templates"`;

// File class filter
const journal_filter = `contains(file.frontmatter.file_class, "journal")`;

async function dv_pdev_date(date) {
  dataview_journal_list = `${three_backtick}dataview
LIST WITHOUT ID
	${link}
FROM
	${source}
WHERE
	${journal_filter}
	AND contains(file.frontmatter.date_created, "${date}")
SORT
	file.frontmatter.date_created ASC
${three_backtick}`;

  return dataview_journal_list;
}

module.exports = dv_pdev_date;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// JOURNAL BY DATE DATAVIEW LIST
//---------------------------------------------------------
const dv_journal_list = await tp.user.dv_pdev_date(date);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[31_01_day_periodic]]
2. [[31_00_day]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_day_journal.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_proj_task|Dataview Table of Completed Project Tasks]]

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
