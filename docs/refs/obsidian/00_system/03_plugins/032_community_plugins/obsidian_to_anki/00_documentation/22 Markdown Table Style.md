---
title:
  - 22 Markdown Table Style
aliases:
  - 22 Markdown Table Style
  - Markdown Table Style
  - markdown_table_style
  - obsidian_to_anki_markdown_table_style
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Markdown-table-style
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags:
---
# Markdown Table Style

## Usage

**[[03 Regex|Regex]] line:** `\|([^\n|]+)\|\n\|(?:[^\n|]+)\|\n\|([^\n|]+)\|\n?`

1. Create a file called `test.md`.
2. Paste the following contents into the file:

```

| How do you use this style? |

| ---- |

| Just like this |



Of course, the script will ignore anything outside a table.



| Furthermore, the script | should also |

| ----- | ----- |

| Ignore any tables | with more than one column |



| Why might this style be useful? |

| --------- |

| It looks nice when rendered as HTML in a markdown editor. |

```

### Obsidian Plugin Users

1. In the plugin settings, paste the Regex line into the 'Custom Regexps' field associated with 'Basic'
2. Ensure that the 'Regex' option is checked
3. Click the Anki icon on the ribbon to run the plugin

### Python Script Users

1. Run the script, and check 'Config' to open up the config file:
   ![[GUI_config.webp]]
2. Navigate to the "Custom Regexps" section
3. Change the line

   ```
   Basic =  
   ```

   to  

   ```
   Basic = \|([^\n|]+)\|\n\|(?:[^\n|]+)\|\n\|([^\n|]+)\|\n?
   ```

4. Save the config file
5. Run the script on the file, with 'Regex' checked:
    ![[GUI_regex.webp]]

### All Users

1. You should see these cards in Anki:
   ![[Table_1.webp]]
   ![[Table_2.webp]]
