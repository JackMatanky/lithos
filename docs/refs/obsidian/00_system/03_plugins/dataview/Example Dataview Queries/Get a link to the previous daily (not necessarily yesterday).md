---
description: Fetches the file that has the highest day that is lower than the current files day in order to get the previous day
topics:
  - navigation
date: "[[2022-07-07]]"
tags: dv/list, dv/from, dv/where, dv/sort, dv/limit
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/list #dv/from #dv/where #dv/sort #dv/limit

# Get a Link to the Previous Daily (not Necessarily yesterday)

## Basic

```dataview
LIST
FROM "10 Example Data/dailys"
WHERE file.name != this.file.name AND file.day < this.file.day
SORT file.day DESC
LIMIT 1
```

## Variants

### As Javascript Inline Statement

`$= dv.pages('"10 Example Data/dailys"').where(p => p.file.day && p.file.day < dv.current().file.day).sort(p => p.file.day, "desc").file.link.limit(1)`

### Show a Custom Prefix before the Link

```dataview
LIST WITHOUT ID t
FROM "10 Example Data/dailys"
WHERE file.name != this.file.name AND file.day < this.file.day
SORT file.day DESC
FLATTEN "Previous day: " + file.link AS t
LIMIT 1
```

### Get a Link to the next Day with a Daily Note

```dataview
LIST WITHOUT ID t
FROM "10 Example Data/dailys"
WHERE file.name != this.file.name AND file.day > this.file.day
SORT file.day ASC
FLATTEN "Next day: " + file.link AS t
LIMIT 1
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
