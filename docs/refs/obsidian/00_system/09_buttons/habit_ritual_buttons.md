---
title: habit_ritual_buttons
aliases:
  - Habit and Ritual Buttons
  - buttons_habit_ritual
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
# Habit and Ritual Buttons

- Colors
	- purple
	- blue
	- green
	- orange
	- yellow
	- pink
	- red

## Habit Templates

### Daily

```button
name 🦿Habits
type note(Untitled, split) template
action 45_01_daily_bidaily_habit
color green
```
^button-habit-daily

- Inline: `button-habit-daily`

```meta-bind-button
label: 🦿Habits
icon: ""
hidden: false
class: mb_button_green
tooltip: Create the daily bi-daily habits note
id: button-habit-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/45_01_daily_bidaily_habit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-habit-daily]`

### Today

```button
name 🦿Habits
type note(Untitled, split) template
action 45_02_today_bidaily_habit
color blue
```
^button-habit-today

- Inline: `button-habit-today`

```meta-bind-button
label: 🦿Habits
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create today's bi-daily habits note"
id: button-habit-today
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/45_02_today_bidaily_habit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-habit-today]`

### Tomorrow

```button
name 🦿Habits
type note(Untitled, split) template
action 45_03_tomorrow_bidaily_habit
color blue
```
^button-habit-tomorrow

- Inline: `button-habit-tomorrow`

```meta-bind-button
label: 🦿Habits
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create tomorrow's bi-daily habits note"
id: button-habit-tomorrow
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/45_03_tomorrow_bidaily_habit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-habit-tomorrow]`

## Morning Ritual Templates

### Daily

```button
name 🍵Morning Rituals
type note(Untitled, split) template
action 46_01_daily_morn_rit
color green
```
^button-morn-rit-daily

- Inline: `button-morn-rit-daily`

```meta-bind-button
label: 🍵Morning Rituals
icon: ""
hidden: false
class: mb_button_green
tooltip: Create the daily morning rituals note
id: button-morn-rit-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/46_01_daily_morn_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-morn-rit-daily]`

### Today

```button
name 🍵Morning Rituals
type note(Untitled, split) template
action 46_02_today_morn_rit
color blue
```
^button-morn-rit-today

- Inline: `button-morn-rit-today`

```meta-bind-button
label: 🍵Morning Rituals
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create today's morning rituals note"
id: button-morn-rit-today
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/46_02_today_morn_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-morn-rit-today]`

### Tomorrow

```button
name 🍵Morning Rituals
type note(Untitled, split) template
action 46_03_tomorrow_morn_rit
color blue
```
^button-morn-rit-tomorrow

- Inline: `button-morn-rit-tomorrow`

```meta-bind-button
label: 🍵Morning Rituals
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create tomorrow's bi-daily habits note"
id: button-morn-rit-tomorrow
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/46_03_tomorrow_morn_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-morn-rit-tomorrow]`

## Workday Startup Ritual Templates

### Daily

```button
name 🌇Workday Startup
type note(Untitled, split) template
action 47_01_daily_work_start_rit
color green
```
^button-work-start-daily

- Inline: `button-work-start-daily`

```meta-bind-button
label: 🌇Workday Startup
icon: ""
hidden: false
class: mb_button_green
tooltip: Create the daily workday startup rituals note
id: button-work-start-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/47_01_daily_work_start_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-work-start-daily]`

### Today

```button
name 🌇Workday Startup
type note(Untitled, split) template
action 47_02_today_work_start_rit
color blue
```
^button-work-start-today

- Inline: `button-work-start-today`

```meta-bind-button
label: 🌇Workday Startup
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create today's workday startup rituals note"
id: button-work-start-today
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/47_02_today_work_start_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-work-start-today]`

### Tomorrow

```button
name 🌇Workday Startup
type note(Untitled, split) template
action 47_03_tomorrow_work_start_rit
color blue
```
^button-work-start-tomorrow

- Inline: `button-work-start-tomorrow`

```meta-bind-button
label: 🌇Workday Startup
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create tomorrow's bi-daily habits note"
id: button-work-start-tomorrow
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/47_03_tomorrow_work_start_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-work-start-tomorrow]`

## Workday Shutdown Ritual Templates

### Daily

```button
name 🌆Workday Shutdown
type note(Untitled, split) template
action 48_01_daily_work_shut_rit
color green
```
^button-work-shut-daily

- Inline: `button-work-shut-daily`

