---
description: Show a bullet point list as a table with their metadata and sections (headers) as columns
topics:
  - lists with metadata
tags: dv/table, dv/from, dv/flatten, dv/sort, dv/regexreplace
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/from #dv/flatten #dv/sort #dv/regexreplace

# Show List Items as Table with Their Metadata and Sections

## Basic

```dataview
TABLE WITHOUT ID L.text AS "Food", L.best-before AS "Best before ⬇"
FROM "10 Example Data/food/Food pantry"
FLATTEN file.lists AS L
SORT L.best-before
```

## Variants

### Show the Headings They Belong to as a Column

```dataview
TABLE WITHOUT ID L.text AS "Food", meta(L.section).subpath AS "Type", L.best-before AS "Best before ⬇"
FROM "10 Example Data/food/Food pantry"
FLATTEN file.lists AS L
SORT L.best-before
```

### Remove Meta Data from Bullet point Text

```dataview
TABLE WITHOUT ID regexreplace(L.text, "\[best-before:: [0-9-]+\]", "") AS "Food", meta(L.section).subpath AS "Type", L.best-before AS "Best before ⬇"
FROM "10 Example Data/food/Food pantry"
FLATTEN file.lists AS L
SORT L.best-before
```

---

<!-- === end of query page ===  -->

> [!help]- Similar Queries
> Maybe these queries are of interest for you, too:
>
> ```dataview
> LIST
> FROM "20 Dataview Queries"
> FLATTEN topics as flattenedTopics
> WHERE contains(this.topics, flattenedTopics)
> AND file.name != this.file.name
> ```

```dataviewjs
dv.view('00 Meta/dataview_views/usedInAUseCase',  { current: dv.current() })
```
