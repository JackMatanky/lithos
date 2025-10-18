---
description: Display random images from a path in vault
topics:
  - random
  - images
tags: dv/dataviewjs, dvjs/list, dvjs/array, dvjs/sort, dvjs/limit, dvjs/map
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

#dv/dataviewjs #dvjs/list #dvjs/array #dvjs/sort #dvjs/limit #dvjs/map

# Display Random Images from a Path

## Basic

```dataviewjs
const amountOfImages = 3; 
const pathToImageFolder = '00 Meta/attachments' 
const imageFiles = app.vault.getFiles().filter(file => file.extension !== 'md' && file.parent.path === pathToImageFolder) 

dv.list(dv.array(imageFiles)
		.sort(() => 0.5 - Math.random())
		.limit(amountOfImages)
		.map(file => dv.func.embed(dv.fileLink(file.path))))
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
