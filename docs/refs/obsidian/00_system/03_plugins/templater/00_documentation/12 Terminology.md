---
title:
  - 12 Terminology
aliases:
  - 12 Terminology
  - templater_documentation_12_Terminology
application: templater
url: 
file_class: lib_documentation
date_created: 2023-03-10T15:40
date_modified: 2023-10-25T16:22
tags: 
---
# Terminology

To understand how [Templater](https://github.com/SilentVoid13/Templater) works, let's define a few terms:

- A **template** is a file that contains **[commands](./commands/overview.md)**.
- A text snippet that starts with an opening tag `, ends with a closing tag ` is what we will call a **[command](./commands/overview.md)**.
- A **function** is an object that we can invoke inside a **command** and that returns a value (the replacement string)

There are two types of functions you can use:

- [Internal functions](./internal-functions/overview.md). They are **predefined** functions that are built within the plugin. As an example, `tp.date.now` is an internal function that will return the current date.
- [User functions](./user-functions/overview.md). Users can define their own functions. They are either [system command](./user-functions/system-user-functions.md) or [user scripts](./user-functions/script-user-functions.md).

## Example

The following template contains 2 commands, calling 2 different internal functions:

```
Yesterday: 2023-08-23
Tomorrow: 2023-08-25
```

We'll see in the next part the syntax required to write some commands.
