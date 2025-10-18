---
title: 32_03_week_day_files
aliases:
  - Weekly Calendar Day Files
  - Calendar Week Day Files
  - Week Calendar Day Files
  - Week Day Files
  - week day files
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
tags: obsidian/templater, javascript, js/momentjs, obsidian/tp/file/include
---
# Weekly Calendar Day Files

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a week's calendar day files.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const week_days = "31_00_days_of_week";

//---------------------------------------------------------
// WEEKDAY CALENDAR FILES
//---------------------------------------------------------
// Retrieve the Weekday Calendar Files template and content
template = await tp.file.find_tfile(week_days);
content = await tp.file.include(template);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// WEEKDAY CALENDAR FILES
//---------------------------------------------------------
template = await tp.file.find_tfile(week_days);
content = await tp.file.include(template);
```

#### Referenced Template

```javascript
//---------------------------------------------------------  
// FOLDER PATH VARIABLES
//---------------------------------------------------------  
const sys_temp_include_dir = "00_system/06_template_include/";
const cal_dir = "10_calendar";  
const cal_day_dir = "10_calendar/11_days";  
const cal_week_dir = "10_calendar/12_weeks";  
const cal_month_dir = "10_calendar/13_months";  
const cal_quarter_dir = "10_calendar/14_quarters";  
const cal_year_dir = "10_calendar/15_years";

//---------------------------------------------------------  
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//---------------------------------------------------------   
const buttons_table_pdev_today = "00_90_buttons_table_pdev_today";  
const buttons_table_note = "00_80_buttons_table_notes";  
const buttons_table_task_habit_today = "00_40_buttons_table_task_habit_today";

//---------------------------------------------------------  
// BUTTONS TABLES
//---------------------------------------------------------  
template = await tp.file.find_tfile(buttons_table_pdev_today);  
content = await tp.file.include(template);  
include_arr = content.toString();  
const journal_buttons = include_arr;

template = await tp.file.find_tfile(buttons_table_note);  
content = await tp.file.include(template);  
include_arr = content.toString();  
const note_buttons = include_arr;

template = await tp.file.find_tfile(buttons_table_task_habit_today);  
content = await tp.file.include(template);  
include_arr = content.toString();  
const task_habit_buttons = include_arr;

//---------------------------------------------------------  
// FILE CREATION AND MODIFIED DATE
//---------------------------------------------------------  
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");  
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//---------------------------------------------------------  
// SET THE WEEK AND NUMBER
//---------------------------------------------------------  
const date_obj_arr = [  
  { key: "Current Week", value: "current" },  
  { key: "Last Week", value: "last" },  
  { key: "Next Week", value: "next" },  
];  
let date_obj = await tp.system.suggester(  
  (item) => item.key,  
  date_obj_arr,  
  false,  
  "Which Week?"  
);

const date_value = date_obj.value;

let full_date = "";

if (date_value.startsWith("current")) {  
  full_date = moment();  
} else if (date_value.startsWith("next")) {  
  full_date = moment().add(1, "week");  
} else {  
  full_date = moment().subtract(1, "week");  
}

const week_number = moment(full_date).format("ww");

//---------------------------------------------------------  
// WEEKDAY CALENDAR VARIABLE
//---------------------------------------------------------  
const sunday = moment(full_date).day(0).format("YYYY-MM-DD");  
const monday = moment(full_date).day(1).format("YYYY-MM-DD");  
const tuesday = moment(full_date).day(2).format("YYYY-MM-DD");  
const wednesday = moment(full_date).day(3).format("YYYY-MM-DD");  
const thursday = moment(full_date).day(4).format("YYYY-MM-DD");  
const friday = moment(full_date).day(5).format("YYYY-MM-DD");  
const saturday = moment(full_date).day(6).format("YYYY-MM-DD");

//---------------------------------------------------------  
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//---------------------------------------------------------  
const type_name = "Day";  
const type_value = type_name.toLowerCase();  
const moment_var = `${type_value}s`;  
const file_class = `cal_${type_value}`;

//---------------------------------------------------------  
// WEEKDAY CALENDAR FILES
//---------------------------------------------------------  
// FRONTMATTER VARIABLES
let fmatter_title;  
let fmatter_alias;  
let fmatter_date;  
let fmatter_year;  
let fmatter_year_day;  
let fmatter_quarter;  
let fmatter_month_name;  
let fmatter_month_number;  
let fmatter_month_day;  
let fmatter_week_number = `week_number: ${week_number}`;  
let fmatter_weekday_name;  
let fmatter_weekday_number;  
let fmatter_metatable = `metatable: true`  
let fmatter_cssclasses = `cssclasses: [\read_view_zoom, \read_wide_margin, \inline_title_hide]`;  
let fmatter_type = `type: ${type_value}`;  
let fmatter_file_class = `file_class: ${file_class}`;  
let fmatter_date_created = `date_created: ${date_created}`;  
let fmatter_date_modified = `date_modified: ${date_modified}`;

