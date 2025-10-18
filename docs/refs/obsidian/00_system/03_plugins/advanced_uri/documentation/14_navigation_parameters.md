---
title: 14 Navigation Parameters
aliases:
  - advanced_uri_documentation_14_Navigation_Parameters
date_created: 2023-04-01T12:47
date_modified: 2023-10-25T16:22
tags: obsidian, obsidian/advanced_uri, documentation
---
# Navigation Parameters

## View Mode

Every action opening or focusing a pane supports the parameter `viewmode`. Accepted values:

- `source`: Sets the editor to editing:source mode
- `live`: Sets the editor to editing:live preview
- `preview`: Sets the editor to reading mode

## Open Mode

Every action opening a pane supports the parameter `openmode`. Accepted values:

- `true` opens file in new pane if not already opened
- `false` opens file in current pane if not already opened
- `window`
- `split`
- `tab`
- `silent` doesn't open the file
- `popover` which requires the [Hover Editor plugin](obsidian://show-plugin?id=obsidian-hover-editor) to be installed and enabled

If the file is already opened in another pane, it gets focused.

You can set a default value in the plugin's settings. The value from the setting gets overwritten by specifying it in the URI.
