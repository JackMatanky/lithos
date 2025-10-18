---
description: Interpret a link to a daily note (in format YYYY-MM-DD) with a given time as a date
topics:
  - working with dates
tags: dv/list, dv/where, dv/flatten, dv/date, dv/regexreplace, dv/table
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/list #dv/where #dv/flatten #dv/date #dv/regexreplace #dv/table

> [!hint] Contributed by [via Discord](https://discord.com/channels/686053708261228577/875721010144477204/1012795332985360544)

# Use Daily File Names as Real Dates

## Basic

due:: [[2022-01-26]]

```dataview
LIST due.day
WHERE due
```

## Variants

### Take a time into account

dueWithTime:: [[2022-01-28]] 10:50

```dataview
LIST dueDate
WHERE dueWithTime
FLATTEN date(regexreplace(dueWithTime, "\[\[(.+?)\]\] (.+)", "$1T$2")) AS dueDate
```

### Ignore Trailing Text

dueWithTrailingText:: [[2022-01-30]] need to hand in assignment

```dataview
LIST dueDate
WHERE dueWithTrailingText
FLATTEN date(regexreplace(dueWithTrailingText, "\[\[(.+?)\]\].+", "$1")) AS dueDate
```

### Ignore Trailing Text but Keep time Information

dueWithTimeAndTrailingText:: [[2022-01-30]] 09:00 need to hand in assignment

```dataview
LIST dueDate
WHERE dueWithTimeAndTrailingText
FLATTEN date(regexreplace(dueWithTimeAndTrailingText, "\[\[(.+?)\]\] ?([0-9:]+).+", "$1T$2")) AS dueDate
```

### Split date/time and Trailing Text into Two Separate Information

```dataview
TABLE dueDate, subject
WHERE dueWithTimeAndTrailingText
FLATTEN date(regexreplace(dueWithTimeAndTrailingText, "\[\[(.+?)\]\] ?([0-9:]+).+", "$1T$2")) AS dueDate
FLATTEN regexreplace(dueWithTimeAndTrailingText, "\[\[(.+?)\]\] ?([0-9:]+)? (.+)", "$3") AS subject
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
