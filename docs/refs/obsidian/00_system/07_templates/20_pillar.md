<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format(`YYYY-MM-DD[T]HH:mm`);
const date_modified = moment().format(`YYYY-MM-DD[T]HH:mm`);

//-------------------------------------------------------------------
// SET THE FILE CLASS
//-------------------------------------------------------------------
const file_class = "pillar"

//-------------------------------------------------------------------
// SET THE FILE'S TITLE
//-------------------------------------------------------------------
// Check if note already has title
const has_title = !tp.file.title.startsWith("Untitled");
let title;

// If note does not have title,
// prompt for title and rename file
if (!has_title) {
  title = await tp.system.prompt(
    "Pillar?",
    "",
    true,
    false
);
} else {
  title = tp.file.title;
};

//-------------------------------------------------------------------
// SET STATUS
//-------------------------------------------------------------------
const status_arr = ["active", "inactive", "on_hold"];
const status = await tp.system.suggester(
  status_arr,
  status_arr,
  false,
  "Status?"
);

//-------------------------------------------------------------------
// SET STATUS
//-------------------------------------------------------------------
const type_arr = ["growth", "interpersonal", "personal", "professional"];
const type = await tp.system.suggester(
  type_arr,
  type_arr,
  false,
  "Type?"
);

//-------------------------------------------------------------------
// MOVE TO DAY CALENDAR DIRECTORY
//-------------------------------------------------------------------
const folder = tp.file.folder(true) + "/";

if (folder!= pillars_dir) {
  await tp.file.move(pillars_dir + title);
};

tR += "---";
%>
title: "<%* tR += title %>"
aliases: []
status: <%* tR += status %>
type: <%* tR += type %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
---

# <%* tR += Title %>

---

## Knowledge

```dataview
LIST
	rows.file.link
FROM -"00_system/05_templates"
WHERE
	contains(file.frontmatter.pillar, "<%* tR += title %>")
	AND contains(file.frontmatter.file_class, "pkm")
GROUP BY file.frontmatter.file_class
```

## Journal Entries

```dataview
LIST
FROM -"00_system/05_templates"
WHERE
	contains(file.frontmatter.pillar, "<%* tR += title %>")
	AND contains(file.frontmatter.file_class, "journal")
```

## Tasks and Events

```dataview
TABLE WITHOUT ID
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)", "") AS Task,
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "^[A-Za-z0-9\'\-\s]*_", "") AS Type,
	T.completion AS Completed,
	T.time_start AS Start,
	T.time_end AS End,
	(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) -
	date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)) AS Duration,
	T.section AS Link
FROM -"00_system/05_templates" AND #task
FLATTEN file.tasks AS T
WHERE any(file.frontmatter.pillar = "<%* tR += title %>")
SORT T.time_start ASC
```
