---
title:
  - 05 Usage
aliases:
  - 05 Usage
  - Usage
  - usage
  - obsidian_to_anki_usage
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Usage
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Usage

**Apart from editing the config file, all operations of the script require Anki to be running.**

The GUI of the script looks like this:  
![[GUI.webp]]

Hopefully the options and path are self-explanatory. The 'recurse' option can be used on the top-level notes directory - it'll pick up all the notes in subfolders automatically. Note that you can run the script over the same file twice perfectly fine - it won't add duplicate cards.

## Command line Usage

If you set 'GUI' in the config file to False, the script is then run from the command line:

- Use `-h` to see help.
- Run the script as `obsidian_to_anki.py [path]`, where `[path]` is the path to the file or folder you wish to add notes from.
- Use `-c` to open up the config file for editing (not guaranteed to work on all operating systems, if it doesn't you'll have to find and edit it manually).
- Use `-u` to update the config file. Do this when you add new note types to Anki.
- Use `-m` to force the script to add all media files detected, instead of lazy addition of media files. Useful if you've e.g. resized the image, and want the changes to be reflected in Anki.
- Use `-r` to use custom regex syntax, ignoring the default syntax of the script.
- Use `-R` to recursively scan subfolders.
