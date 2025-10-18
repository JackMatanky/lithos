---
title:
  - Obsidian Integration
aliases:
  - Obsidian Integration
  - Obsidian Integration
  - obsidian_integration
  - obsidian_to_anki_obsidian_integration
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Obsidian-Integration
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Obsidian Integration

## All Users

### Add File Link

This will automatically append a link to the file that generated the flashcard, on the first field.

## Obsidian Plugin Users

### Add File Link

Using the plugin settings, you can choose which field to append the link to!

### Link Support

The plugin fully supports both normal markdown links and Obsidian style wikilinks.

### Embed Support

The plugin supports obsidian-style image and audio embeds.

### Tag Support

The switch "Add Obsidian tags" tells the plugin to interpret \#tags as tags for Anki.

### Context

Using the plugin settings, you can choose which field to append 'context' to! This is the path of the file the note was generated from, as well as the position of the note in the heading tree (if any) of the file.

For example, if the file was called "test.md", in the folder "Math/Functions", and we had a note in this position:

```

# Overall point

## Subheading 1

## Subheading 2

\START
Basic
This is a test
Back: Test successful!
\END

```

Then, the context for the note would be the string "Math/Functions/test.md > Overall point > Subheading 2"

### Folder Settings

See [[Folder Settings]] for a detailed explanation of this.
