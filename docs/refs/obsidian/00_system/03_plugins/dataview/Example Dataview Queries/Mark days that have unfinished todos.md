---
description: Show a calender that marks all days with uncompleted tasks
topics:
  - task tracking
tags: dv/calendar, dv/from, dv/flatten, dv/where, dv/all, dv/map, dv/any
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/calendar #dv/from #dv/flatten #dv/where #dv/all #dv/map #dv/any

# Mark Days that Have Unfinished Todos

> [!info]
> You'll need to go back to Januray/Februrary 2022 to see the data.

## Basic

```dataview
CALENDAR file.day
FROM "10 Example Data/dailys"
FLATTEN all(map(file.tasks, (x) => x.completed)) AS "allCompleted"
WHERE !allCompleted
```

> [!tip]
> When you try to write complex calendar queries, write a TABLE query first to make sure your query returns the results you're expecting.

```dataview
TABLE file.day, allCompleted
FROM "10 Example Data/dailys"
FLATTEN all(map(file.tasks, (x) => x.completed)) AS "allCompleted"
WHERE allCompleted
```

## Variants

### If You Use Custom Task Status and want to See All without a Status

```dataview
CALENDAR file.day
FROM "10 Example Data/dailys"
FLATTEN any(map(file.tasks, (x) => x.status = " ")) AS "anyEmpty"
WHERE anyEmpty
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
