---
title:
  - 10 Introduction
aliases:
  - 10 Introduction
  - templater_documentation_10_Introduction
application: templater
url:
file_class: lib_documentation
date_created: 2023-03-10T15:37
date_modified: 2023-10-25T16:22
tags:
---
# Introduction

[Templater](https://github.com/SilentVoid13/Templater) is a template language that lets you insert **variables** and **functions** results into your notes. It will also let you execute JavaScript code manipulating those variables and functions.

With [Templater](https://github.com/SilentVoid13/Templater), you will be able to create powerful templates to automate manual tasks.

## Quick Example

The following template file, that is using [Templater](https://github.com/SilentVoid13/Templater) syntax:

```javascript
---
creation date: 2023-06-12 08:14
modification date: Sunday 13th August 2023 12:30:51
---

<< [[2023-08-23]] | [[2023-08-25]] >>

# Troubleshooting

> The most effective way to do it, is to do it.
> — <cite>Amelia Earhart</cite>
```

 Will produce the following result when inserted:

````
---
creation date: 2021-01-07 17:20
modification date: Thursday 7th January 2021 17:20:43
---

<< [[2021-04-08]] | [[2021-04-10]] >>

# Test Test

> Do the best you can until you know better. Then when you know better, do better.
> &mdash; <cite>Maya Angelou</cite>
````