```meta-bind-button
label: 🌆Workday Shutdown
icon: ""
hidden: false
class: mb_button_green
tooltip: Create the daily workday shutdown rituals note
id: button-work-shut-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/48_01_daily_work_shut_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-work-shut-daily]`

### Today

```button
name 🌆Workday Shutdown
type note(Untitled, split) template
action 48_02_today_work_shut_rit
color blue
```
^button-work-shut-today

- Inline: `button-work-shut-today`

```meta-bind-button
label: 🌆Workday Shutdown
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create today's workday shutdown rituals note"
id: button-work-shut-today
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/48_02_today_work_shut_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-work-shut-today]`

### Tomorrow

```button
name 🌆Workday Shutdown
type note(Untitled, split) template
action 48_03_tomorrow_work_shut_rit
color blue
```
^button-work-shut-tomorrow

- Inline: `button-work-shut-tomorrow`

```meta-bind-button
label: 🌆Workday Shutdown
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create tomorrow's workday shutdown rituals note"
id: button-work-shut-tomorrow
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/48_03_tomorrow_work_shut_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-work-shut-tomorrow]`

## Evening Ritual Templates

### Daily

```button
name 🛏️Evening Rituals
type note(Untitled, split) template
action 49_01_daily_eve_rit
color green
```
^button-eve-rit-daily

- Inline: `button-eve-rit-daily`

```meta-bind-button
label: 🛏️Evening Rituals
icon: ""
hidden: false
class: mb_button_green
tooltip: Create the daily evening rituals note
id: button-eve-rit-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/49_01_daily_eve_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-eve-rit-daily]`

### Today

```button
name 🛏️Evening Rituals
type note(Untitled, split) template
action 49_02_today_eve_rit
color blue
```
^button-eve-rit-today

- Inline: `button-eve-rit-today`

```meta-bind-button
label: 🛏️Evening Rituals
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create today's evening rituals note"
id: button-eve-rit-today
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/49_02_today_eve_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-eve-rit-today]`

### Tomorrow

```button
name 🛏️Evening Rituals
type note(Untitled, split) template
action 49_03_tomorrow_eve_rit
color blue
```
^button-eve-rit-tomorrow

- Inline: `button-eve-rit-tomorrow`

```meta-bind-button
label: 🛏️Evening Rituals
icon: ""
hidden: false
class: mb_button_blue
tooltip: "Create tomorrow's evening rituals note"
id: button-eve-rit-tomorrow
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/49_03_tomorrow_eve_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-eve-rit-tomorrow]`

## Weekly Habits and Rituals Templates

### General

```button
name 🦿Weekly Habits
type note(Untitled, split) template
action 44_01_week_bidaily_habit
color blue
```
^button-habit-week

- Inline: `button-habit-week`

```meta-bind-button
label: 🦿Weekly Habits
icon: ""
hidden: false
class: mb_button_blue
tooltip: Create the weekly bi-daily habits notes
id: button-habit-week
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/44_01_week_bidaily_habit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-habit-week]`

### General

```button
name 🍵Morning and Workday Startup🌇
type note(Untitled, split) template
action 44_02_week_morn_work_start_rit
color blue
```
^button-morn-work-start-week

- Inline: `button-morn-work-start-week`

```meta-bind-button
label: 🍵Morning and Workday Startup🌇
icon: ""
hidden: false
class: mb_button_blue
tooltip: Create the weekly morning and workday startup rituals notes
id: button-morn-work-start-week
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/44_02_week_morn_work_start_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-morn-work-start-week]`

### General

```button
name 🌆Workday Shutdown and Evening🛏️
type note(Untitled, split) template
action 44_03_week_work_shut_eve_rit
color blue
```
^button-eve-work-shut-week

- Inline: `button-eve-work-shut-week`

```meta-bind-button
label: 🌆Workday Shutdown and Evening🛏️
icon: ""
hidden: false
class: mb_button_blue
tooltip: Create the weekly workday shutdown and evening rituals notes
id: button-eve-work-shut-week
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/44_03_week_work_shut_eve_rit.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-eve-work-shut-week]`

---

## Use Cases

### Week Files

| Weekly | `button-habit-week` | `button-morn-work-start-week` | `button-eve-work-shut-week` |
| ------ | ------------------- | ----------------------------- | --------------------------- |

| **Daily** | `button-habit-daily` | `button-morn-rit-daily` | `button-work-start-daily` | `button-work-shut-daily` | `button-eve-rit-daily` |
| --------- | -------------------- | ----------------------- | ------------------------- | ------------------------ | ---------------------- |

### Day Files