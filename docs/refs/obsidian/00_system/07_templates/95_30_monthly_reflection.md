<%*
//-------------------------------------------------------------------
// GLOBAL FOLDER PATH VARIABLES
//-------------------------------------------------------------------
const pillars_dir = "20_pillars/";
const insights_dir = "80_insight/";
const monthly_reflection_dir = "80_insight/95_reflection/03_monthly/";
const goals_dir = "30_goals/";
const projects_dir = "40_projects/";

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
// SET THE FILE'S CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// SET FILE CLASS, JOURNAL TYPE
//-------------------------------------------------------------------
const file_class = "pdev_journal";
const type_name = "Monthly Reflection";
const type = type_name.replaceAll(/\s/g, "_").toLowerCase();

//-------------------------------------------------------------------
// CHOOSE THE JOURNAL WRITING DATE
//-------------------------------------------------------------------
// Choose the date for the journal entry
const nl_date = await tp.user.nl_date(tp);
// const prev_date = moment(nl_date).subtract(1, "days").format("YYYY-MM-DD");

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
// Check if note already has title
const has_title = !tp.file.title.startsWith("Untitled");
let title;
let alias;

// If note does not have title,
// prompt for title and rename file
if (!has_title) {
  title = nl_date + " " + type_name;
  alias = nl_date + " " + type;
} else {
  title = tp.file.title;
  title = title.trim();
  alias = nl_date + " " + type;
}

//-------------------------------------------------------------------
// SET PILLAR
//-------------------------------------------------------------------
// Retrieve all files in the Pillars directory
const pillar_files = app.vault
  .getMarkdownFiles()
  .filter((file) => file.path.includes(pillars_dir));

// Find all active pillars
const mapped_file_promises = pillar_files.map(async (file) => {
  const file_cache = await app.metadataCache.getFileCache(file);
  // If the file has the frontmatter YAML key (status)
  // and the correct YAML value (active), include the file
  file.shouldInclude = file_cache?.frontmatter?.status === "active";
  return file;
});

// Wait for all files to be processed
// (have to wait because getting frontmatter is asynchronous)
const mapped_files = await Promise.all(mapped_file_promises);
// Filter out files that should not be included
const filtered_files = mapped_files.filter((file) => file.shouldInclude);
// Convert list of files into list of links
const active_pillars = filtered_files.map((file) => file.basename).sort();

// Create an array for active pillars
const pillars_arr = ["null"];
// Add the active pillars to the array
pillars_arr.push(active_pillars);
// Flatten the array from two dimensions to one
const pillars = pillars_arr.flat();

const pillar = await tp.system.suggester(
  pillars,
  pillars,
  false,
  "Pillar?"
);

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  "Goal?"
);

//-------------------------------------------------------------------
// SET RELATED PROJECT
//-------------------------------------------------------------------
// Filter array to only include project folder paths based on task context
const projects = await tp.user.folder_name({
  dir: projects_dir,
  index: 2,
});

// Choose a project
const project = await tp.system.suggester(
  projects,
  projects,
  false,
  "Is this journal entry related to a project?"
);

//-------------------------------------------------------------------
// SET RELATED PARENT TASK
//-------------------------------------------------------------------
let parent_task;
if (project!== "null") {
  // Filter array to only include parent task folder paths matching the chosen project
  const parent_tasks = await tp.user.folder_name({
    dir: project,
    index: 3,
  });

  // Choose a parent task
  parent_task = await tp.system.suggester(
    parent_tasks,
    parent_tasks,
    false,
    "Is this journal entry related to the project's parent tasks?"
);
} else {
  parent_task = "null";
};

//-------------------------------------------------------------------
// SET THE JOURNAL'S ALIAS AND MOVE TO JOURNALS DIRECTORY
//-------------------------------------------------------------------
const folder_path = tp.file.folder(true) + "/";

if (folder_path!= monthly_reflection_dir) {
   await tp.file.move(monthly_reflection_dir + alias);
};

tR += "---";
%>
title: "<%* tR += title %>"
uuid: <%* tR += await tp.user.uuid() %>
aliases:
  - "<%* tR += type_name %>"
  - "<%* tR += title %>"
  - "<%* tR += alias %>"
date: <%* tR += nl_date %>
pillar: <%* tR += pillar %>
goal: <%* tR += goal %>
project: <%* tR += project %>
parent_task: <%* tR += parent_task %>
subtype:
type: <%* tR += type %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += title %>

> [!<%* tR += type_value %> ] <%* tR += full_type_name %> Details
>
> - **Journal Type**:: <%* tR += type_name %>
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`
> - **Parent Task**: `dv: this.file.frontmatter.parent_task`
> - **Date**: `dv: this.file.frontmatter.date`

---

## What Happened Last Month?

## What Did not Go according to Plan?

## What Did I Achieve?

1. [achievement:: ]
2. [achievement:: ]
3. [achievement:: ]
