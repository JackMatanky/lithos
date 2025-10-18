<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const cal_dir = "10_calendar/";
const cal_day_dir = "10_calendar/11_days/";
const cal_week_dir = "10_calendar/12_weeks/";
const cal_month_dir = "10_calendar/13_months/";
const cal_quarter_dir = "10_calendar/14_quarters/";
const cal_year_dir = "10_calendar/15_years/";
const monthly_reflection_dir = "80_insight/95_reflection/03_monthly/";

/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const hr_line = hyphen.repeat(3);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const colon = String.fromCodePoint(0x3a);
const two_percent = String.fromCodePoint(0x25).repeat(2);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

//Text Formatting
const head_lvl = (int) => `${hash.repeat(int)}${space}`;
const yaml_li = (value) => `${new_line}${ul_yaml}"${value}"`;
const link_alias = (file, alias) => ["[[" + file, alias + "]]"].join("|");
const link_tbl_alias = (file, alias) => ["[[" + file, alias + "]]"].join("\\|");
const cmnt_ob_start = `${two_percent}${space}`;
const cmnt_ob_end = `${space}${two_percent}`;
const cmnt_html_start = `${less_than}${excl}${two_hyphen}${space}`;
const cmnt_html_end = `${space}${two_hyphen}${great_than}`;
const tbl_start = `${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end = `${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${hyphen}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${great_than}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_start}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;
const dv_colon = `${colon.repeat(2)}${space}`;

//-------------------------------------------------------------------
// FORMATTING FUNCTIONS
//-------------------------------------------------------------------
const snake_case_fmt = (name) =>
  name.replaceAll(/(\-\s\-)|(\s)|(\-)]/g, "_").toLowerCase();

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format(`YYYY-MM-DD[T]HH:mm`);
const date_modified = moment().format(`YYYY-MM-DD[T]HH:mm`);

//-------------------------------------------------------------------
// DATE TYPE, MOMENT VARIABLE, AND FILE CLASS
//-------------------------------------------------------------------
const type_name = `Month`;
const type_value = type_name.toLowerCase();
const moment_var = `${type}s`;
const file_class = `cal_${type}`;

//-------------------------------------------------------------------
// SET THE DATE
//-------------------------------------------------------------------
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

if (month_date_value.startsWith(`current`)) {
  full_date = moment();
} else if (month_date_value.startsWith(`next`)) {
  full_date = moment().add(1, moment_var);
} else {
  full_date = moment().subtract(1, moment_var);
};

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const long_date = moment(full_date).format(`MMMM YYYY`);
const med_date = moment(full_date).format(`MMM [']YY`);
const short_date = moment(full_date).format(`YYYY-MM`);
const year_full = moment(full_date).format(`YYYY`);
const year_short = moment(full_date).format(`YY`);
const quarter = moment(full_date).format(`Q`);
const month_full_name = moment(full_date).format(`MMMM`);
const month_number = moment(full_date).format(`MM`);
const date_start = moment(full_date)
  .startOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const date_end = moment(full_date)
  .endOf(type_value)
  .format(`YYYY-MM-DD[T]HH:mm`);
const prev_date = moment(full_date)
  .subtract(1, moment_var)
  .format(`MMM [']YY`);
const next_date = moment(full_date)
  .add(1, moment_var)
  .format(`MMM [']YY`);

//-------------------------------------------------------------------
// MONTHLY CALENDAR TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${long_date}`;
const short_title_name = `${med_date}`;
const short_title_value = `${short_date}`;

const alias_arr = `${new_line}${ul_yaml}${month_name}${ul_yaml}${full_title_name}${new_line}${ul_yaml}"${short_title_name}"${ul_yaml}${short_title_value}`;

const file_name = `${short_date}`;

//-------------------------------------------------------------------
// CALENDAR FILE LINKS AND ALIASES
//-------------------------------------------------------------------
const year_file = `${year_full}`;
const quarter_file = `${year_full}-Q${quarter}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = cal_month_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
   await tp.file.move(`${directory}${file_name}`);
};

tR += "---"
%>
title: <%* tR += file_name %>
aliases: <%* tR += file_alias %>
date_start: <%* tR += date_start %>
date_end: <%* tR += date_end %>
year: <%* tR += year_full %>
quarter: <%* tR += quarter %>
month_name: <%* tR += month_full_name %>
month_number: <%* tR += month_number %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
---

# <%* tR += full_title_name %>

> [!<%* tR += type_value %> ] <%* tR += type_name %> Context
> 
> |            Year            |            Quarter            |
> |:--------------------------:|:-----------------------------:|
> | [[<%* tR += year_file %>]] | [[<%* tR += quarter_file %>]] |

<< [[<%* tR += prev_date %>]] | [[<%* tR += next_date %>]] >>

---

## Journals

- [ ] Monthly Reflection
	- [[<%* tR += monthly_reflection_dir %><%* tR += title %>_monthly_reflection |Monthly Reflection]]

```dataview
TABLE WITHOUT ID
	file.frontmatter.title AS Title,
	file.frontmatter.type AS Type,
	file.frontmatter.date AS Date,
	file.link AS Link
