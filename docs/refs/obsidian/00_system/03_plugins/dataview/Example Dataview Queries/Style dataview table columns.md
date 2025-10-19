---
description: Style columns of your output by i.e. right-align table columns, add bold or italic font style or color text without the need of a css snippet
topics:
  - custom output
tags: dv/TABLE, dv/FROM, dv/FLATTEN
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

 #dv/TABLE #dv/FROM #dv/FLATTEN

> [!hint] Contributed by [mnvwvnm](https://discord.com/channels/686053708261228577/1014259487445622855/1027857346019459153)

# Style Dataview Table Columns

## Basic

```dataview
TABLE
	publisher,
	developer,
    "<span style='display:flex; justify-content: right;'>" + price + "</span>" AS Price
FROM "10 Example Data/games"
```

> [!info] Other style possibilities
> For bold, italic, highlighted or strikethrough text, see the first variant.
> **Underscore text**: `<span style='text-decoration: underline;'> `
> **Right alignment**: `<span style='display:flex; justify-content: right;'> `
> **Center alignment**: `<span style='display:flex; justify-content: center;'> `
> **Make text uppercase**: `<span style='text-transform: uppercase;'> `
> **Text color**: `<span style='color: red;'> `

## Variants

### Use Bold, Italic or Highlight Text Styles

> [!info] Available styles
> You can use every style [Obsidian has available](https://help.obsidian.md/How+to/Format+your+notes) this way.

```dataview
TABLE
	"_" + publisher + "_" AS Publisher,
	"**" + developer + "**" AS Developer,
    "==" + price + "==" AS Price
FROM "10 Example Data/games"
```

### Style Multiple Columns

```dataview
TABLE styleStart + author + styleEnd AS Author,
	genres,
	styleStart + totalPages + styleEnd AS "Total Pages"
FROM "10 Example Data/books"
FLATTEN "<span style='display:flex; justify-content: center;'>" AS styleStart
FLATTEN "</span>" AS styleEnd
```

### Use Different Styles

```dataview
TABLE greenStyle + author + styleEnd AS Author,
	genres,
	rightAlignStyle + totalPages + styleEnd AS "Total Pages"
FROM "10 Example Data/books"
FLATTEN "<span style='color: lightgreen;'>" AS greenStyle
FLATTEN "<span style='display:flex; justify-content: right;'>" AS rightAlignStyle
FLATTEN "</span>" AS styleEnd
```

### Style Page Link

```dataview
TABLE WITHOUT ID styleStart + file.link + styleEnd AS "Book",
	author,
	totalPages
FROM "10 Example Data/books"
FLATTEN "<span style='text-transform: uppercase;'>" AS styleStart
FLATTEN "</span>" AS styleEnd
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
