---
title: pdev_buttons
aliases:
  - Journal Buttons
  - buttons_journal
plugin: buttons
language:
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-30T09:57
date_modified: 2024-09-26T16:37
tags: obsidian/buttons
---
# Journal Buttons

- Colors
	- purple
	- blue
	- green
	- orange
	- yellow
	- pink
	- red

## Daily Journals

```button
name 🕯️Daily Journals
type note(Untitled, split) template
action 90_01_journals_daily_preset
color purple
```
^button-journal-daily

- Inline: `button-journal-daily`

```meta-bind-button
label: 🕯️Daily Journals
icon: ""
hidden: false
class: mb_button_purple
tooltip: "Includes haily reflection, gratitude, and detachment journals"
id: "button-journal-daily"
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/90_01_journals_daily_preset.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-journal-daily]`

## Reflection Journal Templates

### Daily Reflection

#### General

```button
name Daily Reflection
type note(Untitled, split) template
action 95_11_daily_reflection_today
color purple
```
^button-reflection-daily

- Inline: `button-reflection-daily`

```meta-bind-button
label: Daily Reflection
icon: ""
hidden: false
class: mb_button_purple
tooltip: ""
id: button-reflection-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/95_11_daily_reflection_today.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-reflection-daily]`

#### Preset by Date

```button
name Daily Reflection
type note(Untitled, split) template
action 95_12_daily_reflection_today_preset
color purple
```
^button-reflection-daily-preset

- Inline: `button-reflection-daily-preset`

```meta-bind-button
label: Daily Reflection
icon: ""
hidden: false
class: mb_button_purple
tooltip: ""
id: button-reflection-daily-preset
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/95_12_daily_reflection_today_preset.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-reflection-daily-preset]`

### Weekly Reflection

```button
name Weekly Reflection
type note(Untitled, split) template
action 95_20_weekly_reflection
color purple
```
^button-reflection-weekly

- Inline: `button-reflection-weekly`

```meta-bind-button
label: Weekly Reflection
icon: ""
hidden: false
class: mb_button_purple
tooltip: ""
id: button-reflection-weekly
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/95_20_weekly_reflection.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-reflection-weekly]`

## Gratitude Journal Templates

### General

```button
name Daily Gratitude
type note(Untitled, split) template
action 96_10_daily_gratitude_today
color purple
```
^button-gratitude-daily

- Inline: `button-gratitude-daily`

```meta-bind-button
label: Daily Gratitude
icon: ""
hidden: false
class: mb_button_purple
tooltip: ""
id: button-gratitude-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/96_10_daily_gratitude_today.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-gratitude-daily]`

### Preset by Date

```button
name Daily Gratitude
type note(Untitled, split) template
action 96_11_daily_gratitude_today_preset
color purple
```
^button-gratitude-daily-preset

- Inline: `button-gratitude-daily-preset`

```meta-bind-button
label: Daily Gratitude
icon: ""
hidden: false
class: mb_button_purple
tooltip: ""
id: button-gratitude-daily-preset
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/96_11_daily_gratitude_today_preset.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-gratitude-daily-preset]`

## Detachment Journal Templates

### General

```button
name Daily Detachment
type note(Untitled, split) template
action 97_10_daily_detachment_today
color purple
```
^button-detachment-daily

- Inline: `button-detachment-daily`

```meta-bind-button
label: Daily Detachment
icon: ""
hidden: false
class: mb_button_purple
tooltip: ""
id: button-detachment-daily
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/97_10_daily_detachment_today.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-detachment-daily]`

### Preset by Date

```button
name Daily Detachment
type note(Untitled, split) template
action 97_11_daily_detachment_today_preset
color purple
```
^button-detachment-daily-preset

- Inline: `button-detachment-daily-preset`

```meta-bind-button
label: Daily Detachment
icon: ""
hidden: false
class: mb_button_purple
tooltip: ""
id: button-detachment-daily-preset
style: primary
actions:
  - type: templaterCreateNote
    templateFile: 00_system/05_templates/97_11_daily_detachment_today_preset.md
    folderPath: /
    fileName: ""
    openNote: true
```

- Inline: `BUTTON[button-detachment-daily-preset]`

---

## Use Cases
