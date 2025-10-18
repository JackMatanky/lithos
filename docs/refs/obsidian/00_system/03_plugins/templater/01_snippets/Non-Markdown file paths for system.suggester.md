---
aliases:
  - Non-Markdown file paths for system.suggester
title: Non-Markdown file paths for system.suggester
date_created: 2023-03-28T14:06
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/templater, obsidian/templater/suggester, obsidian/api
---
# Script

```javascript
const files = app.vault.getFiles().filter(f => f.path.includes('<insert folder name or partial file path>'));

// Retrieve full file name and file type (.png, .jpeg, etc.)
  const file_names = people_profile_files.map(f => f.name);
  const file = await tp.system.suggester(file_names, file_names);
```

# Functions Used

- Obsidian API: [[getAllLoadedFiles]]
- Templater: [[tp.system.suggester Templater Function]]
- JavaScript: [[filter]]
- JavaScript: [[children]]
- JavaScript: [[map]]
