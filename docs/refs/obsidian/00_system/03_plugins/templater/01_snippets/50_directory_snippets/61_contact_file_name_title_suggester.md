---
title: 61_contact_file_name_title_suggester
aliases:
  - Contact File Name and Title Suggester
  - Contact File Name and Title
  - contact_file_name_and_title_suggester
  - suggester_contact_file_name_and_title
language:
  - javascript
plugin: templater
module:
  - system
  - user
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-01T10:01
date_modified: 2023-10-25T16:23
tags: javascript, obsidian/templater, obsidian/tp/system/suggester
---
# Contact File and Full Name Suggester

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output:: A basename of the chosen contact file
> Description:: Return an contact's file name and the contact's main title from `alias[0]`.

---

## Snippet

```javascript
// Template file to include
const contact_name_alias = "51_contact_name_alias";

//---------------------------------------------------------
// SET CONTACT FILE NAME AND TITLE
//---------------------------------------------------------
// Retrieve the Contact File Name and Alias template and content
temp_file_path = `${sys_temp_include_dir}${contact_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const contact_value = include_arr[0];
const contact_value_link = include_arr[1];
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET CONTACT FILE NAME, ALIAS, LINK, AND YAML LINK
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${contact_name_alias}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const contact_value = include_arr[0];
const contact_value_link = include_arr[1];
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const space = String.fromCodePoint(0x20);
const new_line = String.fromCodePoint(0xa);
const ul_yaml = `${space.repeat(2)}${String.fromCodePoint(0x2d)}${space}`;

//---------------------------------------------------------
// SET CONTACT FILE NAME, ALIAS, LINK, AND YAML LINK
//---------------------------------------------------------
// Contact Files Directory
const contacts_dir = "51_contacts/";
const type_name = "Contact";

// Files Directory
const directory = contacts_dir;

// Arrays of Objects
const bool_obj_arr = [
  { key: "Yes", value: "yes" },
  { key: "No", value: "no" },
];
const md_file_name_alias_obj_arr = await tp.user.md_file_name_alias(directory);

let file_obj_arr = [];
let file_filter = [];
for (let i = 0; i < 10; i++) {
  // File Suggester
  const md_file_name_alias_obj = await tp.system.suggester(
    (item) => item.key,
    md_file_name_alias_obj_arr.filter(
      (file) => !file_filter.includes(file.value)
    ),
    false,
    `${type_name}?`
  );
  file_basename = md_file_name_alias_obj.value;
  file_alias = md_file_name_alias_obj.key;

  if (file_basename == "_user_input") {
    dir_contact_names = await tp.user.dir_contact_names(tp);
    file_alias = dir_contact_names.full_name;
    file_basename = dir_contact_names.last_first_name
      .replaceAll(/,/g, "")
      .replaceAll(/[^\w]/g, "_")
      .toLowerCase();
  } else if (file_basename == "null") {
    file_obj = { key: file_alias, value: file_basename };
    file_obj_arr.push(file_obj);
    break;
  }
  file_obj = { key: file_alias, value: file_basename };
  file_obj_arr.push(file_obj);
  file_filter.push(file_basename);

  const bool_obj = await tp.system.suggester(
    (item) => item.key,
    bool_obj_arr,
    false,
    `Another ${type_name}?`
  );

  if (bool_obj.value == "no") {
    break;
  }
}

const value = file_obj_arr.map((file) => file.value).join(", ");
const name = file_obj_arr.map((file) => file.key).join(", ");
const link = file_obj_arr
  .map((file) => `[[${file.value}|${file.key}]]`)
  .join(", ");
const value_link = file_obj_arr
  .map((file) => `${new_line}${ul_yaml}"[[${file.value}|${file.key}]]"`)
  .join("");

tR += value;
tR += ";";
tR += name;
tR += ";";
tR += link;
tR += ";";
tR += value_link;
```

##### Previously Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// SET CONTACT FILE NAME AND ALIAS
//---------------------------------------------------------
// Contact Files Directory
const contacts_dir = "51_contacts/";

const contact_obj_arr = await tp.user.md_file_name_alias(contacts_dir);
const contact_obj = await tp.system.suggester(
  (item) => item.key,
  contact_obj_arr,
  false,
  'Contact?'
)

let contact_value = contacts_obj.value;
let contact_name = contacts_obj.key;

if (contact_value.includes(`_user_input`)) {
  const contact_names = await tp.user.dir_contact_names(tp);
  const full_name = contact_names.full_name;
  const last_first_name = contact_names.last_first_name;
  contact_name = full_name;
  contact_value = last_first_name.replaceAll(/,/g, "").replaceAll(/[^\w]/g, "_").toLowerCase();
};

tR += contact_value
tR += ","
tR += contact_name
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_00_project|General Project Template]]
2. [[50_01_project_parent_tasks|General Project with Parent Tasks Template]]
3. [[50_10_proj_personal|Personal Project Template]]
4. [[50_20_proj_habit_ritual|Habits and Rituals Project Template]]
5. [[50_21_proj_habit_ritual_month|Monthly Habits and Rituals Project Template]]
6. [[50_30_proj_education|Education Project Template]]
7. [[50_31_proj_ed_course(X)|Education Course Project Template]]
8. [[50_32_proj_ed_book|Education Book Project Template]]
9. [[50_40_proj_professional|Professional Project Template]]
10. [[50_50_proj_work|Work Project Template]]
11. [[51_00_parent_task|General Parent Task Template]]
12. [[52_00_task_event|General Tasks and Events Template]]
13. [[53_00_action_item|Action Item Template]]
14. [[54_00_meeting|Meeting Template]]
15. [[55_10_habit|Habit Task Template]]
16. [[55_31_daily_work_start_rit|Daily Workday Startup Ritual Task Template]]
17. [[55_32_today_work_start_rit|Daily Workday Startup Ritual Task Button Template]]
18. [[55_33_tomorrow_work_start_rit|Tomorrow Workday Startup Ritual Task Button Template]]
19. [[71_00_book|Book Template]]
20. [[72_journal|Journal Article Template]]
21. [[73_report|Report Article Template]]
22. [[75_webpage|Webpage Template]]
23. [[76_10_video_youtube|YouTube Video Template]]
24. [[78_course_OpenU|OpenU Course Template]]
25. [[78_course_OpenU 1]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[61_contact_name_alias]]
2. [[dir_contact_names|Contact Names Prompt]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[get_md_file_names|Markdown File Names for Suggester]]
2. [[62_organization_file_name_title_suggester|Organization File Name and Title Suggester]]
3. [[10_pillar_file_name_title_suggester|Pillar File Name and Title Suggester]]

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

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]
2. [[tp.file.include Templater Function|The Templater tp.file.include() Function]]

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition
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
