---
description: Show on which date which files were edited
topics:
  - visualization
tags: dv/calendar
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/calendar

# Show Modified File Dates in a Calendar view

## Basic

```dataview
CALENDAR file.mtime
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
