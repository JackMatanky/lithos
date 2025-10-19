---
description: Showcase basic syntax of LIST queries
topics:
  - basics
tags: dv/list, dv/from, dv/where, dv/sort, dv/groupby
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/list #dv/from #dv/where #dv/sort #dv/groupby

# Basic List Queries

## Basic

**List pages from a folder**

```
``dataview
LIST
FROM "10 Example Data/games"
```

**List pages from a tag**

```
```dataview
LIST
FROM #type/books
```

**Combine multiple tags**

```
``dataview
LIST
FROM #dvjs/el OR #dv/min
```

**Combine multiple folders**

```
```dataview
LIST
FROM "10 Example Data/books" OR "10 Example Data/games"
```

**Combine tags and folders**

```
```dataview
LIST
FROM "10 Example Data/games" AND #genre/action
```

**List all pages**

> [!attention] Add `dataview` to code block
> The output of this is pretty long. If you want to see it, add `dataview` to the code block - like on the examples above!
> Please note: There needs to be a **space** behind `LIST` to see results!

```
LIST
```

## Variants

### List Pages from a Certain Author

```
```dataview
LIST
FROM #type/books
WHERE author = "Conrad C"
```

### List Pages and Show a Meta Data Field

> [!attention] Only one additional information
> For lists, you can only add **one** additional output. For more, you need to use a [[Basic Table Queries|table]] or [[How to create custom outputs in queries|create a custom output]].

```
```dataview
LIST author
FROM #type/books
```

### List Meta Data Values instead of the Pages

i.e. list source links of your recipes:

```
```dataview
LIST WITHOUT ID source
FROM "10 Example Data/food"
WHERE source
```

### Group List Elements

![[What is#^new-id-after-grouping]]

```
```dataview
LIST rows.file.link
FROM "10 Example Data/books"
GROUP BY author
```

### Sort List Elements

```
```dataview
LIST author
FROM "10 Example Data/books"
SORT author
```

> [!hint] Advanced usage
> Do you want to see more advanced examples? Head over to the [[Queries by Type#List|Query Type Overview]] to see all available LIST queries in the vault!

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
