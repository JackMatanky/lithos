---
description: List pages that have the same value in a field than the current one, i.e. to find recipes that share ingredients
topics:
  - navigation
  - group pages based on meta data
ingredients:
  - parsley
  - bacon
recipe-type: vegetarian
tags: dv/table, dv/from, dv/where, dv/flatten, dv/contains, dv/groupby, dv/length, dv/sort
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/table #dv/from #dv/where #dv/flatten #dv/contains #dv/groupby #dv/length #dv/sort

> [!hint] Contributed [via Discord](https://discord.com/channels/686053708261228577/875721010144477204/1006083409631789086)

# List Pages that Share a Meta Data Value with the Current Page

## Basic

**If you have a single value**:

```
```dataview
LIST
FROM "10 Example Data/food"
WHERE recipe-type = this.recipe-type
```

**If you have a multi value field (list)**:

```
```dataview
TABLE ings AS "shared ingredient"
FROM "10 Example Data/food"
WHERE ingredients
FLATTEN ingredients as ings
WHERE contains(this.ingredients, ings) AND file.name != this.file.name
```

## Variants

### Show File only once as Result

```
```dataview
TABLE rows.ings AS "shared ingredient"
FROM "10 Example Data/food"
WHERE ingredients
FLATTEN ingredients as ings
WHERE contains(this.ingredients, ings) AND file.name != this.file.name
GROUP BY file.link
```

### Group by Ingredient instead of file/recipe

```
```dataview
TABLE WITHOUT ID ings AS "shared ingredient", rows.file.link AS "recipes"
FROM "10 Example Data/food"
WHERE ingredients
FLATTEN ingredients as ings
WHERE contains(this.ingredients, ings) AND file.name != this.file.name
GROUP BY ings
```

### Calculate Count of Meta Data Value Matches

```
```dataview
TABLE WITHOUT ID ings AS "shared ingredient", count AS "recipe count", rows.file.link AS "recipes"
FROM "10 Example Data/food"
WHERE ingredients
FLATTEN ingredients as ings
WHERE contains(this.ingredients, ings) AND file.name != this.file.name
GROUP BY ings
FLATTEN length(rows.file.link) as count
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
