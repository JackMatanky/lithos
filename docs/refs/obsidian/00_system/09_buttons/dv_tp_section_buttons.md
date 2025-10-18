---
title: dv_tp_section_buttons
aliases:
  - Dataview Templater Section Buttons
  - buttons_dv_tp_section
plugin: buttons
language:
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-25T11:06
date_modified: 2023-10-25T16:22
tags: obsidian/buttons, obsidian/templater, obsidian/dataview
---
# Dataview Templater Section Buttons

- Colors
	- purple
	- blue
	- green
	- orange
	- yellow
	- pink
	- red

## Tasks and Events Section

### General

```button
name ✅Related Tasks and Events
type append template
action 100_40_dvmd_related_task_sect
replace [1, 2]
color blue
```
^button-related-task-event

- Inline: `button-related-task-event`

```meta-bind-button
label: ✅Related Tasks and Events
icon: ""
hidden: false
class: mb_button_blue
tooltip: Replace the tasks and events section with MD table of linked files and a filtered DataView table
id: button-related-task-event
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/100_40_dvmd_related_task_sect.md
    templater: true
```

- Inline: `BUTTON[button-related-task-event]`


### Project

#### General

```button
name 🏗️Project Tasks and Events
type append template
action 140_00_dvmd_task_sect_proj
replace [1, 2]
color blue
```
^button-related-task-event-project

- Inline: `button-related-task-event-project`

```meta-bind-button
label: 🏗️Project Tasks and Events
icon: ""
hidden: false
class: mb_button_blue
tooltip: Replace the project tasks and events section with MD table of linked files and a filtered DataView table
id: button-related-task-event-project
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/140_00_dvmd_task_sect_proj.md
    templater: true
```

- Inline: `BUTTON[button-related-task-event-project]`

#### Habits and Rituals

```button
name 🏗️Project Habits and Rituals🤖
type append template
action 140_50_dvmd_task_sect_proj_habit_rit
replace [1, 2]
color blue
```
^button-related-task-event-project-hab-rit

- Inline: `button-related-task-event-project-hab-rit`

```meta-bind-button
label: 🏗️Project Habits and Rituals🤖
icon: ""
hidden: false
class: mb_button_blue
tooltip: Replace the project tasks and events section with MD table of linked files and a filtered DataView table
id: button-related-task-event-project-hab-rit
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/140_50_dvmd_task_sect_proj_habit_rit.md
    templater: true
```

- Inline: `BUTTON[button-related-task-event-project-hab-rit]`

### Parent Task

```button
name ⚒️Parent Task Tasks and Events
type append template
action 141_00_dvmd_task_sect_parent
replace [1, 2]
color blue
```
^button-related-task-event-parent

- Inline: `button-related-task-event-parent`

```meta-bind-button
label: ⚒️Parent Task Tasks and Events
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Replace the parent task's tasks and events section with MD table of linked files and a filtered DataView table"
id: button-related-task-event-parent
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/141_00_dvmd_task_sect_parent.md
    templater: true
```

- Inline: `BUTTON[button-related-task-event-parent]`

### Child Task

```button
name 🔨Child Task Tasks and Events🤝
type append template
action 142_00_dvmd_task_sect_child
replace [1, 2]
color blue
```
^button-related-task-event-child

- Inline: `button-related-task-event-child`

```meta-bind-button
label: 🔨Child Task Tasks and Events🤝
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Replace the child task's tasks and events section with MD table of linked files and a filtered DataView table"
id: button-related-task-event-child
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/142_00_dvmd_task_sect_child.md
    templater: true
```

- Inline: `BUTTON[button-related-task-event-child]`

## Directory Section

```button
name 📇Related Directory Files
type append template
action 100_50_dvmd_related_dir_sect
replace [1, 2]
color pink
```
^button-related-dir

- Inline: `button-related-dir`

```meta-bind-button
label: 📇Related Directory Files
icon: ""
hidden: false
class: mb_button_pink
tooltip: Replace the directory section with MD table of linked files and a filtered DataView table
id: button-related-dir
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/100_50_dvmd_related_dir_sect.md
    templater: true
```

- Inline: `BUTTON[button-related-dir]`

## Library Section

```button
name 🏫Related Library Content
type append template
action 100_60_dvmd_related_lib_sect
replace [1, 2]
color green
```
^button-related-lib

- Inline: `button-related-lib`

```meta-bind-button
label: 🏫Related Library Content
icon: ""
hidden: false
class: mb_button_green
tooltip: Replace the directory section with MD table of linked files and a filtered DataView table
id: button-related-lib
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/100_60_dvmd_related_lib_sect.md
    templater: true
```

- Inline: `BUTTON[button-related-lib]`

## PKM Section

### General

```button
name 🗃️Related PKM Files
type append template
action 100_70_dvmd_related_pkm_sect
replace [1, 2]
color purple
```
^button-related-pkm

- Inline: `button-related-pkm`

```meta-bind-button
label: 🗃️Related PKM Files
icon: ""
hidden: false
class: mb_button_purple
tooltip: Replace the PKM section with MD table of linked files and a filtered DataView table
id: button-related-pkm
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/100_70_dvmd_related_pkm_sect.md
    templater: true
```

- Inline: `BUTTON[button-related-pkm]`

### Tree Info

```button
name 🗃️PKM Tree Info
type append template
action 170_00_dvmd_pkm_tree_info
replace [1, 2]
color purple
```
^button-tree-info

- Inline: `button-tree-info`

```meta-bind-button
label: 🗃️PKM Tree Info
icon: ""
hidden: false
class: mb_button_purple
tooltip: Replace the PKM tree information section with MD table of linked files and a filtered DataView table
id: button-pkm-tree-info
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/170_00_dvmd_pkm_tree_info.md
    templater: true
```

- Inline: `BUTTON[button-pkm-tree-info]`

### Tree Context

```button
name 🗃️PKM Tree Context
type append template
action 170_01_dvmd_pkm_tree_context
replace [1, 2]
color purple
```
^button-tree-context

- Inline: `button-tree-context`

```meta-bind-button
label: 🗃️PKM Tree Context
icon: ""
hidden: false
class: mb_button_purple
tooltip: Replace the PKM tree context section with MD table of linked files and a filtered DataView table
id: button-pkm-tree-context
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/170_01_dvmd_pkm_tree_context.md
    templater: true
```

- Inline: `BUTTON[button-pkm-tree-context]`

## Notes Section

```button
name 🗃️Related Notes
type append template
action 100_71_dvmd_related_note_sect
replace [1, 2]
color purple
```
^button-related-note

- Inline: `button-related-note`

```meta-bind-button
label: 🗃️Related Notes
icon: ""
hidden: false
class: mb_button_purple
tooltip: Replace the PKM notes section with MD table of linked files and a filtered DataView table
id: button-related-pkm-note
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/05_templates/100_71_dvmd_related_note_sect.md
    templater: true
```

- Inline: `BUTTON[button-related-pkm-note]`

---

## Use Cases

1. [[32_00_week|Weekly Calendar Template]]
2. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]
4. [[55_21_daily_morn_rit|Daily Morning Ritual Task Template]]
5. [[55_22_today_morn_rit|Today's Morning Ritual Task Template]]
6. [[55_23_tomorrow_morn_rit|Tomorrow's Morning Ritual Task Template]]
