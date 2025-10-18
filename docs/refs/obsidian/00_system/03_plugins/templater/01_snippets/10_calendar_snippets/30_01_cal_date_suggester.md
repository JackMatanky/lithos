---
title: 30_01_cal_date_suggester
aliases:
  - Calendar Date Suggester
  - suggester_cal_date
  - cal_date_suggester
plugin: templater
language:
  - javascript
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T18:24
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Calendar Date Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Choose the date of the weekly, monthly, quarterly, or yearly calendar file.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET THE DATE
//---------------------------------------------------------  
const date_obj_arr = [
  { key: `Current ${type_name}`, value: `current_${type_value}` },
  { key: `Last ${type_name}`, value: `last_${type_value}` },
  { key: `Next ${type_name}`, value: `next_${type_value}` },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  `Which ${type_name}?`
);

const date_value = date_obj.value;

let full_date = ``;

if (date_value.startsWith(`current`)) {  
  full_date = moment();  
} else if (date_value.startsWith(`next`)) {  
  full_date = moment().add(1, moment_var);  
} else {  
  full_date = moment().subtract(1, moment_var);  
};
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET THE DATE
//---------------------------------------------------------  
const date_obj_arr = [
  { key: `Current ${type_name}`, value: `current_${type_value}` },
  { key: `Last ${type_name}`, value: `last_${type_value}` },
  { key: `Next ${type_name}`, value: `next_${type_value}` },
];
let date_obj = await tp.system.suggester(
  (item) => item.key,
  date_obj_arr,
  false,
  `Which ${type_name}?`
);
const date_value = date_obj.value;

let full_date = ``;
if (date_value.startsWith(`current`)) {  
  full_date = moment();  
} else if (date_value.startsWith(`next`)) {  
  full_date = moment().add(1, moment_var);  
} else {  
  full_date = moment().subtract(1, moment_var);  
};
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_00_week]]
2. [[33_00_month]]
3. [[34_00_quarter]]
4. [[35_00_year]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[30_02_cal_type_and_file_class|Calendar Type and File Class]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[32_02_cal_week_titles_alias_and_file_name|Weekly Calendar Titles, Alias, and File Name]]
2. [[33_02_cal_month_titles_alias_and_file_name|Monthly Calendar Titles, Alias, and File Name]]

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE 
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.definition AS Definition
WHERE 
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
