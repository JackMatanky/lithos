---
title:
  - 23 File Module
aliases:
  - 23 File Module
  - templater_documentation_23_file_module
application: templater
url:
file_class: lib_documentation
date_created: 2023-03-10T15:44
date_modified: 2023-10-25T16:22
tags:
---
# File Module

This module contains every internal function related to files.

## Documentation

Function documentation is using a specific syntax. More information [here](https://silentvoid13.github.io/Templater/syntax.html#function-documentation-syntax)

### `tp.file.content`

Retrieves the file's content

### tp.file.create_new

#### Syntax

`tp.file.create_new(template: TFile ⎮ String, Filename?: String, open_new: Boolean = False, Folder?: TFolder)`

Creates a new file using a specified template or with a specified content.

#### Arguments

- `filename`: The filename of the new file, defaults to "Untitled".

- `folder`: The folder to put the new file in, defaults to obsidian's default location. If you want the file to appear in a different folder, specify it with `app.vault.getAbstractFileByPath("FOLDERNAME")`

- `open_new`: Whether to open or not the newly created file. Warning: if you use this option, since commands are executed asynchronously, the file can be opened first and then other commands are appended to that new file and not the previous file.

- `template`: Either the template used for the new file content, or the file content as a string. If it is the template to use, you retrieve it with `tp.file.find_tfile(TEMPLATENAME)`

### `tp.file.creation_date(format: String = "YYYY-MM-DD HH:mm")`

Retrieves the file's creation date.

#### Arguments

- `format`: Format for the date, refer to format reference

### `tp.file.cursor(order?: number)`

Sets the cursor to this location after the template has been inserted.

You can navigate between the different `tp.file.cursor` using the configured hotkey in obsidian settings.

#### Arguments

- `order`: The order of the different cursors jump, e.g. it will jump from 1 to 2 to 3, and so on. If you specify multiple tp.file.cursor with the same order, the editor will switch to multi-cursor.

### `tp.file.cursor_append(content: string)`

Appends some content after the active cursor in the file.

#### Arguments

- `content`: The content to append after the active cursor

### `tp.file.exists(filename: string)`

The filename of the file we want to check existence. The fullpath to the file, relative to the Vault and containing the extension, must be provided. e.g. MyFolder/SubFolder/MyFile.

#### Arguments

- `filename`: The filename of the file we want to check existence, e.g. MyFile.

### `tp.file.find_tfile(filename: string)`

Search for a file and returns its `TFile` instance

#### Arguments

- `filename`: The filename we want to search and resolve as a `TFile`

### `tp.file.folder(relative: Boolean = false)`

Retrieves the file's folder name.

#### Arguments

- `relative`: If set to true, appends the vault relative path to the folder name.

### `tp.file.include(include_link: String ⎮ TFile)`

Includes the file's link content. Templates in the included content will be resolved.

#### Arguments

- `include_link`: The link to the file to include, e.g. [[MyFile]], or a TFile object. Also supports sections or blocks inclusions, e.g. [[MyFile#Section1]]

### `tp.file.last_modified_date(format: String = "YYYY-MM-DD HH:mm")`

Retrieves the file's last modification date.

#### Arguments

- `format`: Format for the date, refer to format reference.

### `tp.file.move(new_path: String, file_to_move?: TFile)`

Moves the file to the desired vault location.

#### Arguments

- `new_path`: The new vault relative path of the file, without the file extension. Note: the new path needs to include the folder and the filename, e.g. /Notes/MyNote

### `tp.file.path(relative: Boolean = false)`

Retrieves the file's absolute path on the system.

#### Arguments

- `relative`: If set to true, only retrieves the vault's relative path.

### `tp.file.rename(new_title: string)`

Renames the file (keeps the same file extension).

#### Arguments

- `new_title`: The new file title.

### `tp.file.selection()`

Retrieves the active file's text selection.

### `tp.file.tags`

Retrieves the file's tags (array of string)

### `tp.file.title`

Retrieves the file's title.

## Examples

```javascript
File content: <% tp.file.content %>

File creation date: <% tp.file.creation_date() %>
File creation date with format: <% tp.file.creation_date("dddd Do MMMM YYYY HH:mm") %>

File creation: [[<% (await tp.file.create_new("MyFileContent", "MyFilename")).basename %>]]

File cursor: <% tp.file.cursor(1) %>

File cursor append: <% tp.file.cursor_append("Some text") %>

File existence: <% await tp.file.exists("MyFolder/MyFile.md") %>
File existence of current file: <% await tp.file.exists(tp.file.folder(true)+"/"+tp.file.title+".md") %>

File find TFile: <% tp.file.find_tfile("MyFile").basename %>

File Folder: <% tp.file.folder() %>
File Folder with relative path: <% tp.file.folder(true) %>

File Include: <% tp.file.include("[[Template1]]") %>

File Last Modif Date: <% tp.file.last_modified_date() %>
File Last Modif Date with format: <% tp.file.last_modified_date("dddd Do MMMM YYYY HH:mm") %>

File Move: <% await tp.file.move("/A/B/" + tp.file.title) %>
File Move + Rename: <% await tp.file.move("/A/B/NewTitle") %>

File Path: <% tp.file.path() %>
File Path with relative path: <% tp.file.path(true) %>

File Rename: <% await tp.file.rename("MyNewName") %>
Append a "2": <% await tp.file.rename(tp.file.title + "2") %>

File Selection: <% tp.file.selection() %>

File tags: <% tp.file.tags %>

File title: <% tp.file.title %>
Strip the Zettelkasten ID of title (if space separated): <% tp.file.title.split(" ")[1] %>
```
