---
title: dv_pkm_type_status_dates
aliases:
  - PKM Files by Type, Status, and Dates Dataview Table
  - pkm files by type, status, and dates dataview table
  - Dataview Table of PKM Files by Type, Status, and Dates
  - dataview table of pkm files by type, status, and dates
  - PKM Files by Type, Status, and Dates
  - pkm files by type, status, and dates
  - dv pkm type status dates
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-25T10:15
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# PKM Files by Type, Status, and Date or Between Dates Dataview Table

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return a Dataview table of pkm files by type, status, date or dates written between two dates, primarily used in the weekly calendar.

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
const yaml_title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias}) AS Title`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]" AS Title`;

// YAML PKM Category
const yaml_category = `file.frontmatter.category`;

// YAML PKM Branch
const yaml_branch = `file.frontmatter.branch`;

// YAML PKM Field
const yaml_field = `file.frontmatter.field`;

// YAML PKM Subject
const yaml_subject = `file.frontmatter.subject`;

// YAML PKM Topic
const yaml_topic = `file.frontmatter.topic`;

// Status
const yaml_status = `file.frontmatter.status`;
const pkm_status = `choice(${yaml_status} = "review", "🌱️Review",
	choice(${yaml_status} = "clarify", "🌿️Clarify",
	choice(${yaml_status} = "develop", "🪴Develop",
	choice(${yaml_status} = "evergreen", "🌳Evergreen", "🗃️Resource"))))
	AS Status`;

// File type
const yaml_type = `file.frontmatter.type`;

// File subtype
const yaml_subtype = `file.frontmatter.subtype`;

// PKM Subtype
const pkm_subtype = `choice(contains(${yaml_subtype}, "category"), "🏘️Category",
	choice(contains(${yaml_subtype}, "branch"), "🪑Branch",
	choice(contains(${yaml_subtype}, "field"), "🚪Field",
	choice(contains(${yaml_subtype}, "subject"), "🗝️Subject",
	choice(contains(${yaml_subtype}, "topic"), "🧱Topic",
	choice(contains(${yaml_subtype}, "subtopic"), "🔩Subtopic",
	choice(contains(${yaml_subtype}, "question"), "❔Question",
	choice(contains(${yaml_subtype}, "evidence"), "⚖️Evidence",
	choice(contains(${yaml_subtype}, "conclusion"), "💡Conclusion",
	choice(contains(${yaml_subtype}, "problem"), "🪨Problem",
	choice(contains(${yaml_subtype}, "step"), "🪜Step",
	choice(contains(${yaml_subtype}, "answer"), "🎱Answer",
	choice(contains(${yaml_subtype}, "quote"), "⏺️Quote",
	choice(contains(${yaml_subtype}, "idea"), "💭Idea",
	choice(contains(${yaml_subtype}, "concept"), "🎞️Concept", "🪟Definition")))))))))))))))
	AS Subtype`;

// PKM Content
const pkm_content = `choice(${yaml_subtype} = "qec_question", list(Context, Question),
	choice(${yaml_subtype} = "qec_evidence", Evidence,
	choice(${yaml_subtype} = "qec_conclusion", Conclusion,
	choice(${yaml_subtype} = "psa_problem", list(Context, Problem),
	choice(${yaml_subtype} = "psa_step", Step,
	choice(${yaml_subtype} = "psa_answer", Answer,
	choice(${yaml_subtype} = "quote", Quote,
	choice(${yaml_subtype} = "idea", Idea,
	choice(${yaml_subtype} = "definition", Definition, Description)))))))))
	AS Content`;

// Knowledge Tree Subtype
const tree_subtype = `choice(contains(${yaml_subtype}, "category"), "🏘️Category",
	choice(contains(${yaml_subtype}, "branch"), "🪑Branch",
	choice(contains(${yaml_subtype}, "field"), "🚪Field",
	choice(contains(${yaml_subtype}, "subject"), "🗝️Subject",
	choice(contains(${yaml_subtype}, "topic"), "🧱Topic", "🔩Subtopic")))))
	AS Subtype`;

// Knowledge Tree Content
const tree_content = `Description AS Description`;

// Knowledge Tree Context
const tree_context = `choice(${yaml_subtype} = "subtopic", flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject, file.frontmatter.topic)),
	choice(${yaml_subtype} = "topic", flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject)),
	choice(${yaml_subtype} = "subject", flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field)),
	choice(${yaml_subtype} = "field", flat(list(file.frontmatter.category, file.frontmatter.branch)),
	choice(${yaml_subtype} = "branch", Category, "")))))
	AS Context`;

