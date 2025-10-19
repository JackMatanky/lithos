---
description: 'List all bullet points from a certain date while ignoring the year, i.e. for "today last year" retrospectives'
topics:
  - dailies
  - bullet points filtering
tags: dv/LIST, dv/WHERE, dv/dateformat
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

 #dv/LIST #dv/WHERE #dv/dateformat

# List Bullet Points from Dailys of a Specific Date without Year

## Basic

```dataview
LIST file.lists.text
WHERE dateformat(file.day, "MM-dd") = "02-17"
```

## Variants

### Description of Variant A - what Does it Differently? What Do We Achieve with That?

> [!info] Usage in dailies
> When used in a daily and your dailies are named in format `YYYY-MM-DD`, the part `AND file.day.year!= this.file.day.year` will filter out the bullet points of the daily currently open. In the case of this example file it doesnt do anything, though. See [[2022-02-17]] for a working example.

```dataview
LIST file.lists.text
WHERE dateformat(file.day, "MM-dd") = "02-17" AND file.day.year != this.file.day.year
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
