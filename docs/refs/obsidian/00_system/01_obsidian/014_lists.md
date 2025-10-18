---
aliases:
  - Lists_Obsidian Markdown Syntax
title: Format your notes
subtitle:
author: 
date_published: 
publisher: Obsidian Help
url: https://publish.obsidian.md/
type: documentation
file_class: lib_documentation
cssclasses:
date_created: 2023-03-21T12:12
date_modified: 2023-09-05T19:18
tags: markdown/obsidian, obsidian, markdown, tags
---
# Lists

## Unordered Lists

```
- Item 1
- Item 2
  - Item 2a
  - Item 2b
```

- Item 1
- Item 2
    - Item 2a
    - Item 2b

## Ordered Lists (Numbered)

```
1. Item 1
2. Item 2
3. Item 3
   1. Item 3a
   2. Item 3b
```

1. Item 1
2. Item 2
3. Item 3
    1. Item 3a
    2. Item 3b

Create a *loose list* by adding a blank line between any two list items.

```
- Item 1

- Item 2

- Item 3
```

Will look like this:

- Item 1
    
- Item 2
    
- Item 3

## Task List

```
- [x] #tags, [links](), **formatting** supported
- [x] list syntax required (any unordered or ordered list supported)
- [x] this is a complete item
- [?] this is also a complete item (works with every character)
- [ ] this is an incomplete item
- [ ] tasks can be clicked in Preview to be checked off
```

- [x] #tags, [links](), **formatting** supported
- [x] list syntax required (any unordered or ordered list supported)
- [x] this is a complete item
- [?] this is also a complete item (works with every character)
- [ ] this is an incomplete item
- [ ] tasks can be clicked in Preview to be checked off
