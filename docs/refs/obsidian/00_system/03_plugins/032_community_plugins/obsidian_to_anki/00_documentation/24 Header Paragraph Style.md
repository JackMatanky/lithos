---
title:
  - 24 Header Paragraph Style
aliases:
  - 24 Header Paragraph Style
  - Header Paragraph Style
  - header_paragraph_style
  - obsidian_to_anki_header_paragraph_style
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Header-paragraph-style
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags:
---
# Header Paragraph Style

## Usage

**[[03 Regex|Regex]] line:** `^#+(.+)\n*((?:\n(?:^[^\n#].{0,2}$|^[^\n#].{3}(?<!<!--).*))+)`

1. Create a file called `test.md`
2. Paste the following contents into the file:

```
# Style
This style is suitable for having the header as the front, and the answer as the back
# Overall heading
## Subheading 1
You're allowed to nest headers within each other
## Subheading 2
It'll take the deepest level for the question
## Subheading 3



It'll even
Span over
Multiple lines, and ignore preceding whitespace
```

### Obsidian Plugin Users

1. In the plugin settings, paste the Regex line into the 'Custom Regexps' field associated with 'Basic'
2. Ensure that the 'Regex' option is checked
3. Click the Anki icon on the ribbon to run the plugin

### Python Script Users

1. Run the script, and check 'Config' to open up the config file: ![[GUI_config.webp]]
2. Navigate to the "Custom Regexps" section
3. Change the line

```
Basic =
```

to

```
Basic = ^#+(.+)\n*((?:\n(?:^[^\n#].{0,2}$|^[^\n#].{3}(?&lt;!&lt;!--).*))+)
```

1. Save the config file
2. Run the script on the file, with 'Regex' checked: ![[GUI_regex.webp]]

### All Users

1. You should see these cards in Anki: ![[Header_1.webp]]
   ![[Header_2.webp]]
   ![[Header_3.webp]]
   ![[Header_4.webp]]

#### Subheader Paragraph Style

If you'd like the effect of the header paragraph style, but only want it to add cards below a certain subheading level (e.g. 3 # or more), use the following regex:

- 2 or more - `^#{2,}(.+)\n*((?:\n(?:^[^\n#].{0,2}$|^[^\n#].{3}(?<!<!--).*))+)`
- 3 or more - `^#{3,}(.+)\n*((?:\n(?:^[^\n#].{0,2}$|^[^\n#].{3}(?<!<!--).*))+)`
- n or more - `^#{n,}(.+)\n*((?:\n(?:^[^\n#].{0,2}$|^[^\n#].{3}(?<!<!--).*))+)`, where you replace `{n,}` with the value of the number n. E.g. if n was 4, it would read `^#{4,}(.+)\n*((?:\n(?:^[^\n#].{0,2}$|^[^\n#].{3}(?<!<!--).*))+)`
