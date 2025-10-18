---
title:
  - 32 From within Obsidian
aliases:
  - 32 From within Obsidian
  - advanced_uri_documentation_32_From_within_Obsidian
date_created: 2023-04-01T12:47
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/advanced_uri, documentation
---
# From within Obsidian

You can access the query parameters of the last opened Advanced URI via the following:

```js
const lastParameters = app.plugins.plugins["obsidian-advanced-uri"].lastParameters
```

This can be useful to continue processing the URI via the dataview or templater plugin. See [#77](https://github.com/Vinzent03/obsidian-advanced-uri/issues/77) for the initial request and use case.
