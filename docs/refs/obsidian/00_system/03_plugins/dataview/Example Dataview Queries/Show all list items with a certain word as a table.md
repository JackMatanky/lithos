---
description: Print all Bullet Points that contain a certain word
topics:
  - filter bullet points
tags: dv/table, dv/from, dv/where, dv/groupby, dv/flatten, dv/icontains
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/from #dv/where #dv/groupby #dv/flatten #dv/icontains

# Show All List Items with a Certain Word as a Table

## Basic

```dataview
TABLE L.text AS "My lists"
FROM "10 Example Data/dailys"
FLATTEN file.lists AS L
WHERE icontains(L.text, "ipsum")
```

## Variants

### Group Items by File

```dataview
TABLE rows.L.text AS "My lists"
FROM "10 Example Data/dailys"
FLATTEN file.lists AS L
WHERE icontains(L.text, "ipsum")
GROUP BY file.link
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
