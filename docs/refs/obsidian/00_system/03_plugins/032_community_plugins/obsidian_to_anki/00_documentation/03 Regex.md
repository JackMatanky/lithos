---
title:
  - 03 Regex
aliases:
  - 03 Regex
  - Regex
  - regex
  - obsidian_to_anki_regex
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Regex
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Regex

## Table of Contents

This page lists templates for custom syntax. In each case, copy-paste the regex line into the desired note type in the config file to use the template.

- [[RemNote single-line style]]
- [[24 Header Paragraph Style|Header Paragraph Style]]
- [[25 Question Answer Style|Question Answer Style]]
- [[Neuracache flashcard style]]
- [[Ruled style]]
- [[22 Markdown Table Style|Markdown Table Style]]
- [[23 Cloze Paragraph Style|Cloze Paragraph Style]]

## Custom Styles

The above styles are but a few examples of the endless possible styles you can make using regular expressions. If you want to make your own style, however, you should know these things:

- The script automatically compiles the regular expression with a 'multiline' flag, so you can use the `^` character to signal the beginning of a line
- You need to have as many capture groups in your regexp as there are fields in the note type - the 1st capture group becomes the 1st field, the 2nd becomes the 2nd field etc
- If making a 'paragraph' regex, consider using this group to match lines at the end - `(?:^.{1,3}$|^.{4}(?<!<!--).*))`. It ensures that you don't accidentally match the `<!--` at the start of an ID comment!

If you'd like for your style to be added to this page, make a style-request issue and I'll consider it.

## Tagging Notes

Cards made using this format support tags - simply append a "Tags: {tag_list}" to the end of your block. The guidance is to use same line for single-line regexps, and the following line for paragraph regexps. If you're having problems, do consider whether FILE TAGS or tags from the folder (see [[Folder Settings]]) would be easier!

### Obsidian Plugin Users

## Deleting Notes

To delete notes made using this format, remove the content before the ID and make it look like:

```
{Delete Regex Note Line}  
&lt;!--ID: 129840142123--&gt;  
```

With the default settings:

```
DELETE  
&lt;!--ID: 129414201900--&gt;  
```

Note that if you manually delete a note in Anki, you should remove the ID line from Obsidian/the file too. The script will print a message if a note is identified with an ID that doesn't exist in Anki.

## Conflicts?

Try to make sure your regex matches don't overlap with each other. The script is designed, however, to not recognise a match inside another match (for different note types).

For example, if you're using the default syntax of the script for the 'Cloze' note type:

```
START
Cloze
This is a {{c1::test}}
END
```

You don't have to worry about a RemNote single-line match being picked up.
