---
title:
  - 26 System Module
aliases:
  - 26 System Module
  - templater_documentation_26_system_module
application: templater
url: 
file_class: lib_documentation
date_created: 2023-03-10T15:45
date_modified: 2023-10-25T16:22
tags: 
---
# System Module

This module contains system related functions.

## Documentation

Function documentation is using a specific syntax. More information [here](../../syntax.md#function-documentation-syntax)

### `tp.system.clipboard()`

Retrieves the clipboard's content

### [[tp.system.prompt Templater Function|system.prompt]]

#### Syntax

```javascript
tp.system.prompt(prompt_text?: String, default_value?: String, throw_on_cancel: Boolean = False, Multiline?: Boolean = false)
```

Spawns a prompt modal and returns the user's input.

#### Arguments

- `default_value`: A default value for the input field
    
- `multiline`: If set to true, the input field will be a multiline text area
    
- `prompt_text`: Text placed above the input field
    
- `throw_on_cancel`: Throws an error if the prompt is canceled, instead of returning a `null` value

### [[tp.system.suggester Templater Function]]

Spawns a suggester prompt and returns the user's chosen item.

#### Syntax

```javascript
tp.system.suggester(text_items: string[] ⎮ ((item: T) => string), Items: T[], throw_on_cancel: Boolean = False, Placeholder: String = "", Limit?: Number = undefined)
```

#### Arguments

- `items`: Array containing the values of each item in the correct order.
    
- `limit`: Limit the number of items rendered at once (useful to improve performance when displaying large lists)
    
- `placeholder`: Placeholder string of the prompt
    
- `text_items`: Array of strings representing the text that will be displayed for each item in the suggester prompt. This can also be a function that maps an item to its text representation.
    
- `throw_on_cancel`: Throws an error if the prompt is canceled, instead of returning a `null` value

## Examples

```javascript
Clipboard content: <% tp.system.clipboard() %>

Entered value: <% tp.system.prompt("Please enter a value") %>
Mood today: <% tp.system.prompt("What is your mood today ?", "happy") %>

Mood today: <% tp.system.suggester(["Happy", "Sad", "Confused"], ["Happy", "Sad", "Confused"]) %>
Picked file: [[<% (await tp.system.suggester((item) => item.basename, app.vault.getMarkdownFiles())).basename %>]]


 <%* const execution_value = await tp.system.suggester(["Yes", "No"], ["true", "false"]) %>
Are you using Execution Commands: <%*  tR + execution_value %>

```
