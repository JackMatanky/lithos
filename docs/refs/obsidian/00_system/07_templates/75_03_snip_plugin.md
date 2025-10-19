<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const sys_plugins_dir = "00_system/03_plugins/";
const pkm_dir = "70_pkm/";
const autohotkey_dir = "70_pkm/autohotkey/";
const google_apps_script_dir = "70_pkm/google_apps_script/";
const google_sheets_dir = "70_pkm/google_sheets/";
const javascript_dir = "70_pkm/javascript/";
const latex_dir = "70_pkm/latex/";
const ms_excel_dir = "70_pkm/ms_excel/";
const python_dir = "70_pkm/python/";
const sql_dir = "70_pkm/sql/";
const tableau_dir = "70_pkm/tableau/";

//-------------------------------------------------------------------
// SET THE FILE'S CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt("Title", "", true, false);
} else {
  title = tp.file.title;
};
title = title.trim();
const short_title = title.toLowerCase();

//-------------------------------------------------------------------
// SET PROGRAMMING LANGUAGE AND FILE CLASS
//-------------------------------------------------------------------
const file_class = "pkm_code_snippet";
// Filter array to only include folder paths in the tools directory
const language_obj_arr = [
  {
    name: "JavaScript",
    value: "javascript",
    tag: "js",
    dir: javascript_dir
  },
  {
    name: "Python",
    value: "python",
    tag: "py",
    dir: python_dir
  },
  {
    name: "SQL",
    value: "sql",
    tag: "sql",
    dir: sql_dir
  },
  {
    name: "AutoHotkey",
    value: "autohotkey",
    tag: "ahk",
    dir: autohotkey_dir
  },
  {
    name: "Google Apps Script",
    value: "google_apps_script",
    tag: "gas",
    dir: google_apps_script_dir
  },
  {
    name: "Google Sheets",
    value: "google_sheets",
    tag: "g_sheets",
    dir: google_sheets_dir
  },
  {
    name: "Microsoft Excel",
    value: "microsoft_excel",
    tag: "ms_excel",
    dir: ms_excel_dir
  },
  {
    name: "Tableau",
    value: "tableau",
    tag: "tableau",
    dir: tableau_dir
  },
];

// Choose a programming language
let language_obj = await tp.system.suggester(
  (item) => item.name,
  language_obj_arr,
  false,
  "Programming Language?"
);

const language_name = language_obj.name;
const language_value = language_obj.value;
const language_tag = language_obj.tag;
const language_dir = language_obj.dir;

const alias = `${language} ${short_title}`;

//-------------------------------------------------------------------
// SET MODULE
//-------------------------------------------------------------------
const module = await tp.system.prompt("What is the module?");

/* ---------------------------------------------------------- */
/*                       SET DESCRIPTION                      */
/* ---------------------------------------------------------- */
const description = await tp.system.prompt("Describe the snippet?");

//-------------------------------------------------------------------
// SET INPUT TYPE
//-------------------------------------------------------------------
const input = await tp.system.prompt("What are the snippet's input values?");

//-------------------------------------------------------------------
// SET OUTPUT TYPE
//-------------------------------------------------------------------
const output = await tp.system.prompt("What are the snippet's output, or return, values?");

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
// Get the current folder path
const folder_path = tp.file.folder(true);

// Get all the vault's folder paths
const all_directory_paths = app.vault.getAllLoadedFiles().filter(i => i.children).map(folder => folder.path);

tR += "---";
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
pillar: <%* tR += pillar_value_link %>
category: <%* tR += category_value_link %>
branch: <%* tR += branch_value_link %>
field: <%* tR += field_value_link %>
subject: <%* tR += subject_value_link %>
topic: <%* tR += topic_value_link %>
subtopic: <%* tR += subtopic_value_link %>
library:
plugin:
language: <%* tR += language_value %>
module: <%* tR += module %>
cssclasses:
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title %>

## Description

> [!snippet] Snippet Details
>
> Plugin:
> Language: <%* tR += language_name %>
> Input:: <%* tR += input %>
> Output:: <%* tR += output %>
> Description:: <%* tR += description %>

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```<%* tR += language_name %>

```

### Plugin

<!-- Add the full code excluding explanatory comments  -->

```<%* tR += language_name %>

```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```<%* tR += language_name %>

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	file.frontmatter.about AS Description,
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
