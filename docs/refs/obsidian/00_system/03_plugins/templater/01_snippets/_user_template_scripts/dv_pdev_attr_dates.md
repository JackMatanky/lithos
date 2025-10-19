---
title: dv_pdev_attr_dates
aliases:
  - Journals and Attributes Between Dates Dataview Table
  - Dataview Table of Journals and Attributes Between Dates
  - journals and attributes between dates dataview table
  - dataview table of journals and attributes between dates
  - dv journal between date
  - dv journal btwn date
  - dv week journal
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-20T13:57
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Journals and Attributes Between Dates Dataview Table

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a Dataview table of Journal's written between two dates, primarily used in the weekly calendar.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// Title
const title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias}) AS Title`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]" AS Title`;

// Journal frontmatter date
const yaml_date = `file.frontmatter.date`;
const date = `${yaml_date} AS Date`;

// Journal creation date
const creation_date = `file.frontmatter.date_created`;

//---------------------------------------------------------
// SECT: >>>>> DATA SOURCES <<<<<
//---------------------------------------------------------
// Insight directory
const insight_dir = `"80_insight"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//---------------------------------------------------------
// File class filter
const class_filter = `contains(file.frontmatter.file_class, "pdev")`;

// Detachment and achievement filters
const ordinal_section_filter = `regextest("(1st)|(2nd)|(3rd)|(4th)", regexreplace(string(L.section), "(.+\\>\\s)|(\\]\\]$)", ""))`;

// Gratitude and self gratitude filter
const gratitude_filter = `regextest("(I)|(For)", regexreplace(string(L.section), "(.+\\>\\s)|(\\]\\]$)", ""))`;

// Limiting belief filter
const limiting_belief_filter = `regextest("Limiting", regexreplace(string(L.section), "(.+\\>\\s)|(\\]\\]$)", ""))`;

//---------------------------------------------------------
// SECT: >>>>> DATA GROUPING <<<<<
//---------------------------------------------------------
// section group
const section_group = `link(L.section, dateformat(date(${yaml_date}), "DDDD") + regexreplace(string(L.section), ".+>|\]\]$", ""))`;

// Journal date link
const date_link = `link(file.link, dateformat(date(${yaml_date}), "DDDD"))`;

//---------------------------------------------------------
// DATAVIEW TABLE FOR JOURNALS WRITTEN BETWEEN DATES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR: JOURNAL: "file",
// VAR: ATTR: "recount", "best-experience", "blindspot", "achievement",
// VAR: ATTR: "gratitude", "detachment", "limiting_belief", "lesson"

async function dv_pdev_attr_dates({
  attribute: attribute,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  const attribute_arg = `${attribute}`;
  const md_arg = `${md}`;

  const date_start_filter = `date(${creation_date}) >= date(${date_start})`;
  const date_end_filter = `date(${creation_date}) <= date(${date_end})`;

  let dataview_query;

  if (attribute_arg == "detachment") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    file.lists AS L
WHERE
    ${ordinal_section_filter}
    AND contains(file.frontmatter.type, "${attribute_arg}")
    AND ${class_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else if (attribute_arg == "achievement") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    file.lists AS L
WHERE
    ${ordinal_section_filter}
    AND contains(file.frontmatter.type, "reflection")
    AND ${class_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else if (attribute_arg == "gratitude") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    file.lists AS L
WHERE
    ${gratitude_filter}
    AND contains(file.frontmatter.type, "${attribute_arg}")
    AND ${class_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else if (attribute_arg == "limiting_belief") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    file.lists AS L
WHERE
    ${limiting_belief_filter}
    AND contains(file.frontmatter.type, "${attribute_arg}")
    AND ${class_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else {
    dataview_query = `${dataview_block}
LIST
    rows.D
FROM
    ${insight_dir}
FLATTEN
    ${attribute_arg} AS D
WHERE
    ${class_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
    AND regextest(".", ${attribute_arg})
GROUP BY
    ${date_link}
SORT
    ${yaml_date} ASC
${three_backtick}`;
  }

  if (attribute_arg == "file") {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${title_link},
    ${date}
FROM
    ${insight_dir}
WHERE
    ${class_filter}
    AND ${date_start_filter}
    AND ${date_end_filter}
SORT
    ${yaml_date} ASC
${three_backtick}`;
  }

  if (md_arg == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    const md_query = String(
      dataview_query
        .replace(dataview_block_start_regex, "")
        .replace(dataview_block_end_regex, "")
        .replaceAll(/\n\s+/g, " ")
        .replaceAll(/\n/g, " ")
        .replace(title_link, md_title_link)
    );

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
  }

  return dataview_query;
}

module.exports = dv_pdev_attr_dates;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// PDEV DATAVIEW TABLE
//---------------------------------------------------------
// MD: "true", "false"
// JOURNAL: "file",
// ATTR: "recount", "best-experience", "blindspot", "achievement",
// ATTR: "gratitude", "detachment", "limiting_belief", "lesson"
const pdev_attr_start_end_table = await tp.user.dv_pdev_attr_dates({
  attribute: attribute,
  start_date: date_start,
  end_date: date_end,
  md: md,
});
```

#### Examples

```javascript
//---------------------------------------------------------
// PDEV DATAVIEW TABLE
//---------------------------------------------------------
// MD: "true", "false"
// JOURNAL: "file",
// ATTR: "recount", "best-experience", "blindspot", "achievement",
// ATTR: "gratitude", "detachment", "limiting_belief", "lesson"

// >>>>> WEEK TEMPLATE <<<<<
const week_day_journals = await tp.user.dv_pdev_attr_dates({
  attribute: "file",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_recount = await tp.user.dv_pdev_attr_dates({
  attribute: "recount",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_best_experiecnes = await tp.user.dv_pdev_attr_dates({
  attribute: "best-experience",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_achievements = await tp.user.dv_pdev_attr_dates({
  attribute: "achievement",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_gratitude = await tp.user.dv_pdev_attr_dates({
  attribute: "gratitude",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_blindspots = await tp.user.dv_pdev_attr_dates({
  attribute: "blindspot",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_detachments = await tp.user.dv_pdev_attr_dates({
  attribute: "detachment",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_limiting_beliefs = await tp.user.dv_pdev_attr_dates({
  attribute: "limiting_belief",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lessons_learned = await tp.user.dv_pdev_attr_dates({
  attribute: "lesson",
  start_date: date_start,
  end_date: date_end,
  md: "false",
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

1. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
2. [[32_00_week|Weekly Calendar Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]
4. [[32_05_cal_week_journal|Weekly Journal Dataview Tables]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_pdev_attr_start_end.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_pdev_date|Journals by Date Dataview List]]

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
