---
title: dv_lib_status_dates
aliases:
  - Library Content By Status and Dates Dataview Table
  - Dataview Table for Library Content By Status and Dates
  - dv lib status dates
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-25T12:58
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Library Content By Status and Dates Dataview Table

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a Dataview table of content completed, created between two dates, created between two dates or on a specific date, with undetermined status, or active content.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias}) AS Title`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]" AS Title`;

// File type
const yaml_type = `file.frontmatter.type`;
const file_type = `choice(${yaml_type} = "book", "📚Book",
	choice(${yaml_type} = "book_chapter", "📑Book Chapter",
	choice(${yaml_type} = "journal", "📜️Journal",
	choice(${yaml_type} = "report", "📈Report",
	choice(${yaml_type} = "news", "🗞️News",
	choice(${yaml_type} = "magazine", "📰️Magazine",
	choice(${yaml_type} = "webpage", "🌐Webpage",
	choice(${yaml_type} = "blog", "💻Blog",
	choice(${yaml_type} = "video", "🎥️Video",
	choice(${yaml_type} = "youtube", "▶YouTube",
	choice(${yaml_type} = "documentary", "🖼️Documentary",
	choice(${yaml_type} = "audio", "🔉Audio",
	choice(${yaml_type} = "podcast", "🎧️Podcast", "📃Documentation")))))))))))))
	AS Type`;

// Status
const yaml_status = `file.frontmatter.status`;
const lib_status = `choice(${yaml_status} = "undetermined", "🤷Unknown",
	choice(${yaml_status} = "to_do", "🔜To do",
	choice(${yaml_status} = "in_progress", "👟In progress",
	choice(${yaml_status} = "done", "✔️Done",
	choice(${yaml_status} = "resource", "🗃️Resource",
	choice(${yaml_status} = "schedule", "📅Schedule", "🤌On hold"))))))
	AS Status`;

// Author
const author = `Author AS Author`;

// Date published
const date_publish = `choice(contains(file.frontmatter.type, "book"), file.frontmatter.year_published, file.frontmatter.date_published) AS "Published"`;

// Tags
const tags = `file.etags AS Tags`;

//---------------------------------------------------------
// SECT: >>>>> DATA SOURCES <<<<<
//---------------------------------------------------------
// Library directory
const lib_dir = `"60_library"`;

// Inbox directory
const inbox_dir = `"inbox"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//---------------------------------------------------------
// File class filters
const class_filter = `contains(file.frontmatter.file_class, "lib")`;

// Resource content filter
const resource_filter = `contains(${yaml_status}, "resource")`;

//---------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//---------------------------------------------------------
const lib_status_sort = `choice(${yaml_status} = "done", 1,
	choice(${yaml_status} = "in_progress", 2,
	choice(${yaml_status} = "to_do", 3,
	choice(${yaml_status} = "schedule", 4,
	choice(${yaml_status} = "resource", 5,
	choice(${yaml_status} = "on_hold", 6, 7))))))`;

//---------------------------------------------------------
// DATAVIEW TABLE FOR JOURNALS WRITTEN BETWEEN DATES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// COMPLETED OPTIONS: "done", "completed",
// ACTIVE OPTIONS: "active", "to_do", "in_progress"
// SCHEDULE OPTIONS: "schedule", "on_hold"
// CREATED OPTIONS: "new", "created"
// DETERMINE OPTIONS: "undetermined", "determine"

async function dv_lib_status_dates({
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  const status_arg = `${status}`;
  const start_date_arg = `${date_start}`;
  const end_date_arg = `${date_end}`;
  const md_arg = `${md}`;

  let sort_field = yaml_type;
  let date_field = "null";
  let filter = "null";

  // DATE fields DONE, NEW, and MODIFIED statuses
  if (status_arg.startsWith("don") || status_arg.startsWith("comp")) {
    date_field = "Completed";
    sort_field = "Completed";
  } else if (status_arg.startsWith("new") || status_arg.startsWith("created")) {
    date_field = "file.frontmatter.date_created";
  } else if (status_arg.startsWith("mod")) {
    date_field = "file.frontmatter.date_modified";
  }

  // DATE FILTER
  if (end_date_arg == "") {
    // SPECIFIC date
    filter = `date(${date_field}) = date(${start_date_arg})`;
  } else {
    // BETWEEN two dates
    filter = `date(${date_field}) >= date(${start_date_arg})
    AND date(${date_field}) <= date(${end_date_arg})`;
  }

  // FILTERs for ACTIVE, SCHEDULE, and UNDETERMINED statuses
  if (status_arg.startsWith("act")) {
    // Active content filter
    filter = `(contains(${yaml_status}, "to_do")
      OR contains(${yaml_status}, "in_progress"))`;
  } else if (status_arg.startsWith("sch")) {
    filter = `(contains(${yaml_status}, "schedule")
      OR contains(${yaml_status}, "on_hold"))`;
  } else if (status_arg.startsWith("und") || status_arg.startsWith("det")) {
    filter = `contains(${yaml_status}, "undetermined")`;
  }

  let dataview_query;
  dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${title_link},
    ${lib_status},
    ${author},
    ${date_publish},
    ${file_type},
    ${tags}
FROM
    ${lib_dir}
    OR ${inbox_dir}
WHERE
    ${class_filter}
    AND ${filter}
SORT
    ${lib_status_sort},
    ${sort_field},
    ${alias} ASC
LIMIT 25
${three_backtick}`;

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

module.exports = dv_lib_status_dates;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// LIBRARY DATAVIEW TABLE
//---------------------------------------------------------
// COMPLETED OPTIONS: "done", "completed",
// ACTIVE OPTIONS: "active", "to_do", "in_progress"
// SCHEDULE OPTIONS: "schedule", "on_hold"
// CREATED OPTIONS: "new", "created"
// DETERMINE OPTIONS: "undetermined", "determine"
const dv_library_table = await tp.user.dv_lib_status_dates({
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
});
```

#### Examples

```javascript
//---------------------------------------------------------
// LIBRARY DATAVIEW TABLE
//---------------------------------------------------------
// COMPLETED OPTIONS: "done", "completed",
// ACTIVE OPTIONS: "active", "to_do", "in_progress"
// SCHEDULE OPTIONS: "schedule", "on_hold"
// CREATED OPTIONS: "new", "created"
// DETERMINE OPTIONS: "undetermined", "determine"
const week_lib_done = await tp.user.dv_lib_status_dates({
  status: "done",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_active = await tp.user.dv_lib_status_dates({
  status: "active",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_new = await tp.user.dv_lib_status_dates({
  status: "new",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_schedule = await tp.user.dv_lib_status_dates({
  status: "schedule",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_lib_undetermined = await tp.user.dv_lib_status_dates({
  status: "determine",
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
4. [[32_06_cal_week_library|Weekly Library Dataview Tables]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_lib_status_dates.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_pkm_status(X)|Library Content by Status Dataview Table]]
2. [[dv_lib_content created or modified(X)|Content by Created or Modified Date Dataview Table]]
3. [[dv_pkm_type_status_dates|PKM Files by Type and Between Dates Dataview Table]]
4. [[dv_pdev_attr_dates|Journals and Attributes Between Dates Dataview Table]]
5. [[32_06_cal_week_library|Weekly Library Dataview Tables]]

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
