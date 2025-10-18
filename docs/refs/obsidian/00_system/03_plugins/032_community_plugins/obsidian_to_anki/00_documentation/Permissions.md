---
title:
  - Permissions
aliases:
  - Permissions
  - Permissions
  - permissions
  - obsidian_to_anki_permissions
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Permissions
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Permissions

The script needs to be able to:

- Make a [[04 Config|Config]] file in the directory the script is installed.
- Read the file in the directory the script is used.
- Make a backup file in the directory the script is used.
- Rename files in the directory the script is used.
- Remove a backup file in the directory the script is used.
- Change the current working directory temporarily (so that local image paths are resolved correctly).
