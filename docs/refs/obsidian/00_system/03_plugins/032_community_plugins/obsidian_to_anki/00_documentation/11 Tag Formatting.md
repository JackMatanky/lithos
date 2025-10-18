---
title:
  - 11 Tag Formatting
aliases:
  - 11 Tag Formatting
  - Tag Formatting
  - tag_formatting
  - obsidian_to_anki_tag_formatting
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Tag-formatting
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Tag Formatting

## Per-note Basis

For reference, the note formatting style is:

```
START
{Note Type}
{Note Fields}
Tags:
END
```

Note that the Tags: line is optional - if you don't want tags, you may leave out the line.

Tags should be formatted as such:

```
Tags: Tag1 Tag2 Tag3
```

So, **a space between the colon and the first tag**, and a space between tags.

Hence, this syntax **would not work**:

```
Tags:Tag1 Tag2 Tag3
```

The above section only applies to regular notes - see [[Inline Notes]] and [[03 Regex|Regex]] for respective information on those note types.

## Per-file Basis

These tags will be added to every card in the file.

To do this: Anywhere within the file, format the file tags as follows:

```
{File Tags Line}
{Tag list}
```

Or as:

```
{File Tags Line}: {Tag list}
```

For example, with the default settings:

```
FILE TAGS
Maths School Physics
```

Or:

```
FILE TAGS: Maths School Physics
```

Like with tag-line formatting, you need a space between tags - however, do not include the "Tags: " prefix.
