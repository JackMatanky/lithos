---
title: task_event_buttons
aliases:
  - Task and Event Buttons
  - buttons_task_event
plugin: buttons
language:
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-30T09:57
date_modified: 2023-10-25T16:22
tags: obsidian/buttons
---
# Task and Event Buttons

- Colors
	- purple
	- blue
	- green
	- orange
	- yellow
	- pink
	- red

## Project Templates

### General

```button
name 🏗️Project
type note(Untitled, split) template
action 40_00_project
color green
```
^button-project

- Inline: `button-project`

```meta-bind-button
label: 🏗️Project
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-project
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/40_00_project.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-project]`

### Short

```button
name 🏗️Project
type note(Untitled, split) template
action 40_00_project
color blue
```
^button-project-short

- Inline: `button-project-short`

```meta-bind-button
label: 🏗️Project
icon: ""
hidden: false
class: mb_button_blue
tooltip: ""
id: button-project-short
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/40_00_project.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-project-short]`

### Task Table

```button
name 🏗️Project
type note(Untitled, split) template
action 40_00_project
color green
```
^button-project-task-table

- Inline: `button-project-task-table`

```meta-bind-button
label: 🏗️Project
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-project-task-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/40_00_project.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-project-task-table]`

## Parent Task Templates

### General

```button
name ⚒️Parent Task
type note(Untitled, split) template
action 41_00_parent_task
customColor #ff693f
customTextColor #18191e
```
^button-parent

- Inline: `button-parent`

```meta-bind-button
label: ⚒️Parent Task
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-parent
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/41_00_parent_task.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-parent]`

### Short

```button
name ⚒️Parent
type note(Untitled, split) template
action 41_00_parent_task
customColor #ff693f
customTextColor #18191e
```
^button-parent-short

- Inline: `button-parent-short`

```meta-bind-button
label: ⚒️Parent
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-parent-short
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/41_00_parent_task.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-parent-short]`

### Task Table

```button
name ⚒️Parent
type note(Untitled, split) template
action 41_00_parent_task
customColor #ff693f
customTextColor #18191e
```
^button-parent-task-table

- Inline: `button-parent-task-table`

```meta-bind-button
label: ⚒️Parent
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-parent-task-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/41_00_parent_task.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-parent-task-table]`

### Job Application

#### General

```button
name ⚒️Job Application💼
type note(Untitled, split) template
action 41_31_par_job_app
color orange
```
^button-parent-job

- Inline: `button-parent-job`

```meta-bind-button
label: ⚒️Job Application💼
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-parent-job
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/41_31_par_job_app.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-parent-job]`

#### Short

```button
name ⚒️Job App.💼
type note(Untitled, split) template
action 41_31_par_job_app
color orange
```
^button-parent-job-short

- Inline: `button-parent-job-short`

```meta-bind-button
label: ⚒️Job App.💼
icon: ""
hidden: false
class: mb_button_orange
tooltip: ""
id: button-parent-job-short
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/41_31_par_job_app.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-parent-job-short]`

## Action Items Templates

### General

```button
name 🔨Action Item
type note(Untitled, split) template
action 42_00_action_item
color yellow
```
^button-action-item

- Inline: `button-action-item`

```meta-bind-button
label: 🔨Action Item
icon: ""
hidden: false
class: mb_button_green
tooltip: ""
id: button-action-item
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/42_00_action_item.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-action-item]`

### Short

```button
name 🔨Action
type note(Untitled, split) template
action 42_00_action_item
color blue
```
^button-action-item-short

- Inline: `button-action-item-short`

```meta-bind-button
label: 🔨Action
icon: ""
hidden: false
class: mb_button_blue
tooltip: ""
id: button-action-item-short
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/42_00_action_item.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-action-item-short]`

### Task Table

```button
name 🔨Task
type note(Untitled, split) template
action 42_00_action_item
color yellow
```
^button-action-item-task-table

- Inline: `button-action-item-task-table`

```meta-bind-button
label: 🔨Task
icon: ""
hidden: false
class: mb_button_yellow
tooltip: ""
id: button-action-item-task-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/42_00_action_item.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-action-item-task-table]`

## Meeting Templates

### General Meeting

```button
name 🫱🏼‍🫲🏽General Event
type note(Untitled, split) template
action 43_00_event
color yellow
```
^button-meeting

- Inline: `button-meeting`

```meta-bind-button
label: 🫱🏼‍🫲🏽General Event
icon: ""
hidden: false
class: mb_button_yellow
tooltip: ""
id: button-meeting
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/43_00_event.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-meeting]`

### Short

```button
name 🤝Event
type note(Untitled, split) template
action 43_00_event
color blue
```
^button-meeting-short

- Inline: `button-meeting-short`

```meta-bind-button
label: 🤝Event
icon: ""
hidden: false
class: mb_button_blue
tooltip: ""
id: button-meeting-short
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/43_00_event.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-meeting-short]`

### Task Table

```button
name 🫱🏼‍🫲🏽Event
type note(Untitled, split) template
action 43_00_event
color yellow
```
^button-meeting-task-table

- Inline: `button-meeting-task-table`

```meta-bind-button
label: 🤝Event
icon: ""
hidden: false
class: mb_button_yellow
tooltip: ""
id: button-meeting-task-table
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/07_templates/43_00_event.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-meeting-task-table]`

---

## Use Cases

### Daily Note

| **Tasks & Events**   | `button-project-task-table` | `button-parent-task-table` | `button-action-item-task-table` | `button-meeting-task-table` |
| -------------------- | --------------------------- | ------------------------------- | ------------------------------- | --------------------------- |
| `button-habit-today` | `button-morn-rit-today`     | `button-work-start-today`       | `button-work-shut-today`        | `button-eve-rit-today`      |

### Week Files

| Weekly | `button-habit-week` | `button-morn-work-start-week` | `button-eve-work-shut-week` |
| ------ | ------------------- | ----------------------------- | --------------------------- |

| **Daily** | `button-habit-daily` | `button-morn-rit-daily` | `button-work-start-daily` | `button-work-shut-daily` | `button-eve-rit-daily` |
| --------- | -------------------- | ----------------------- | ------------------------- | ------------------------ | ---------------------- |

### Tasks and Events

> [!task] Tasks and Events
> 
> `button-project-task-table`|`button-parent-task-table`|`button-action-item-task-table`|`button-meeting-task-table`