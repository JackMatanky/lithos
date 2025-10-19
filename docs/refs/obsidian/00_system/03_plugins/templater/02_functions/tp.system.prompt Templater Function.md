---
title: tp.system.prompt Templater Function
aliases:
  - tp.system.prompt()
  - tp.system.prompt
  - The Templater tp.system.prompt() Function
  - prompt
  - system.prompt
  - tp.system.prompt
  - templater_system.prompt
language:
  - javascript
plugin: templater
module: system
syntax: "tp.system.prompt(prompt_text?: String, default_value?: String, throw_on_cancel: Boolean = False, Multiline?: Boolean = False)"
url: https://silentvoid13.github.io/Templater/internal-functions/internal-modules/system-module.html#tpsystempromptprompt_text-string-default_value-string-throw_on_cancel-boolean--false-multiline-boolean--false
cssclasses:
type: function
file_class: pkm_code
date_created: 2023-03-28T00:00
date_modified: 2023-10-25T16:22
tags: javascript, obsidian, obsidian/templater, obsidian/tp/system/prompt
---
# The Templater `tp.system.prompt()` Function

## Description

> [!function] Function Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Module: System
> Input::
> Output::
> Definition:: Spawns a prompt modal and returns the user's input.
>
> Link: [tp.system.prompt](https://silentvoid13.github.io/Templater/internal-functions/internal-modules/system-module.html#tpsystempromptprompt_text-string-default_value-string-throw_on_cancel-boolean--false-multiline-boolean--false)

---

## Syntax

```javascript
tp.system.prompt(
  prompt_text?: String,
  default_value?: String,
  throw_on_cancel: Boolean = False,
  Multiline?: Boolean = False
)
```

## Parameter Values

| Parameter       |  Type   | Description                                                                     |
|:--------------- |:-------:|:------------------------------------------------------------------------------- |
| prompt_text     | string  | Text placed above the input field.                                              |
| default_value   | string  | A default value for the input field                                             |
| throw_on_cancel | boolean | Throws an error if the prompt is canceled, instead of returning a `null` value. |
| multiline       | boolean | If set to true, the input field will be a multiline text area                   |

## Additional Information

## Examples

```<%* tR += language %>

```

## Notes and Remarks

---

## Related

### Snippets (Use Cases)

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Functions

#### By Plugin

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.module AS Module,
	Definition AS Definition
WHERE
	file.name != this.file.name
	AND (file.frontmatter.file_class = "pkm_code_function")
	AND (file.frontmatter.plugin = this.file.frontmatter.plugin)
SORT file.frontmatter.module, file.name
```

#### By Tag

<!-- Add tags in contains function as needed  -->
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition,
	string(file.frontmatter.language) AS Language,
	sort(file.etags) AS Tags
WHERE
	file.name != this.file.name
	AND file.frontmatter.file_class = "pkm_code_function"
	AND contains(file.tags, "system")
SORT file.frontmatter.language, file.name
LIMIT 10
```

#### Outgoing Function Links

<!-- Link related functions here -->

#### All Function Links

<!-- Excluding functions of the same module  -->
<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.definition AS Definition
WHERE
	file.name != this.file.name
	AND file.frontmatter.module != this.file.frontmatter.module
	AND file.frontmatter.file_class = "pkm_code_function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
