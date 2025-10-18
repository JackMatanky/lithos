---
title:
  - 30 User Functions
aliases:
  - 30 User Functions
  - templater_documentation_30_user_functions
application: templater
url: 
file_class: lib_documentation
date_created: 2023-03-10T15:46
date_modified: 2023-10-25T16:22
tags: 
---
# User Functions

You can define your own functions in Templater.

There are two types of user functions you can use:

- [[31 Script User Functions]]
- [[32 System Command User Functions]]

## Invoking User Functions

You can call a user function using the usual function call syntax: `tp.user.<user_function_name>()`, where `<user_function_name>` is the function name you defined.

For example, if you defined a system command user function named `echo`, a complete command invocation would look like this:

```js
 <% tp.user.echo() %>
```

## No Mobile Support

Currently user functions are unavailable on Obsidian for mobile.
