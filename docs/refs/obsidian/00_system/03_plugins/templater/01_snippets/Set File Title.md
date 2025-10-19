---
aliases:
  - system.suggester_set_time
title: Set time for system.suggester
date_created: 2023-03-28T14:06
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/templater, obsidian/templater/suggester
---
# Script

```javascript
//-----------------------------------------
// SET THE FILE'S TITLE
//-----------------------------------------
let title = tp.file.title

// If the file's title is "Untitled",
// add a new title based on user input for title
if (title.startsWith("Untitled")) {
  title = await tp.system.prompt("Title");
  await tp.file.rename(title);
  }
```

# Functions Used

- JavaScript: [[startsWith]]
- Templater: [[system.prompt1]]
- Templater: [[file.rename]]
