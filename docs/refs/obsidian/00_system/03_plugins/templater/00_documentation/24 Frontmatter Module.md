---
title:
  - 24 Frontmatter Module
aliases:
  - 24 Frontmatter Module
  - templater_documentation_24_frontmatter_module
application: templater
url: 
file_class: lib_documentation
date_created: 2023-03-10T15:45
date_modified: 2023-10-25T16:22
tags: 
---
# Frontmatter Module

## Documentation

### `tp.frontmatter.<frontmatter_variable_name>`

Retrieves the file's frontmatter variable value.

If your frontmatter variable name contains spaces, you can reference it using the bracket notation like so:

````
undefined
````

## Examples

Let's say you have the following file:

````
---
aliases: myfile
note type: seedling
---

file content
````

Then you can use the following template:

````
File's metadata aliases: undefined
Note's type: undefined
````
