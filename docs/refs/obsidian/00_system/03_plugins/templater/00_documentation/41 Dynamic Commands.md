---
title:
  - 41 Dynamic Commands
aliases:
  - 41 Dynamic Commands
  - templater_documentation_41_dynamic_commands
application: templater
url: 
file_class: lib_documentation
date_created: 2023-03-10T16:44
date_modified: 2023-10-25T16:22
tags: 
---
# Dynamic Commands

With this command utility, you can declare a command as "dynamic", which means that this command will be resolved when entering preview mode.

To declare a dynamic command add a plus `+` sign after the command opening tag: <\%+

That's it, your command will now be executed only in preview mode.

This is useful for internal functions like `tp.file.last_modified_date` for example:

```javascript
Last modified date: NaN
```

## Refresh Problems

One "downside" of the preview mode is that it puts the rendered note in cache, to speed things up.

This means that your dynamic command will be rendered only once, when you open the note, but won't be refreshed after.

If you want to refresh it, you must close the note to clear the cache and open it again.
