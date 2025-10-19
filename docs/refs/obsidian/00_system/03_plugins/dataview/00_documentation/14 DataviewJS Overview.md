---
title:
  - 14 DataviewJS Overview
aliases:
  - 14 DataviewJS Overview
  - DataviewJS Overview
  - dataviewjs_overview
  - dataview_dataviewjs_overview
application: dataview
url: https://blacksmithgu.github.io/obsidian-dataview/api/intro/
file_class: lib_documentation
date_created: 2023-03-09T17:10
date_modified: 2023-10-25T16:22
tags:
---
# [Overview](https://blacksmithgu.github.io/obsidian-dataview/api/intro/)

The Dataview JavaScript API allows for executing arbitrary JavaScript with access to the dataview indices and query engine, which is good for complex views or interop with other plugins. The API comes in two flavors: plugin facing, and user facing (or 'inline API usage').

## Inline Access

You can create a "DataviewJS" block via:

```sql
```dataviewjs
dv.pages("#thing")…
```

Code executed in such codeblocks have access to the `dv` variable, which provides the entirety of the codeblock-relevant dataview API (like `dv.table()`, `dv.pages()`, and so on). For more information, check out the [codeblock API reference](../code-reference/).

## Plugin Access

You can access the Dataview Plugin API (from other plugins or the console) through `app.plugins.plugins.dataview.api`; this API is similar to the codeblock reference, with slightly different arguments due to the lack of an implicit file to execute the queries in. For more information, check out the [Plugin API reference](../code-reference/).
