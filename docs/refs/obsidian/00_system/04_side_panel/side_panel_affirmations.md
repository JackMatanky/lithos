---
title: side_panel_affirmations
aliases:
  - Side Panel Affirmations
  - side panel affirmations
  - side_panel_affirmations
cssclasses:
  - inline_title_hide
  - side_panel_style
  - list_narrow
  - read_hide_properties
file_class: pdev
date_created: 2023-07-19T10:10
date_modified: 2023-09-05T19:17
tags:
  - self_affirmation
---
- I control my urges and do not smoke cigarettes.
- I expect my parents will let me down and not change.
- I am capable and past achievements prove it.
- I succeeded in the past and succeed in the present.
- I define my successes and failures.
- I am successful as long as I try.
- I do not concern myself with others' expectations.
- I design a happy life with the skills I learn and implement.
- I am in control of how I spend my energy.
- I create and manage my work-life balance.

```dataview
LIST
	rows.D
FROM 
    "80_insight"
FLATTEN
	affirmation AS D
WHERE 
	contains(file.frontmatter.file_class, "pdev")
	AND regextest(".", affirmation)
GROUP BY
	link(file.link, file.frontmatter.aliases[0])
SORT
	file.frontmatter.date ASC
```