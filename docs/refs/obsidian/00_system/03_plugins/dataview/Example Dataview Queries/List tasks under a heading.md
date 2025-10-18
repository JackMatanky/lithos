---
description: 'Lists tasks under a heading. Useful if you have i.e. a "Urgent" heading in project files'
topics:
  - task tracking
  - dailies
tags: dv/TASK, dv/FROM, dv/meta, dv/WHERE, dv/date
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

 #dv/TASK #dv/FROM #dv/meta #dv/WHERE #dv/date

# List Tasks under a Heading

## Basic

```dataview
TASK 
FROM "10 Example Data/projects" 
WHERE meta(section).subpath = "Urgent"
```

## Variants

### List Tasks from a Certain Daily

```dataview
TASK 
FROM "10 Example Data/dailys" 
WHERE file.day = date("2022-02-16") AND meta(section).subpath = "Gonna do this tmrw"
```

```dataview
TASK 
FROM "10 Example Data/dailys" 
WHERE file.day = date("2022-02-16") AND meta(section).subpath = "Gonna do this tmrw"
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
