---
description: Display metadata of a certain week as emojis (i.e. for habit tracking)
topics:
  - habit tracking
  - emojis as output
tags: dv/table, dv/from, dv/where, dv/choice
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/from #dv/where #dv/choice

# Display Metadata of a Week as Emjois (habit tracking)

## Basic

```dataview
TABLE choice(praying, "💚", "➖") AS Praying, choice(breathing, "💚", "➖") AS breathing, choice(beingthankful, "💚", "➖") AS "being thankful", choice(slowdown, "💚", "➖") AS "slow down"
FROM "10 Example Data/dailys"
WHERE wellbeing.mood > 0 AND date(file.day).weekyear = 2
```

## Variants

Add the mood of the day as a smiley

```dataview
TABLE choice(wellbeing.mood = 1, "😢", choice(wellbeing.mood = 2 or wellbeing.mood = 3, "😐", choice(wellbeing.mood > 3, "😃", ""))) as Mood, choice(praying, "💚", "➖") AS Praying, choice(breathing, "💚", "➖") AS breating, choice(beingthankful, "💚", "➖") AS "being thankful", choice(slowdown, "💚", "➖") AS "slow down"
FROM "10 Example Data/dailys"
WHERE wellbeing.mood > 0 AND date(file.day).weekyear = 2
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
