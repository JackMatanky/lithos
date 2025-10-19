---
title: DataviewJS dv.table Function
aliases:
  - dv.table()
  - dv.table
  - dataviewjs_dv.table()
  - dv.table DataviewJS Function
  - The DataviewJS dv.table() Function
language:
  - javascript
plugin: dataview
module:
  - dataviewjs
class: dataviews
syntax: "dv.table(headers, elements)"
url: https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvtableheaders-elements
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-06-04T10:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/dataviewjs/table, dvjs/function/table
---
# The DataviewJS `dv.table()` Function

## Description

> [!function] Function Details
>
> Plugin: [[Dataview]]
> Language: [[JavaScript]]
> Module: DataviewJS
> Class: Dataviews
> Input::
> Output::
> Definition:: Render a dataview table.
>
> Link: [dv.table](https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvtableheaders-elements)

---

## Syntax

```javascript
dv.table(headers, elements)
```

## Parameter Values

| Parameter | Type  | Description                |
|:--------- |:-----:|:-------------------------- |
| headers   | array | an array of column headers |
| elements  | array | an array of rows           |

## Additional Information

Each row is itself an array of columns. Inside a row, every column which is an array will be rendered with bullet points.

## Examples

```js
dv.table(
    ["Col1", "Col2", "Col3"],
        [
            ["Row1", "Dummy", "Dummy"],
            ["Row2",
                ["Bullet1",
                 "Bullet2",
                 "Bullet3"],
             "Dummy"],
            ["Row3", "Dummy", "Dummy"]
        ]
    );
```

An example of how to render a simple table of book info sorted by rating.

```javascript
dv.table(["File", "Genre", "Time Read", "Rating"], dv.pages("#book")
    .sort(b => b.rating)
    .map(b => [b.file.link, b.genre, b["time-read"], b.rating]))
```

## Notes and Remarks

---

## Related

### Snippets (Use Cases)

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

### Functions

#### By Plugin

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.module AS Module,
	Definition AS Definition
WHERE
	file.name != this.file.name
	AND (file.frontmatter.file_class = "pkm_code_function")
	AND (file.frontmatter.plugin = this.file.frontmatter.plugin)
SORT file.frontmatter.module, file.name
```

#### By Tag

<!-- Add tags in contains function as needed  -->
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition,
	string(file.frontmatter.language) AS Language,
	sort(file.etags) AS Tags
WHERE
	file.name != this.file.name
	AND file.frontmatter.file_class = "pkm_code_function"
	AND contains(file.tags, "table")
SORT file.frontmatter.language, file.name
LIMIT 10
```

#### Outgoing Function Links

<!-- Link related functions here -->

#### All Function Links

<!-- Excluding functions of the same module  -->
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.definition AS Definition
WHERE
	file.name != this.file.name
	AND file.frontmatter.module != this.file.frontmatter.module
	AND file.frontmatter.file_class = "pkm_code_function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
