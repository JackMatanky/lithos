---
title: DataviewJS dv.queryMarkdown Function
aliases:
  - dv.queryMarkdown()
  - dv.queryMarkdown
  - dataviewjs_dv.queryMarkdown()
  - dv.queryMarkdown DataviewJS Function
  - The DataviewJS dv.queryMarkdown() Function
language:
  - javascript
plugin: dataview
module:
  - dataviewjs
class: query_evaluation
syntax: "dv.queryMarkdown(source, [file], [settings])"
url: https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvquerymarkdownsource-file-settings
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-06-06T10:19
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/dataview/dataviewjs/querymarkdown, dvjs/function/querymarkdown
---
# The DataviewJS `dv.queryMarkdown()` Function

## Description

> [!function] Function Details
>
> Plugin: [[Dataview]]
> Language: [[JavaScript]]
> Module: DataviewJS
> Class: Query Evaluation
> Input::
> Output::
> Definition:: Execute a Dataview query and return the results rendered in Markdown.
>
> Link: [dv.queryMarkdown](https://blacksmithgu.github.io/obsidian-dataview/api/code-reference/#dvquerymarkdownsource-file-settings)

---

## Syntax

```javascript
dv.queryMarkdown(source, [file], [settings])
```

## Parameter Values

| Parameter |  Type  | Description                                                                                                                                                               |
|:--------- |:------:|:------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| source    | string | The Dataview query to execute                                                                                                                                             |
| file      | string | The file path to resolve the query from (in case of references to `this`). Defaults to the current file.                                                                  |
| settings  |        | Execution settings for running the query. This is largely an advanced use case and it is recommend to directly check the API implementation to see all available options. |

## Additional Information

Execute a Dataview query and return the results as rendered Markdown. The return type of this function varies by the query type being executed, though will always be an object with a `type` denoting the return type. This version of `queryMarkdown` returns a result type - you may want [[DataviewJS dv.tryQueryMarkdown Function|dv.tryQueryMarkdown()]], which instead throws an error on failed query execution.

## Examples

```js
await dv.queryMarkdown("LIST FROM #tag") =>
    { successfult: true, value: { "- [[Page 1]]\n- [[Page 2]]" } }
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
FROM -"00_system/05_templates"
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Functions

#### Related Function Links

<!-- Link related functions here  -->

#### By Tag

<!-- Add tags in contains function as needed  -->
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, string(file.frontmatter.language) + " " + file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition,
	string(file.frontmatter.language) AS Language,
	sort(file.etags) AS Tags
FROM -"00_system/05_templates"
WHERE
	file.name != this.file.name
	AND contains(file.tags, "table")
	AND (file.frontmatter.file_class = "pkm_code_function")
SORT file.frontmatter.language, file.name
LIMIT 10
```

#### By Plugin

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.module AS Module,
	Definition AS Definition
WHERE
	file.name != this.file.name
	AND regextest("\w", file.frontmatter.plugin)
	AND (file.frontmatter.plugin = this.file.frontmatter.plugin)
	AND (file.frontmatter.file_class = "pkm_code_function")
SORT file.frontmatter.module, file.name
LIMIT 10
```

#### All Related Function

<!-- Excluding functions of the same plugin  -->
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	string(file.frontmatter.language) AS Language,
	Definition AS Definition
FROM -"00_system/05_templates"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND regextest("\w", file.frontmatter.plugin)
	AND (file.frontmatter.plugin != this.file.frontmatter.plugin)
	AND (file.frontmatter.file_class = "pkm_code_function")
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
