---
aliases:
  - file linking with aliases
title: File Linking with Aliases
subtitle: 
author: 
date_published: 
publisher: Obsidian Help
url: https://publish.obsidian.md/
type: documentation
file_class: lib_documentation
cssclasses:
date_created: 2023-03-21T11:41
date_modified: 2023-09-05T19:18
tags: obsidian, obsidian/internal_link
---
# Aliases

If you want to reference a file using different names, consider adding *aliases* to the note. An alias is an alternative name for a note.

Use aliases for things like acronyms, nicknames, or to refer to a note in a different language.

## Add an Alias to a Note

To add an alias for a note, add an `alias`, or `aliases`, property in the note [front matter](https://help.obsidian.md/Editing+and+formatting/Metadata):

```
---
aliases: Doggo
---

# Dog
```

You can add multiple aliases using commas:

```
---
aliases: Doggo, Woofer, Yapper
---

# Dog
```

Or, you can also add multiple aliases using a YAML array:

```
---
aliases:
  - Doggo
  - Woofer
  - Yapper
---

# Dog
```

## Link to a Note Using an Alias

To link to a note using an aliases:

1. Start typing the alias in an [internal link](https://help.obsidian.md/Linking+notes+and+files/Internal+links). Any alias shows up in the list of suggestions, with a curved arrow icon next to it.
2. Press `Enter` to select the alias.

Obsidian creates the link with the alias as its custom display text, for example `[[Artificial Intelligence|AI]]`.

> [!Note]  
> Rather than just using the alias as the link destination (`[[AI]]`), Obsidian uses the `[[Artificial Intelligence|AI]]` link format to ensure interoperability with other applications using the Wikilink format.

## Find Unlinked Mentions for an Alias

By using [Backlinks](https://help.obsidian.md/Plugins/Backlinks), you can find unlinked mentions of aliases.

For example, after setting "AI" as an alias for "Artificial intelligence", you can see mentions of "AI" in other notes.

If you link an unlinked mention to an alias, Obsidian turns the mention into an [internal link](https://help.obsidian.md/Linking+notes+and+files/Internal+links) with the alias as its display text.
