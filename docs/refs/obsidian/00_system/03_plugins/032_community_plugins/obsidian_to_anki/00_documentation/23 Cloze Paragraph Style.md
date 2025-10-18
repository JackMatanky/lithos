---
title:
  - 13 Cloze Paragraph Style
aliases:
  - 13 Cloze Paragraph Style
  - Cloze Paragraph Style
  - cloze_paragraph_style
  - obsidian_to_anki_cloze_paragraph_style
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Cloze-Paragraph-style
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Cloze Paragraph Style

## Usage

**[[03 Regex|Regex]] line:** `((?:.+\n)*(?:.*{.*)(?:\n(?:^.{1,3}$|^.{4}(?<!<!--).*))*)`

1. Create a file called `test.md`.
2. Paste the following contents into the file:

```
The idea of {cloze paragraph style} is to be able to recognise any paragraphs that contain {cloze deletions}.

The script should ignore paragraphs that have math formatting like $\frac{3}{4}$ but no actual cloze deletions.

With {2:CurlyCloze} enabled, you can also use the {c1|easier cloze formatting},
but of course {{c3::Anki}}'s formatting is always an option.
```

### Obsidian Plugin Users

1. In the plugin settings, paste the Regex line into the 'Custom Regexps' field associated with 'Cloze'
2. Ensure that the 'Regex' and 'CurlyCloze' options are checked
3. Click the Anki icon on the ribbon to run the plugin

### Python Script

1. Run the script, and check '[[04 Config|Config]]' to open up the config file:  
   ![[GUI_config.webp]]
   
2. Navigate to the "Custom Regexps" section
3. Change the line

   ```
   Cloze =  
   ```

   to  
   `Cloze = (.*{.*\n?)`  

4. Also set `CurlyCloze = True` to have the above example work properly.
5. Save the config file
6. Run the script on the file, with 'Regex' checked:  
   ![[GUI_regex.webp]]

### All Users

1. You should see these cards in Anki:  
    ![[Cloze_1.webp]]  
    ![[Cloze_2.webp]]

#### Highlight-cloze Style (Obsidian Plugin only)

**[[03 Regex|Regex]] line:**: `((?:.+\n)*(?:.*==.*)(?:\n(?:^.{1,3}$|^.{4}(?<!<!--).*))*)`

You can also use markdown highlights instead of curly braces in order to indicate cloze deletions - just use the above style, and also enable 'CurlyCloze - Highlights to Clozes'!
