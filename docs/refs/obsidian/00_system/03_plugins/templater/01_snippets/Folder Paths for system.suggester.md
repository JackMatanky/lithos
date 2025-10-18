---
aliases:
  - Folder Paths for system.suggester
title: Folder Paths for system.suggester
date_created: 2023-03-28T14:06
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/templater, obsidian/templater/suggester, obsidian/api
---
# Script

## Original Script

Link: <https://discord.com/channels/686053708261228577/875720842443649045/1089826746259476511>

```
<%* 
const folders = app.vault.getAllLoadedFiles().filter(i => i.children).map(folder => folder.path); 
const choice = await tp.system.suggester(folders, folders); 
tR += choice 
%>
```

Or if you want to chain suggesters and first pick a folder, then a file from that folder

```
<%*
const folders = app.vault.getAllLoadedFiles().filter(i => i.children).map(folder => folder.path);
const folder = await tp.system.suggester(folders, folders);

const files = app.vault.getMarkdownFiles().filter(f => f.path.includes(folder))
const filenames = files.map(f => f.basename)
const file = await tp.system.suggester(filenames, filenames)

tR += file
%>
```

## Revised Script

```
<%*
// Get all the vault's folder paths
const all_directory_paths = app.vault.getAllLoadedFiles().filter(i => i.children).map(folder => folder.path);
  
// Filter array to only include project folder paths
// Extract the first subdirectory name, or the project name
const project_directories = all_directory_paths.filter(path => path.includes('40_projects/')).map(project_path => project_path.split('/')[1]);
  
// Filter array to show unique values
let projects = [];
project_directories.forEach((item) => {
  if(!projects.includes(item)) {
    projects.push(item);
  }
});
  
// Choose a project
const project = await tp.system.suggester(projects, projects);
%>
```

# Functions Used

- Obsidian API: [[getAllLoadedFiles]]
- Templater: [[tp.system.suggester Templater Function]]
- JavaScript: [[filter]]
- JavaScript: [[children]]
- JavaScript: [[map]]
