---
title: 30_03_cal_date_variables
aliases:
  - Calendar Date Variables
  - Date Variables for Calendars
  - cal_date_variables
  - date_cal_variables
plugin: templater
language:
  - javascript
module:
  - momentjs
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-04T17:27
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, js/momentjs
---
# Calendar Date Variables

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: All date variables used in calendar files.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// GENERAL DATE VARIABLES
//---------------------------------------------------------
const year_long = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const year_day = moment(full_date).format(`DDDD`);
const quarter_num = moment(full_date).format(`Q`);
const quarter_ord = moment(full_date).format(`Qo`);
const month_name_full = moment(full_date).format(`MMMM`);
const month_name_short = moment(full_date).format(`MMM`);
const month_num_long = moment(full_date).format(`MM`);
const month_num_short = moment(full_date).format(`M`);
const month_day_long = moment(full_date).format(`DD`);
const month_day_short = moment(full_date).format(`D`);
const week_number = moment(full_date).format(`ww`);
const weekday_name = moment(full_date).format(`dddd`);
const weekday_num = moment(full_date).format(`[0]e`);

//---------------------------------------------------------
// DAILY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_day = `${month_full_name} ${month_day_short}, ${year_long}`;
const med_date_day = `${year_long}-${month_num_long}-${month_day_long}`;
const short_date_day = `${year_short}-${month_num_short}-${month_day_short}`;
const prev_day = moment(full_date)
  .subtract(1, `days`)
  .format(`YYYY-MM-DD`);
const next_day = moment(full_date)
  .add(1, `days`)
  .format(`YYYY-MM-DD`);

