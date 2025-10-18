---
description: Display Files grouped after metadata where one felt discomfort
topics:
  - grouping
  - group pages based on meta data
tags: dv/table, dv/from, dv/where, dv/groupby, dv/choice
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/from #dv/where #dv/groupby #dv/choice

# Group Files by Meta Data

## Basic

```
```dataview
TABLE rows.file.link, rows.wellbeing.pain-type
FROM #daily 
WHERE wellbeing.mood-notes = "discomfort"
GROUP BY wellbeing.pain
```

## Variants

Add better readable table headers

```
```dataview
TABLE WITHOUT ID row.key AS "Pain", rows.file.link AS "Dailys", rows.wellbeing.pain-type AS "Type of Pain"
FROM #daily 
WHERE wellbeing.mood-notes = "discomfort"
GROUP BY wellbeing.pain
```

---

Replace pain numbers with textual information

```
```dataview
TABLE WITHOUT ID choice(row.key = 0, "None", choice(row.key = 1, "Little", choice(row.key = 2, "Middle", choice(row.key = 3, "High", row.key))))  AS "Pain", rows.file.link AS "Dailys", rows.wellbeing.pain-type AS "Type of Pain"
FROM #daily 
WHERE wellbeing.mood-notes = "discomfort"
GROUP BY wellbeing.pain
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
