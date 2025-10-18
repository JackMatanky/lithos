---
description: List all bullet points under a certain heading
topics:
  - filter bullet points
tags: dv/table, dv/from, dv/where, dv/groupby, dv/flatten, dv/meta, dv/filter
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/from #dv/where #dv/groupby #dv/flatten #dv/meta #dv/filter

# Show All List Items under a Specific Heading

## Basic

All lists inside the section "Research"

```dataview
TABLE L.text AS "My lists"
FROM "10 Example Data/dailys"
FLATTEN file.lists AS L
WHERE meta(L.section).subpath = "Research"
```

## Variants

### Grouping by File

```dataview
TABLE rows.L.text AS "My lists"
FROM "10 Example Data/dailys"
FLATTEN file.lists AS L
WHERE meta(L.section).subpath = "Research"
GROUP BY file.link
```

### Using flatten/filter instead of Group-by

```dataview
TABLE WITHOUT ID "<nobr>" + file.link + "</nobr>" AS Page, Research
FROM "10 Example Data/dailys"
FLATTEN list(filter(file.lists, (x) => meta(x.section).subpath = "Research").text) as Research
WHERE Research
```

### Using Flatten to Make Multiple Columns Based on Different Headings

```dataview
TABLE WITHOUT ID "<nobr>" + file.link + "</nobr>" AS Page, Research, Topics
FROM "10 Example Data/dailys"
FLATTEN list(filter(file.lists, (x) => meta(x.section).subpath = "Research").text) as Research
FLATTEN list(filter(file.lists, (x) => meta(x.section).subpath = "Topics").text) as Topics
WHERE Research OR Topics
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