//---------------------------------------------------------
// WEEKLY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_week = `Week #${week_number} ${year_long}`;
const short_date_week = `${year_long} W${week_number}`;
const week_start = moment(full_date)
  .startOf(`week`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const week_end = moment(full_date)
  .endOf(`week`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_week = moment(full_date)
  .subtract(1, `weeks`)
  .format(`YYYY-[W]ww`);
const next_week = moment(full_date)
  .add(1, `weeks`)
  .format(`YYYY-[W]ww`);

// Weekday variable
const sunday = moment(full_date).day(0).format(`YYYY-MM-DD`);
const monday = moment(full_date).day(1).format(`YYYY-MM-DD`);
const tuesday = moment(full_date).day(2).format(`YYYY-MM-DD`);
const wednesday = moment(full_date).day(3).format(`YYYY-MM-DD`);
const thursday = moment(full_date).day(4).format(`YYYY-MM-DD`);
const friday = moment(full_date).day(5).format(`YYYY-MM-DD`);
const saturday = moment(full_date).day(6).format(`YYYY-MM-DD`);

//---------------------------------------------------------
// MONTHLY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_month = `${month_full_name} ${year_long}`;
const med_date_month = `${month_name_short} '${year_short}`;
const short_date_month = `${year_long}-${month_num_short}`;
const month_start = moment(full_date)
  .startOf(`month`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const month_end = moment(full_date)
  .endOf(`month`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_month = moment(full_date)
  .subtract(1, `months`)
  .format(`MMM [']YY`);
const next_month = moment(full_date)
  .add(1, `months`)
  .format(`MMM [']YY`);

//---------------------------------------------------------
// QUARTERLY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_quarter = `${quarter_ord} Quarter of ${year_long}`;
const med_date_quarter = `Q${quarter_num} '${year_long}`;
const short_date_quarter = `Q${quarter_num} '${year_short}`;
const quarter_start = moment(full_date)
  .startOf(`quarter`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const quarter_end = moment(full_date)
  .endOf(`quarter`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_quarter = moment(full_date)
  .subtract(1, `quarters`)
  .format(`[Q]Q [']YY`);
const next_quarter = moment(full_date)
  .add(1, `quarters`)
  .format(`[Q]Q [']YY`);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// GENERAL DATE VARIABLES
//---------------------------------------------------------
const year_long = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const year_day = moment(full_date).format(`DDDD`);
const quarter_num = moment(full_date).format(`Q`);
const quarter_ord = moment(full_date).format(`Qo`);
const month_name_full = moment(full_date).format(`MMMM`);
const month_name_short = moment(full_date).format(`MMM`);
const month_num_long = moment(full_date).format(`MM`);
const month_num_short = moment(full_date).format(`M`);
const month_day_long = moment(full_date).format(`DD`);
const month_day_short = moment(full_date).format(`D`);
const week_number = moment(full_date).format(`ww`);
const weekday_name = moment(full_date).format(`dddd`);
const weekday_num = moment(full_date).format(`[0]e`);

//---------------------------------------------------------
// DAILY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_day = `${month_full_name} ${month_day_short}, ${year_long}`;
const med_date_day = `${year_long}-${month_num_long}-${month_day_long}`;
const short_date_day = `${year_short}-${month_num_short}-${month_day_short}`;
const prev_day = moment(full_date)
  .subtract(1, `days`)
  .format(`YYYY-MM-DD`);
const next_day = moment(full_date)
  .add(1, `days`)
  .format(`YYYY-MM-DD`);

//---------------------------------------------------------
// WEEKLY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_week = `Week #${week_number} ${year_long}`;
const short_date_week = `${year_long} W${week_number}`;
const week_start = moment(full_date)
  .startOf(`week`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const week_end = moment(full_date)
  .endOf(`week`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_week = moment(full_date)
  .subtract(1, `weeks`)
  .format(`YYYY-[W]ww`);
const next_week = moment(full_date)
  .add(1, `weeks`)
  .format(`YYYY-[W]ww`);

// Weekday variable
const sunday = moment(full_date).day(0).format(`YYYY-MM-DD`);
const monday = moment(full_date).day(1).format(`YYYY-MM-DD`);
const tuesday = moment(full_date).day(2).format(`YYYY-MM-DD`);
const wednesday = moment(full_date).day(3).format(`YYYY-MM-DD`);
const thursday = moment(full_date).day(4).format(`YYYY-MM-DD`);
const friday = moment(full_date).day(5).format(`YYYY-MM-DD`);
const saturday = moment(full_date).day(6).format(`YYYY-MM-DD`);

//---------------------------------------------------------
// MONTHLY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_month = `${month_full_name} ${year_long}`;
const med_date_month = `${month_name_short} '${year_short}`;
const short_date_month = `${year_long}-${month_num_short}`;
const month_start = moment(full_date)
  .startOf(`month`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const month_end = moment(full_date)
  .endOf(`month`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_month = moment(full_date)
  .subtract(1, `months`)
  .format(`MMM [']YY`);
const next_month = moment(full_date)
  .add(1, `months`)
  .format(`MMM [']YY`);

//---------------------------------------------------------
// QUARTERLY CALENDAR DATE VARIABLES
//---------------------------------------------------------
const long_date_quarter = `${quarter_ord} Quarter of ${year_long}`;
const med_date_quarter = `Q${quarter_num} '${year_long}`;
const short_date_quarter = `Q${quarter_num} '${year_short}`;
const quarter_start = moment(full_date)
  .startOf(`quarter`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const quarter_end = moment(full_date)
  .endOf(`quarter`)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_quarter = moment(full_date)
  .subtract(1, `quarters`)
  .format(`[Q]Q [']YY`);
const next_quarter = moment(full_date)
  .add(1, `quarters`)
  .format(`[Q]Q [']YY`);
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[31_00_day]]
2. [[31_01_day_periodic]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[30_02_cal_type_and_file_class|Daily Calendar Type and File Class]]
2. [[nl_date|Natural Language Date Suggester]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
2. [[32_01_cal_week_date_variables|Weekly Calendar Date Variables]]
3. [[33_01_cal_month_date_variables|Monthly Calendar Date Variables]]
4. [[34_01_cal_quarter_date_variables|Quarterly Calendar Date Variables]]

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