// DATES
let date;  
let long_date;  
let short_date;  
let year_long;  
let year_short;  
let year_day;  
let quarter_num;  
let quarter_ord;  
let month_name_full;  
let month_name_short;  
let month_num_long;  
let month_num_short;  
let month_day_long;  
let month_day_short;  
let weekday_name;  
let weekday_number;  
let prev_date;  
let next_date;

// TITLES AND ALIAS
let full_title_name;  
let short_title_name;  
let full_title_value;  
let short_title_value;  
let alias_arr;

// FILE CREATION VARIABLES
let file_name;  
let file_content;  
const directory = cal_day_dir;

const percent = String.fromCodePoint(37);  
const comment = percent.repeat(2);  
const backtick = String.fromCodePoint(0x60);  
const three_backtick = backtick.repeat(3);
const space = String.fromCodePoint(0x20);  
const two_space = space.repeat(2);  
const call_start = `>${space}`;
const tbl_pipe =`${space}|${space}`;  
const tbl_start =`|${space}`;  
const call_tbl_start = `${call_start}${tbl_start}`;
const tbl_end =`${space}|`;
const call_tbl_end = `${tbl_end}${two_space}`;

// WEEKDAY DATES ARRAY
const weekday_arr = [  
  sunday,  
  monday,  
  tuesday,  
  wednesday,  
  thursday,  
  friday,  
  saturday,  
];

