---
title:
  - 25 Question Answer Style
aliases:
  - 25 Question Answer Style
  - Question Answer Style
  - question_answer_style
  - obsidian_to_anki_question_answer_style
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Question-answer-style
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags: 
---
# Question-Answer Style

## Usage

**[[03 Regex|Regex]] line:** `^Q: ((?:.+\n)*)\n*A: (.+(?:\n(?:^.{1,3}$|^.{4}(?<!<!--).*))*)`

**Example usage:**

1. Create a file called `test.md`
2. Paste the following contents into the file:

```
Q: How do you use this style?
A: Just like this.

Q: Can the question
run over multiple lines?
A: Yes, and
So can the answer

Q: Does the answer need to be immediately after the question?


A: No, and preceding whitespace will be ignored.

Q: How is this possible?
A: The 'magic' of regular expressions!
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
Basic = ^Q: ((?:.+\n)*)\n*A: (.+(?:\n(?:^.{1,3}$|^.{4}(?&lt;!&lt;!--).*))*)
```

1. Save the config file
2. Run the script on the file, with 'Regex' checked: ![[GUI_regex.webp]]

### All Users

1. You should see these cards in Anki: ![[Question_1.webp]]  
   ![[Question_2.webp]]  
   ![[Question_3.webp]]  
   ![[Question_4.webp]]