FROM "80_insight"
WHERE 
	contains(file.frontmatter.file_class, "journal")
	AND ((date(file.frontmatter.date_created) >= date(<%* tR += date_start %>))
	OR (file.cday >= (<%* tR += date_start %>)))
	AND ((date(file.frontmatter.date_created) <= date(<%* tR += date_end %>))
	OR (file.cday <= (<%* tR += date_end %>)))
SORT file.frontmatter.date
```

### Failures

```dataview
LIST
	rows.failure
FROM "80_insight"
FLATTEN failure
WHERE 
	contains(file.frontmatter.file_class, "journal")
	AND ((date(file.frontmatter.date_created) >= date(<%* tR += date_start %>))
	OR (file.cday >= (<%* tR += date_start %>)))
	AND ((date(file.frontmatter.date_created) <= date(<%* tR += date_end %>))
	OR (file.cday <= (<%* tR += date_end %>)))
GROUP BY file.frontmatter.date
SORT file.frontmatter.date
```

### Achievements

```dataview
LIST
	rows.achievement
FROM "80_insight"
FLATTEN achievement
WHERE 
	contains(file.frontmatter.file_class, "journal")
	AND contains(achievement, " ")
	AND (date(file.frontmatter.date_created) >= date(<%* tR += date_start %>)
	OR file.cday >= (<%* tR += date_start %>))
	AND (date(file.frontmatter.date_created) <= date(<%* tR += date_end %>)
	OR file.cday <= (<%* tR += date_end %>))
GROUP BY file.frontmatter.date
SORT file.frontmatter.date
```

---

## Knowledge Management

### Zettelkasten

#### Literature

```dataview
TABLE WITHOUT ID
	file.frontmatter.title AS Title,
	file.frontmatter.subtype AS Subtype,
	file.tags AS Tags,
	file.link AS Link
FROM -"00_system/05_templates"
WHERE 
	file.frontmatter.file_class = "pkm_note"
	AND file.frontmatter.type = "literature"
	AND (date(file.frontmatter.date_created) >= date(<%* tR += date_start %>)
	OR file.cday >= (<%* tR += date_start %>))
	AND (date(file.frontmatter.date_created) <= date(<%* tR += date_end %>)
	OR file.cday <= (<%* tR += date_end %>))
SORT file.frontmatter.file_class
```

#### Fleeting

```dataview
TABLE WITHOUT ID
	file.frontmatter.title AS Title,
	file.frontmatter.subtype AS Subtype,
	file.tags AS Tags,
	file.link AS Link
FROM -"00_system/05_templates"
WHERE 
	file.frontmatter.file_class = "pkm_note"
	AND file.frontmatter.type = "fleeting"
	AND (date(file.frontmatter.date_created) >= date(<%* tR += date_start %>)
	OR file.cday >= (<%* tR += date_start %>))
	AND (date(file.frontmatter.date_created) <= date(<%* tR += date_end %>)
	OR file.cday <= (<%* tR += date_end %>))
SORT file.frontmatter.file_class
```

### Library

#### Created This Month

```dataview
TABLE WITHOUT ID
	file.frontmatter.title AS Title,
	regexreplace(regexreplace(file.frontmatter.file_class, "pkm", ""), "_", " ") AS "File Class",
	file.tags AS Tags,
	file.link AS Link
FROM -"00_system/05_templates"
WHERE 
	file.frontmatter.file_class != "pkm_note"
	AND contains(file.frontmatter.file_class, "pkm")
	AND (date(file.frontmatter.date_created) >= date(<%* tR += date_start %>)
	OR file.cday >= (<%* tR += date_start %>))
	AND (date(file.frontmatter.date_created) <= date(<%* tR += date_end %>)
	OR file.cday <= (<%* tR += date_end %>))
SORT file.frontmatter.file_class
LIMIT 50
```

#### Modified This Month

```dataview
TABLE WITHOUT ID
	file.frontmatter.title AS Title,
	regexreplace(regexreplace(file.frontmatter.file_class, "pkm", ""), "_", " ") AS "File Class",
	file.tags AS Tags,
	file.link AS Link
FROM -"00_system/05_templates"
WHERE 
	file.frontmatter.file_class != "pkm_note"
	AND contains(file.frontmatter.file_class, "pkm")
	AND (date(file.frontmatter.date_modified) >= date(<%* tR += date_start %>)
	OR file.mday >= date(<%* tR += date_start %>))
	AND (date(file.frontmatter.date_modified) <= date(<%* tR += date_end %>)
	OR file.mday <= date(<%* tR += date_end %>))
SORT file.frontmatter.file_class
LIMIT 50
```

## Tasks and Events

### Due This Month

```tasks
not done
due after <%* tR += date_start %>
due before <%* tR += date_end %>
path does not include 00_system/05_templates
```

### Completed This Month

```dataview
TABLE WITHOUT ID 
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)", "") AS Task,
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "^[A-Za-z0-9\'\-\s]*_", "") AS Type,
	T.completion AS Completed,
	T.time_start AS Start,
	T.time_end AS End,
	T.duration_est AS Estimate,
	(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) - 
	date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)) AS Duration,
	T.section AS Link
FROM -"00_system/05_templates" AND #task
FLATTEN file.tasks AS T
WHERE any(file.tasks, (t) => t.completion >= date(<%* tR += date_start %>) 
	AND t.completion <= date(<%* tR += date_end %>))
SORT T.completion, T.time_start ASC
```
