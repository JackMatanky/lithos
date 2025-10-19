---
title:
  - 22 Writing
aliases:
  - 22 Writing
  - advanced_uri_documentation_22_Writing
date_created: 2023-04-01T12:47
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/advanced_uri, documentation
---
# Writing

> [!caution]
Make sure your values are properly [encoded](../concepts/encoding.md)

> [!info]
The `data` parameter can be replaced with `clipboard=true` to get the content from the clipboard.

| /         | parameters                              | explanation                                                                                     |
| --------- | --------------------------------------- | ----------------------------------------------------------------------------------------------- |
| write     | <identification\>, data                 | Only writes `data` to the file if the file is not already present                               |
| overwrite | <identification\>, data, mode=overwrite | Writes `data` to `filepath` even if the file already exists                                     |
| append    | <identification\>, data, mode=append    | Only appends `data` to the file                                                                 |
| prepend   | <identification\>, data, mode=prepend   | Only prepends `data` to the file                                                                |
| new       | filepath, data, mode=new                | Definitely creates a new file. If `filepath` already exists, an incrementing number is appended |

> [!note] Example
> **Write** "Hello World" to "my-file.md":
>
> ```uri
> obsidian://advanced-uri?vault=<'your-vault'> &filepath=my-file&data=Hello%2520World
> ```
>
> **Overwrite** "This text is overwritten" to "my-file.md":
>
> ```uri
> obsidian://advanced-uri?vault=<'your-vault'> &filepath=my-file&data=This%2520text%2520is%2520overwritten&mode=overwrite
> ```
>
> **Append** "Hello World" to today's **daily note**:
>
> ```uri
> obsidian://advanced-uri?vault=<'your-vault'> &daily=true&data=Hello%2520World&mode=append
> ```
>
> **Append** content from the **clipboard** to today's **daily note**:
>
> ```uri
> obsidian://advanced-uri?vault=<'your-vault'> &daily=true&clipboard=true&mode=append
> ```

> [!info]
> You may use the `heading` parameter to append and prepend data to a heading. More information in [Navigation](navigation.md)
