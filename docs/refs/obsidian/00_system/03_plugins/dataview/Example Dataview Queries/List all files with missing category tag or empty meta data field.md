---
description: Find files that are missing required meta data, like a category tag or a meta data field that should have a value
topics:
  - vault maintenance
tags: dv/contains, dv/list, dv/from, dv/where, dv/length
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/contains #dv/list #dv/from #dv/where #dv/length

# List All Files with Missing Category Tag

## Basic

**Look for missing category tags**

```dataview
LIST
FROM "10 Example Data/books"
WHERE !contains(file.tags, "#type/")
```

**Look for empty, but required meta data**

```dataview
LIST
FROM "10 Example Data/books"
WHERE !author
```

## Variants

### Check for Empty Meta Data Fields that Are Lists

> [!hint] Use case
> This is handy when you prefill your yaml frontmatter, i.e. via a template, with something like
>
> ```
> genres:
>   -
> ```
>
> `genres` is *not* empty in this case and won't be found via `WHERE!genres`

```dataview
LIST
FROM "10 Example Data/books"
WHERE !genres OR (length(genres) = 1 AND contains(genres, null))
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
