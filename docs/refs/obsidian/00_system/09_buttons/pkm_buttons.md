---
title: pkm_buttons
aliases:
  - PKM Buttons
  - buttons_pkm
plugin: buttons
language:
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-30T09:57
date_modified: 2023-10-25T16:22
tags: obsidian/buttons, zettelkasten
---
# Note Buttons

- Colors
	- purple
	- blue
	- green
	- orange
	- yellow
	- pink
	- red

## Literature Note Templates

### Question

#### General

```button
name ❔Question
type note(Untitled, split) template
action 80_31_note_question
color green
```
^button-pkm-question

- Inline: `button-pkm-question`

```meta-bind-button
label: ❔Question
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-question
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_31_note_question.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-question]`

#### PKM Table

```button
name ❔Question
type note(Untitled, split) template
action 80_31_note_question
color blue
```
^button-pkm-question-table

- Inline: `button-pkm-question-table`

```meta-bind-button
label: ❔Question
icon: ""
hidden: false
class: mb_button_blue
tooltip: ""
id: button-pkm-question-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_31_note_question.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-question-table]`

### Evidence

#### General

```button
name ⚖️Evidence
type note(Untitled, split) template
action 80_32_note_evidence
color green
```
^button-pkm-evidence

- Inline: `button-pkm-evidence`

```meta-bind-button
label: ⚖️Evidence
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-evidence
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_32_note_evidence.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-evidence]`

#### PKM Table

```button
name ⚖️Proof
type note(Untitled, split) template
action 80_32_note_evidence
color blue
```
^button-pkm-evidence-table

- Inline: `button-pkm-evidence-table`

```meta-bind-button
label: ⚖️Proof
icon: ""
hidden: false
class: mb_button_blue
tooltip: ""
id: button-pkm-evidence-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_32_note_evidence.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-evidence-table]`

### Steps

#### General

```button
name 🪜Steps
type note(Untitled, split) template
action 80_42_note_steps
color green
```
^button-pkm-steps

- Inline: `button-pkm-steps`

```meta-bind-button
label: 🪜Steps
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-steps
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_42_note_steps.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-steps]`

#### PKM Table

```button
name 🪜Step
type note(Untitled, split) template
action 80_42_note_steps
color blue
```
^button-pkm-steps-table

- Inline: `button-pkm-steps-table`

```meta-bind-button
label: 🪜Step
icon: ""
hidden: false
class: mb_button_blue
tooltip: ""
id: button-pkm-steps-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_42_note_steps.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-steps-table]`

### Conclusion

#### General

```button
name 🎱Conclusion
type note(Untitled, split) template
action 80_33_note_conclusion
color green
```
^button-pkm-conclusion

- Inline: `button-pkm-conclusion`

```meta-bind-button
label: 🎱Conclusion
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-conclusion
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_33_note_conclusion.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-conclusion]`

#### PKM Table

```button
name 🎱Answer
type note(Untitled, split) template
action 80_33_note_conclusion
color blue
```
^button-pkm-conclusion-table

- Inline: `button-pkm-conclusion-table`

```meta-bind-button
label: 🎱Answer
icon: ""
hidden: false
class: mb_button_blue
tooltip: ""
id: button-pkm-conclusion-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_33_note_conclusion.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-conclusion-table]`

## Fleeting Note Templates

### Quick Summary

#### General

```button
name 📝Summary
type note(Untitled, split) template
action 80_12_note_summary
color green
```
^button-pkm-summary

- Inline: `button-pkm-summary`

```meta-bind-button
label: 📝Summary
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-summary
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_12_note_summary.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-summary]`

#### PKM Table

```button
name 📝Summary
type note(Untitled, split) template
action 80_12_note_summary
color green
```
^button-pkm-summary-table

- Inline: `button-pkm-summary-table`

```meta-bind-button
label: ❔Question
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-summary-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_12_note_summary.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-summary-table]`

### Quote

#### General

```button
name ⏺️Quote
type note(Untitled, split) template
action 80_13_note_quote
color green
```
^button-pkm-quote

- Inline: `button-pkm-quote`

```meta-bind-button
label: ⏺️Quote
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-quote
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_13_note_quote.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-quote]`

#### PKM Table

```button
name ⏺️Quote
type note(Untitled, split) template
action 80_13_note_quote
color green
```
^button-pkm-quote-table

- Inline: `button-pkm-quote-table`

```meta-bind-button
label: ⏺️Quote
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-quote-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_13_note_quote.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-quote-table]`

### Idea

#### General

```button
name 💭Idea
type note(Untitled, split) template
action 80_11_note_idea
color green
```
^button-pkm-idea

- Inline: `button-pkm-idea`

```meta-bind-button
label: 💭Thought
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-idea
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_11_note_idea.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-idea]`

#### PKM Table

```button
name 💭Idea
type note(Untitled, split) template
action 80_11_note_idea
color green
```
^button-pkm-idea-table

- Inline: `button-pkm-idea-table`

```meta-bind-button
label: 💭Idea
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-pkm-idea-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_11_note_idea.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-idea-table]`

## Info Note Templates

### Concept

#### General

```button
name 🎞️Concept
type note(Untitled, split) template
action 80_51_note_concept
customColor #ff693f
customTextColor #18191e
```
^button-pkm-concept

- Inline: `button-pkm-concept`

```meta-bind-button
label: 🎞️Concept
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-pkm-concept
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_51_note_concept.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-concept]`

#### PKM Table

```button
name 🎞️Concept
type note(Untitled, split) template
action 80_51_note_concept
customColor #ff693f
customTextColor #18191e
```
^button-pkm-concept-table

- Inline: `button-pkm-concept-table`

```meta-bind-button
label: ❔Question
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-pkm-concept-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_51_note_concept.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-concept-table]`

### Definition

#### General

```button
name 🪟Definition
type note(Untitled, split) template
action 80_52_note_definition
customColor #ff693f
customTextColor #18191e
```
^button-pkm-definition

- Inline: `button-pkm-definition`

```meta-bind-button
label: 🪟Definition
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-pkm-definition
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_52_note_definition.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-definition]`

#### PKM Table

```button
name 🪟Definition
type note(Untitled, split) template
action 80_52_note_definition
customColor #ff693f
customTextColor #18191e
```
^button-pkm-definition-table

- Inline: `button-pkm-definition-table`

```meta-bind-button
label: 🪟Definition
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-pkm-definition-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/80_52_note_definition.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-pkm-definition-table]`

---

## Use Cases

| `button-action-item`     | `button-meeting`         |
| ------------------------ | ------------------------ |
| `button-pkm-concept`    | `button-pkm-definition` |
| `button-pkm-quote`      | `button-pkm-idea`       |
| `button-pkm-question`   | `button-pkm-problem`    |
| `button-pkm-evidence`   | `button-pkm-steps`      |
| `button-pkm-conclusion` | `button-pkm-answer`     |

| INFO                     | FLEETING            | `button-pkm-question`   | `button-pkm-problem` |
| ------------------------ | ------------------- | ------------------------ | --------------------- |
| `button-pkm-concept`    | `button-pkm-idea`  | `button-pkm-evidence`   | `button-pkm-steps`   |
| `button-pkm-definition` | `button-pkm-quote` | `button-pkm-conclusion` | `button-pkm-answer`  |