// Note Subtype
const note_subtype = `choice(contains(${yaml_subtype}, "question"), "❔Question",
	choice(contains(${yaml_subtype}, "evidence"), "⚖️Evidence",
	choice(contains(${yaml_subtype}, "conclusion"), "💡Conclusion",
	choice(contains(${yaml_subtype}, "problem"), "🪨Problem",
	choice(contains(${yaml_subtype}, "step"), "🪜Step",
	choice(contains(${yaml_subtype}, "answer"), "🎱Answer",
	choice(contains(${yaml_subtype}, "quote"), "⏺️Quote",
	choice(contains(${yaml_subtype}, "idea"), "💭Idea",
	choice(contains(${yaml_subtype}, "concept"), "🎞️Concept", "🪟Definition")))))))))
	AS Subtype`;

// File creation date
const creation_date = `file.frontmatter.date_created`;

// Tags
const tags = `file.etags AS Tags`;

//---------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//---------------------------------------------------------
// Knowledge tree directory
const tree_dir = `"70_pkm_tree"`;

// Knowledge lab directory
const lab_dir = `"80_pkm_lab"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTER <<<<<
//---------------------------------------------------------
// File class filter
const class_filter = `contains(file.frontmatter.file_class, "pkm")`;

//---------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//---------------------------------------------------------
const pkm_sort = `choice(${yaml_subtype} = "category", 1,
	choice(${yaml_subtype} = "branch", 2,
	choice(${yaml_subtype} = "field", 3,
	choice(${yaml_subtype} = "subject", 4,
	choice(${yaml_subtype} = "topic", 5,
	choice(${yaml_subtype} = "subtopic", 6,
	choice(${yaml_subtype} = "qec_question", 7,
	choice(${yaml_subtype} = "qec_evidence", 8,
	choice(${yaml_subtype} = "qec_conclusion", 9,
	choice(${yaml_subtype} = "psa_problem", 10,
	choice(${yaml_subtype} = "psa_step", 11,
	choice(${yaml_subtype} = "psa_answer", 12,
	choice(${yaml_subtype} = "quote", 13,
	choice(${yaml_subtype} = "idea", 14,
	choice(${yaml_subtype} = "concept", 15, 16)))))))))))))))`;

const tree_subtype_sort = `choice(${yaml_subtype} = "category", 1,
	choice(${yaml_subtype} = "branch", 2,
	choice(${yaml_subtype} = "field", 3,
	choice(${yaml_subtype} = "subject", 4,
	choice(${yaml_subtype} = "topic", 5, 6)))))`;

const note_subtype_sort = `choice(${yaml_subtype} = "qec_question", 1,
	choice(${yaml_subtype} = "qec_evidence", 2,
	choice(${yaml_subtype} = "qec_conclusion", 3,
	choice(${yaml_subtype} = "psa_problem", 4,
	choice(${yaml_subtype} = "psa_step", 5,
	choice(${yaml_subtype} = "psa_answer", 6,
	choice(${yaml_subtype} = "quote", 7,
	choice(${yaml_subtype} = "idea", 8,
	choice(${yaml_subtype} = "concept", 9, 10)))))))))`;

//---------------------------------------------------------
// DATAVIEW TABLE FOR NOTES WRITTEN BETWEEN DATES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR: TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// VAR: STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"

async function dv_pkm_type_status_dates({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  const type_arg = `${type}`;
  const status_arg = `${status}`;
  const start_date_arg = `${date_start}`;
  const end_date_arg = `${date_end}`;
  const md_arg = `${md}`;

  let data_field;
  if (type_arg == "") {
    data_field = `${title_link},
	${pkm_subtype},
	${pkm_status},
	${pkm_content}`;
  } else if (type_arg.startsWith("know")) {
    data_field = `${title_link},
	${tree_subtype},
	${tree_content},
	${tree_context},
	${tags}`;
  } else if (
    type_arg.startsWith("perm") ||
    type_arg.startsWith("lit") ||
    type_arg.startsWith("fle") ||
    type_arg.startsWith("info") ||
    type_arg.startsWith("not_tree")
  ) {
    data_field = `${title_link},
	${note_subtype},
	${pkm_status},
	${pkm_content},
	${tags}`;
  }

  let filter;
  let date_filter;
  // DATE FILTER
  if (end_date_arg == "") {
    // SPECIFIC date
    date_filter = `date(${creation_date}) = date(${start_date_arg})`;
  } else {
    // Completed BETWEEN dates
    date_filter = `date(${creation_date}) >= date(${start_date_arg})
    AND date(${creation_date}) <= date(${end_date_arg})`;
  }

  if (start_date_arg != "") {
    if (type_arg == "not_tree" && status_arg != "") {
      filter = `!contains(${yaml_type}, "know")
	  AND contains(${yaml_status}, "${status_arg}")
	  AND ${date_filter}`;
    } else if (type_arg != "" && status_arg != "") {
      filter = `contains(${yaml_type}, "${type_arg}")
	  AND contains(${yaml_status}, "${status_arg}")
	  AND ${date_filter}`;
    } else if (type_arg != "" && status_arg == "") {
      filter = `contains(${yaml_type}, "${type_arg}")
	  AND ${date_filter}`;
    } else if (type_arg == "" && status_arg != "") {
      filter = `contains(${yaml_status}, "${status_arg}")
	  AND ${date_filter}`;
    }
  } else {
    if (type_arg == "not_tree" && status_arg != "") {
      filter = `!contains(${yaml_type}, "know")
	  AND contains(${yaml_status}, "${status_arg}")`;
    } else if (type_arg != "" && status_arg != "") {
      filter = `contains(${yaml_type}, "${type_arg}")
	  AND contains(${yaml_status}, "${status_arg}")`;
    } else if (type_arg != "" && status_arg == "") {
      filter = `contains(${yaml_type}, "${type_arg}")`;
    } else if (type_arg == "" && status_arg != "") {
      filter = `contains(${yaml_status}, "${status_arg}")`;
    }
  }

  let sort_field;
  if (end_date_arg == "") {
    if (type_arg == "") {
      sort_field = `${pkm_sort},
	  ${yaml_title}`;
    } else if (type_arg.startsWith("know")) {
      sort_field = `${tree_subtype_sort},
	  ${yaml_title}`;
    } else {
      sort_field = `${note_subtype_sort},
	  ${yaml_title}`;
    }
  } else if (status_arg != "") {
    if (type_arg == "") {
      sort_field = `${creation_date},
	  ${pkm_sort},
	  ${yaml_title}`;
    } else if (type_arg.startsWith("know")) {
      sort_field = `${creation_date},
	  ${tree_subtype_sort},
	  ${yaml_title}`;
    } else {
      sort_field = `${creation_date},
	  ${note_subtype_sort},
	  ${yaml_title}`;
    }
  }

  let dataview_query;
  dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${data_field}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${class_filter}
	AND ${filter}
SORT
	${sort_field} ASC
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

module.exports = dv_pkm_type_status_dates;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// PKM FILES DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
const pkm_type_start_end_table = await tp.user.dv_pkm_type_status_dates({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
});
```

