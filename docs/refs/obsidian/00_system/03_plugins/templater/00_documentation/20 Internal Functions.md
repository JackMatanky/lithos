---
title:
  - 20 Internal Functions
aliases:
  - 20 Internal Functions
  - templater_documentation_20_internal_functions
application: templater
url:
file_class: lib_documentation
date_created: 2023-03-10T15:44
date_modified: 2023-10-25T16:22
tags:
---
# Internal Functions

The different internal variables and functions offered by [Templater](https://github.com/SilentVoid13/Templater) are available under different **modules**, to sort them. The existing **internal modules** are:

- [[21 Config Module]]: `tp.config`
- [[22 Date Module]]: `tp.date`
- [[23 File Module]]: `tp.file`
- [[24 Frontmatter Module]]: `tp.frontmatter`
- [[25 Obsidian Module]]: `tp.obsidian`
- [[26 System Module]]: `tp.system`
- [[27 Web Module]]: `tp.web`

If you understood the [object hierarchy](../syntax.md#objects-hierarchy) correctly, this means that a typical internal function call looks like this: ` <% tp.<module_name>.<internal_function_name> %>`

## Contribution

I invite everyone to contribute to this plugin development by adding new internal functions. More information [here](./contribute.md).
