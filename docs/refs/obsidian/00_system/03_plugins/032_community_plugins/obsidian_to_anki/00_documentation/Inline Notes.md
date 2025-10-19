---
title:
  - Inline Notes
aliases:
  - Inline Notes
  - Inline Notes
  - inline_notes
  - obsidian_to_anki_inline_notes
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Inline-notes
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags:
---
# Inline Notes

These are formatted as such:

```
\START\I [{Note Type}] {Note Data} \END\I
```

For example

```
\START\I [Basic] This is a test. \Back: Test successful! \END\I
```

![[Pasted image 20230530193551.webp]]

Unlike regular 'block' notes, you can put inline notes anywhere on a line - for example, you could have a bulletpointed list of inline notes.
Also, unlike regular 'block' notes, the script identifies the note type through the string in square brackets. Hence, **note types with \[ or \] in the name are not supported for inline notes.**
