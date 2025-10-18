<%*
//-------------------------------------------------------------------
// GLOBAL FILE PATH VARIABLES
//-------------------------------------------------------------------
const goals_dir = "30_goals/";
const value_goals_dir = "30_goals/31_value_goals/";
const outcome_goals_dir = "30_goals/31_outcome_goals/";
const personal_proj_dir = "41_personal/";
const habit_ritual_proj_dir = "42_habits_and_rituals/";
const education_proj_dir = "42_education/";
const professional_proj_dir = "43_professional/";
const work_proj_dir = "44_work/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";
const pkm_dir = "70_pkm/";
const tools_pkm_dir = "70_pkm/86_tools/";

//-------------------------------------------------------------------
// SET EDUCATIONAL INSTITUTION
//-------------------------------------------------------------------

//-------------------------------------------------------------------
// SET THE FILE'S TITLE
//-------------------------------------------------------------------
// Check if note already has title
const has_title = !tp.file.title.startsWith("Untitled");
let title;

// If note does not have title,
// prompt for title and rename file
if (!has_title) {
  title = await tp.system.prompt("Title");
  await tp.file.rename(title);
} else {
  title = tp.file.title;
}

//-------------------------------------------------------------------
// SET PROGRAMMING LANGUAGE
//-------------------------------------------------------------------
// Get all the vault's folder paths
const all_directory_paths = app.vault
  .getAllLoadedFiles()
  .filter((i) => i.children)
  .map((folder) => folder.path);

// Filter array to only include folder paths in the tools directory
const tools_dirs = all_directory_paths.filter((path) =>
  path.includes(tools_pkm_dir)
);

// Extract the tool name from the directory path
const tool_names = tools_dirs
  .map((tool_path) => tool_path.split("/")[2].toLowerCase())
  .filter((tool_name) => tool_name);

// Filter array to show unique values
//const tools = [new Set(tool_names)].sort();
let tools = [];
tool_names.forEach((item) => {
  if (!tools.includes(item)) {
    tools.push(item);
  }
});

// Sort the array values alphabetically
tools.sort();

// Choose a project
let language = await tp.system.suggester(
  tools,
  tools,
  false,
  "Programming Language?"
);

const alias = language + "_" + title.toLowerCase();

tR += "---";
%>
title: \<course name>*\<course id> (title case)
aliases: \<institution short name>*\<date>\<semester>*\<course name>*\<course id> (lower case)
institution: Open University of Israel
format: online, frontal
date: YYYY
semester: b
course_name: Linear Algebra I
course_id: 20109
lecturer:
file_class: lib_course
date_created:
date_modified:
---

# {{title}}

> [!info]- General Course Information
> Lecturer:  
> Phone:  
> Email:  
> Lecture day and time:  

## Syllabus

## Lectures

## Assignments

## Course Materials