#### Examples

```javascript
//---------------------------------------------------------
// PKM FILES DATAVIEW TABLE
//---------------------------------------------------------
// TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
// STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
const week_note_perm = await tp.user.dv_pkm_type_status_dates({
  type: "permanent",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_lit = await tp.user.dv_pkm_type_status_dates({
  type: "literature",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_fleet = await tp.user.dv_pkm_type_status_dates({
  type: "type",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_info = await tp.user.dv_pkm_type_status_dates({
  type: "info",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_tree = await tp.user.dv_pkm_type_status_dates({
  type: "tree",
  status: "",
  start_date: date_start,
  end_date: date_end,
  md: "false",
});

const week_note_review = await tp.user.dv_pkm_type_status_dates({
  type: "",
  status: "review",
  start_date: "",
  end_date: "",
  md: "false",
});

const week_note_clarify = await tp.user.dv_pkm_type_status_dates({
  type: "",
  status: "clarify",
  start_date: "",
  end_date: "",
  md: "false",
});

const week_note_develop = await tp.user.dv_pkm_type_status_dates({
  type: "",
  status: "develop",
  start_date: "",
  end_date: "",
  md: "false",
});
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript
dv_class_type_status_start_end
```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
2. [[32_00_week|Weekly Calendar Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]
4. [[32_07_cal_week_pkm|Weekly PKM Dataview Tables]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[dv_pkm_type_status_dates.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_pkm_linked|Linked Personal Knowledge Files Dataview Table]]
2. [[dv_pkm_type_date(X)|PKM Files by Date and Type Dataview Table]]
3. [[dv_pdev_attr_dates|Journals and Attributes Between Dates Dataview Table]]
4. [[dv_lib_status_dates|Library Content By Status and Between Dates Dataview Table]]
5. [[dv_task_type_status_dates|Tasks and Events by Type, Status, and Dates Dataview Tables]]

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
