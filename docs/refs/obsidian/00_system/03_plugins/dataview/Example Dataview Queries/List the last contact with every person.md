---
description: Show all persons in your vault and when you had last contact with them 
topics:
  - contacts
  - latest items
tags: dv/table, dv/max, dv/from, dv/where, dv/flatten, dv/groupby, dv/sort, dv/min, dv/choice
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/max #dv/from #dv/where #dv/flatten #dv/groupby #dv/sort #dv/min #dv/choice

> [!hint] Contributed by mnvwvnm - Thanks!

# List the Last contact with Every Person

## Basic

```
```dataview
TABLE WITHOUT ID
contactedPerson AS "Person",
max(rows.file.day) AS "Last contact"
FROM "10 Example Data/dailys"
WHERE person
FLATTEN person AS contactedPerson
GROUP BY contactedPerson
SORT max(rows.file.day) DESC
```

## Variants

### Calculate Elapsed Days since Last contact

```
```dataview
TABLE WITHOUT ID
contactedPerson AS "Person",
max(rows.file.link) AS "Last contact",
min(rows.elapsedDays) + " days" AS "Elapsed days"
FROM "10 Example Data/dailys"
WHERE person
FLATTEN (date(today) - file.day).days AS elapsedDays
FLATTEN person AS contactedPerson
GROUP BY contactedPerson
SORT max(rows.file.day) DESC
```

### Show a Graphical Representation how long the Last contact Has Passed

```
```dataview
TABLE WITHOUT ID
contactedPerson AS "Person",
max(rows.file.link) AS "Last contact",
min(rows.elapsedDays) + " days" AS "Elapsed days",
choice(min(rows.elapsedDays)<30, "🟢", choice(min(rows.elapsedDays)<60, "🟡", choice(min(rows.elapsedDays)<90, "🟠", "☎️"))) AS "Return contact"
FROM "10 Example Data/dailys"
WHERE person
FLATTEN (date(today) - file.day).days AS elapsedDays
FLATTEN person AS contactedPerson
GROUP BY contactedPerson
SORT max(rows.file.day) DESC
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
