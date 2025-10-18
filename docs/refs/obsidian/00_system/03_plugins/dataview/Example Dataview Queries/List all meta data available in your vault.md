---
description: List all available meta data keys in your vault, to i.e. check which ones are duplicated or unused.
topics:
  - vault maintenance
tags: dv/dataviewjs, dvjs/pages, dvjs/list, dvjs/table, dvjs/array
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

 #dv/dataviewjs #dvjs/pages #dvjs/list #dvjs/table #dvjs/array

# List All Meta Data Available in Your Vault

## Basic

> [!info] Keys in lower case and without spaces  
> Keys are listed in lower case here. Due to the fact that dataview saves a "sanitized" meta data key that's all lower-case, keys containing capitalized letters would be duplicated. So `TotalPages` will show up as `totalpages`. The same counts for keys that contains spaces: The sanitized version uses dashes instead, so `go to sleep` will show up as `go-to-sleep`

```dataviewjs
const metadata = [];

dv.pages().forEach(page => {
	Object.keys(page).forEach(key => {
		key = key.toLowerCase().replaceAll(" ", "-")
		if (key === 'file' || metadata.indexOf(key) >= 0) return;

		metadata.push(key)
	})
})

dv.list(metadata.sort())
```

## Variants

### Rendering a Table with All Meta Data Fields and the Files They Are Contained in

> [!attention] Memory intensive calculation  
> Depending on the size of your vault and meta data usage, this query can cause your vault to freeze or crash. If you have a lot of data, better use the next variant that only shows up to 5 containing pages and is less straining on your computer.

> [!missing] Add `dataviewjs` to code block  
> This query is disabled by default, otherwise opening up this page would take quite some time. If you want to see the result - or use the query in your own vault, be sure to add a `dataviewjs` right on the first three backticks to enable it.

```
const metadataMap = {};

dv.pages().forEach(page => {
	Object.keys(page).forEach(metadata => {
		metadata = metadata.toLowerCase().replaceAll(" ", "-")
		if (metadata === 'file') return;
		if (!metadataMap[metadata]) {
			metadataMap[metadata] = []
		}
		if (!metadataMap[metadata].some(l => l.path === page.file.link.path)) {
			metadataMap[metadata].push(page.file.link);
		}
	})
})

dv.table(["meta data", "pages"], Object.keys(metadataMap).map(key => [key, metadataMap[key]]))
```

# Rendering a Table with All Meta Data Fields and the First 5 Files They Are Contained in

> [!attention] Memory intensive calculation  
> While this query is easier on your memory than the one listing all pages, it's still quite hungry. If the query does not render, try to reduce `pagelimit` to a smaller number.

```dataviewjs
const pagelimit = 5;
const metadataMap = {};

dv.pages().forEach(page => {
	Object.keys(page).forEach(metadata => {
		if (metadata === 'file') return;
		metadata = metadata.toLowerCase().replaceAll(" ", "-")
		if (!metadataMap[metadata]) {
			metadataMap[metadata] = []
		}
		if (!metadataMap[metadata].some(l => l.path === page.file.link.path)) {
			metadataMap[metadata].push(page.file.link);
		}
	})
})

dv.table(["meta data", "page count", `pages (first ${pagelimit})`], Object.keys(metadataMap).sort().map(key => [key, metadataMap[key].length, dv.array(metadataMap[key]).limit(pagelimit)]))
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
