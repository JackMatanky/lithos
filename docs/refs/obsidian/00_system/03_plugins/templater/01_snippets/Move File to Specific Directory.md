---
aliases:
  - move_file_to_specific_directory
title: Move File to Specific Directory
date_created: 2023-03-28T14:06
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/templater, obsidian/templater/file_exists, obsidian/templater/file_move
---
# Script

```javascript
//-----------------------------------------
// MOVE TO PROJECTS DIRECTORY
//-----------------------------------------

const dir = '<directory name>/';
const sub_dir = (dir + '<title>');
if (!tp.file.exists(sub_dir)) {
  await this.app.vault.createFolder(sub_dir);
}
await tp.file.move(sub_dir + '/' + '<title>');
```

# Functions

- Obsidian API: [[createFolder]]
- Templater: [[file.exists]]
- Templater: [[file.move]]
