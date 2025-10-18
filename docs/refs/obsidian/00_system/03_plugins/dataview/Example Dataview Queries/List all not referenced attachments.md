---
description: List all attachments that are not linked anywhere
topics:
  - attachments
  - unresolved links
tags: dv/dataviewjs, dvjs/pages, dvjs/filter, dvjs/list, dvjs/fileLink, dvjs/array, dvjs/map
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/dataviewjs #dvjs/pages #dvjs/filter #dvjs/list #dvjs/fileLink #dvjs/array #dvjs/map

# List All not Referenced Attachments

## Basic

```dataviewjs
const allNonMdFiles = app.vault.getFiles().filter(file => file.extension !== 'md')
const allNonMdOutlinks = dv.pages().file.outlinks.path.filter(link => !link.endsWith('.md'))
const notReferenced = allNonMdFiles.filter(file => !allNonMdOutlinks.includes(file.path));

dv.list(dv.array(notReferenced).map(link => dv.fileLink(link.path)))
```

## Variants

Be able to specify file endings to ignore and show a callout if no result is found

```dataviewjs
// add all extensions you want to ignore to the array, i.e. ["md", "js", "css"]
const allNonMdFiles = app.vault.getFiles().filter(file => !["md"].includes(file.extension))
const allNonMdOutlinks = dv.pages().file.outlinks.path.filter(link => !link.endsWith('.md'))
const notReferenced = allNonMdFiles.filter(file => !allNonMdOutlinks.includes(file.path));

if (!notReferenced.length) {
	dv.span(`> [!done] All good! No unused attachments found :) `)
} 

dv.list(dv.array(notReferenced).map(link => dv.fileLink(link.path)))
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
