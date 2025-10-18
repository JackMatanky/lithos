---
description: Show a calender that marks all days with praying = yes
topics:
  - habit tracking
tags: dv/table, dv/from, dv/where
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/from #dv/where

# Show a Calendar with All Days You've Prayed

## Basic

> [!info]  
> You'll need to go back to Januray/Februrary 2022 to see the data.

```dataview
CALENDAR file.day
FROM "10 Example Data/dailys"
WHERE praying = "yes"
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