// LOOP THROUGH WEEKDAY DATES ARRAY
for (let i = 0; i < weekday_arr.length; i++) {  
  file_name = weekday_arr[i];  
  full_date = moment(weekday_arr[i]);

  // DATE VARIABLES
  date = moment(full_date).format("YYYY-MM-DD");  
  long_date = moment(full_date).format("MMMM D, YYYY");  
  short_date = moment(full_date).format("YY-M-D");  
  year_long = moment(full_date).format("YYYY");  
  year_short = moment(full_date).format("YY");  
  year_day = moment(full_date).format("DDDD");  
  quarter_num = moment(full_date).format("Q");  
  quarter_ord = moment(full_date).format("Qo");  
  month_name_full = moment(full_date).format("MMMM");  
  month_name_short = moment(full_date).format("MMM");  
  month_num_long = moment(full_date).format("MM");  
  month_num_short = moment(full_date).format("M");  
  month_day_long = moment(full_date).format("DD");  
  month_day_short = moment(full_date).format("D");  
  weekday_name = moment(full_date).format("dddd");  
  weekday_number = moment(full_date).format("[0]e");  
  prev_date = moment(full_date).subtract(1, moment_var).format("YYYY-MM-DD");  
  next_date = moment(full_date).add(1, moment_var).format("YYYY-MM-DD");

  // DAILY CALENDAR TITLES, ALIAS, AND FILE NAME
  full_title_name = `${weekday_name}, ${long_date}`;  
  short_title_name = `${long_date}`;  
  full_title_value = `${date}`;  
  short_title_value = `${short_date}`;

  alias_arr = `${new_line}${ul_yaml}"${full_title_name}"${ul_yaml}"${short_title_name}"${new_line}${ul_yaml}${full_title_value}${ul_yaml}${short_title_value}`;

  // CALENDAR FILE LINKS AND ALIASES
  year_file = `[[${year_long}]]`;  
  quarter_file = `[[${year_long}-Q${quarter_num}]]`;  
  month_file = `[[${year_long}-${month_num_long}\|${month_name_short} '${year_short}]]`;  
  week_file = `[[${year_long}-W${week_number}]]`;

  // DAY CONTEXT CALLOUT
  callout = `${call_start}[!${type_value}]${space}${type_name}${space}Context
${call_start}
${call_tbl_start}Year${tbl_pipe}Quarter${tbl_pipe}Month${tbl_pipe}Week${call_tbl_end}
${call_tbl_start}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${call_tbl_end}
${call_tbl_start}${year_file}${tbl_pipe}${quarter_file}${tbl_pipe}${month_file}${tbl_pipe}${week_file}${call_tbl_end}`;

  prev_next_date = `<< [[${prev_date}]] | [[${next_date}]] >>`;
  
  const toc_title = `${call_start}[!toc]${space}${type_name}${space}[[${file_name}#${full_title_name}\|Contents]]
${call_start}\n`;
  const toc_section = `${call_tbl_start}[[${file_name}#Journal Entries\|Journal Entries]]${tbl_pipe}[[${file_name}#Notes\|Notes]]${tbl_pipe}[[${file_name}#Library\|Library]]${tbl_pipe}[[${file_name}#Tasks and Events\|Tasks and Events]]${call_tbl_end}\n`;
  const toc_divide = `${call_tbl_start}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${tbl_pipe}:---------:${call_tbl_end}`

  const toc = `${toc_title}${toc_section}${toc_divide}`;

  // DATAVIEW JOURNAL LIST
  const journal_list = await tp.user.dv_pdev_date(date, "false");

  // DAILY TASKS AND EVENTS DATAVIEW TABLES
  // STATUS OPTIONS: 'due', 'done', 'new'
  const tasks_due = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "due",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const tasks_done = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "done",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const tasks_created = await tp.user.dv_task_type_status_dates({
    type: "child_task",
    status: "new",
    start_date: date,
    end_date: "",
    md: "false",
  });
 
  // DAILY PKM FILES DATAVIEW TABLE
  // TYPES: "pkm_tree", "permanent", "literature", "fleeting", "info"
  // STATUSES: "schedule", "review", "clarify", "develop", "done", "resource"
  const pkm_tree = await tp.user.dv_pkm_type_status_dates({
    type: "tree",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  
  const pkm_note_perm = await tp.user.dv_pkm_type_status_dates({
    type: "permanent",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });
  
  const pkm_note_lit = await tp.user.dv_pkm_type_status_dates({
    type: "literature",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });

  const pkm_note_fleet = await tp.user.dv_pkm_type_status_dates({
    type: "type",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });

  const pkm_note_info = await tp.user.dv_pkm_type_status_dates({
    type: "info",
    status: "",
    start_date: date,
    end_date: "",
    md: "false",
  });

  // DAILY LIBRARY DATAVIEW TABLE
  // STATUS OPTIONS: 'created', 'modified'
  const lib_completed = await tp.user.dv_lib_status_dates({
    status: "done",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const lib_created = await tp.user.dv_lib_status_dates({
    status: "new",
    start_date: date,
    end_date: "",
    md: "false",
  });
  const lib_modified = await tp.user.dv_lib_status_dates({
    status: "modified",
    start_date: date,
    end_date: "",
    md: "false",
  });

  fmatter_title = `title: ${file_name}`;  
  fmatter_alias = `aliases: ${alias_arr}`;  
  fmatter_date = `date: ${date}`;  
  fmatter_year = `year: ${year_long}`;  
  fmatter_year_day = `year_day: ${year_day}`;  
  fmatter_quarter = `quarter: ${quarter_num}`;  
  fmatter_month_name = `month_name: ${month_name_full}`;  
  fmatter_month_number = `month_number: ${month_num_long}`;  
  fmatter_month_day = `month_day: ${month_day_long}`;  
  fmatter_weekday_name = `weekday_name: ${weekday_name}`;  
  fmatter_weekday_number = `weekday_number: ${weekday_number}`;

  file_content = `---  
${fmatter_title}
${fmatter_alias}
${fmatter_date}
${fmatter_year}
${fmatter_year_day}
${fmatter_quarter}
${fmatter_month_name}
${fmatter_month_number}
${fmatter_month_day}
${fmatter_week_number}
${fmatter_weekday_name}
${fmatter_weekday_number}
${fmatter_metatable}
${fmatter_cssclasses}
${fmatter_type}
${fmatter_file_class}
${fmatter_date_created}
${fmatter_date_modified}
---\n
# ${full_title_name}\n
${callout}\n
${prev_next_date}\n
---\n
## Journal Entries\n
${toc}\n
${journal_buttons}\n
${journal_list}\n
---\n
## Notes\n
${toc}\n
${note_buttons}\n
### Knowledge Tree
${pkm_tree}\n
### Permanent\n
${pkm_note_perm}\n
### Literature\n
${pkm_note_lit}\n
### Fleeting\n
${pkm_note_fleet}\n
### General Info\n
${pkm_note_info}\n
---\n
## Library\n
${toc}\n
### Completed Today\n
${comment} Limit 50 ${comment}\n
${lib_completed}\n
### Modified Today\n
${comment} Limit 50 ${comment}\n
${lib_modified}\n
### Created Today\n
${comment} Limit 50 ${comment}\n
${lib_created}\n
---\n
## Tasks and Events\n
${toc}\n
${task_habit_buttons}\n
### Due Today\n
${tasks_due}\n
### Completed Today\n
${tasks_done}\n
### Created Today\n
${tasks_created}\n
---\n
`;

  await tp.file.create_new(  
    file_content,  
    file_name,  
    false,  
    app.vault.getAbstractFileByPath(directory)  
  );  
}
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[32_02_week_days|Weekly and Weekdays Calendar Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[31_00_days_of_week]]
2. [[30_01_cal_date_suggester|Calendar Date Suggester]]
3. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
4. [[31_02_cal_day_titles_alias_and_file_name|Daily Calendar Titles, Alias, and File Name]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[32_02_cal_week_titles_alias_and_file_name|Weekly Calendar Titles, Alias, and File Name]]
2. [[31_01_cal_day_date_variables|Daily Calendar Date Variables]]
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
