---
description: List all tasks that have a duedate (a meta data) set and are due to a specific date or before
topics:
  - task tracking
  - dailies
tags: dv/TASK, dv/WHERE, dv/date, dv/contains
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

 #dv/TASK #dv/WHERE #dv/date #dv/contains

# List Tasks that Are due Today or a Specific Day

## Basic

> [!info] Usage in daily notes  
> When used in a daily note that's named in format `YYYY-MM-DD`, you can replace the specific date information (`date("2022-11-30")`) with `this.file.day`

```
```dataview
TASK 
WHERE !completed AND duedate AND duedate <= date("2022-11-30") AND contains(text, "due")
```

## Variants

### Show Tasks that Are due Today or Earlier

```
```dataview
TASK 
WHERE !completed AND duedate AND duedate <= date(today) AND contains(text, "due")
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
