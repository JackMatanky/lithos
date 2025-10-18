---
date_created: 2023-07-05T11:31
date_modified: 2023-10-25T16:22
---

```dataviewjs
const datetime = dv.luxon.DateTime;
const today = datetime.now().toFormat('yyyy-MM-dd');
const pages = dv.pages('"80_insight/97_detachment"').file;

const journal_list = dv.list(pages
	.filter((t) => dv.equal(datetime.fromISO(t.frontmatter.date_created).toFormat('yyyy-MM-dd'), today))
    .map((p) => p.link)
);
```
