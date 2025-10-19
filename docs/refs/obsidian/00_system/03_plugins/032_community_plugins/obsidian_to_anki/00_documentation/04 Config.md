---
title:
  - 04 Config
aliases:
  - 04 Config
  - Config
  - config
  - obsidian_to_anki_config
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Config
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags:
---
# Config

## DEFAULT Section

Section for setting default parameters of the script:

### All Users

- Tag - the default tag added to every card produced by this script.
- Deck - the default deck every card produced by this script goes into. Overridden by TARGET DECK (see [syntax](#syntax))
- CurlyCloze - explained in [[Cloze-formatting]]
- GUI - allows you to enable/disable the GUI of the script. See [[Command line usage]]
- ID Comments - Toggle whether or not card IDs are wrapped in a HTML comment. Wrapping IDs in a HTML comment makes them invisible on 'preview' mode, which can make notes look cleaner.
- Add file link: Whether you want to append a link to the associated obsidian file.

### Python Script Users

- Anki Path and Anki Profile - If you supply both the absolute path to the Anki executable, and your profile on Anki, the script will attempt to open Anki when run if it's not already running. Useful for automation - see [[Technical]]

Note that 'Add file link' currently only adds to the first field of the card in the script, whereas in the plugin you can configure which field to add the link to.

## Syntax

Note that START, END, TARGET DECK, FILE TAGS and DELETE all require an **exact match*- on the line - you cannot have spaces afterwards.

- Begin Note - The string that signals the start of a [note](#note-formatting). Defaults to START.
- End Note - The string that signals the end of a note. Defaults to END.
- Begin Inline Note - The string that signals the start of an [inline note](#inline-note-formatting). Defaults to STARTI (Start-Inline).
- End Inline Note - The string that signals the end of an inline note. Defaults to ENDI (End-Inline).
- Target Deck Line - The string that signals "the line beneath me is the name of the target deck". Defaults to TARGET DECK. Can also use as TARGET DECK: {Deck Name} - see [[08 Deck Formatting|Deck Formatting]]
- File Tags Line - The string that signals "the line beneath me is the set of tags that should be added to all notes from this file". Defaults to FILE TAGS. Can also use as FILE TAGS: {Tag List} - see [[11 Tag Formatting|Tag Formatting]]
- Delete Note Line - The string that signals "the line beneath me is an id string for a note that should be deleted." Defaults to DELETE.

## Custom Regexp Section

See [[03 Regex|Regex]] for an explanation of this.

## Folder Settings (Obsidian Plugin only)

See [[Folder Settings]] for more info.
