---
date_created: 2023-07-05T11:51
date_modified: 2023-10-25T16:22
---

```dataviewjs
dv.list(dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file
	.filter((x) => String(x.frontmatter.type).includes("project"))
	.map((p) => p.name)
	.sort()
	);
```
