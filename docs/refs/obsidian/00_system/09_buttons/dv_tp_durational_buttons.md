---
title: dv_tp_durational_buttons
aliases:
  - Dataview Templater Durational Buttons
  - buttons_dv_tp_durational
plugin: buttons
language:
module: 
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
tags: obsidian/buttons, obsidian/templater, obsidian/dataview
---
# Dataview Templater Durational Buttons

- Colors
	- purple
	- blue
	- green
	- orange
	- yellow
	- pink
	- red

## Day File

```button
name 📆Day MD File
type append template
action 111_00_dvmd_day_file
replace [44, 616]
color purple
```
^button-day-md

- Inline: `button-day-md`

## Daily Tasks Due Table

### General

```button
name Tasks Due Today
type append template
action 111_40_dvmd_day_tasks_due
replace [67, 104]
color yellow
```
^button-tasks-due-daily

- Inline: `button-tasks-due-daily`

### Morning Ritual

```button
name Daily Task Schedule
type append template
action 111_40_dvmd_day_tasks_due
replace [67, 104]
color yellow
```
^button-tasks-due-daily-morn-rit

- Inline: `button-tasks-due-daily-morn-rit`

## Daily Tasks Completed Table

### General

```button
name ✅Daily Schedule Review
type append template
action 111_42_dvmd_day_tasks_done
replace [67, 104]
color yellow
```
^button-tasks-done-daily

- Inline: `button-tasks-done-daily`

### Workday Shutdown Ritual

#### Button

- Inline: `button-dvjs-md-task-table-morn-rit`

## Weekly Tasks and Events

### Tasks and Events Due

```button
name ✅Active Tasks and Events
type append template
action 112_40_dvmd_week_tasks_due
replace [1475, 1836]
color blue
```
^button-tasks-due-weekly

- Inline: `button-tasks-due-weekly`

### Tasks and Events Completed

```button
name ✅Completed Tasks and Events
type append template
action 112_41_dvmd_week_tasks_done
replace [1475, 1836]
color blue
```
^button-tasks-done-weekly

- Inline: `button-tasks-done-weekly`

## Weekly Habits and Rituals

### Habits and Rituals Due

```button
name Weekly Planned Habits and Rituals
type append template
action 112_45_dvmd_week_habit_rit_due
replace [718, 864]
color blue
```
^button-habit-rit-due-weekly

- Inline: `button-habit-rit-due-weekly`

### Habits and Rituals Completed

```button
name Weekly Completed Habits and Rituals
type append template
action 112_45_dvmd_week_habit_rit_done
replace [872, 1117]
color blue
```
^button-habit-rit-done-weekly

- Inline: `button-habit-rit-done-weekly`

## Weekly PDEV Journals and Attributes

```button
name Weekly Journals and Attributes
type append template
action 112_90_dvmd_week_pdev
replace [52, 245]
color purple
```
^button-journals-attr-weekly

- Inline: `button-journals-attr-weekly`

## Weekly Library Content

```button
name Weekly Library Content
type append template
action 112_60_dvmd_week_lib
replace [1, 2]
color green
```
^button-lib-weekly

- Inline: `button-lib-weekly`

## Weekly PKM Table

```button
name Weekly PKM Files
type append template
action 112_70_dvmd_week_pkm
replace [1, 2]
color green
```
^button-pkm-weekly

- Inline: `button-pkm-weekly`

---

## Use Cases

1. [[32_00_week|Weekly Calendar Template]]
2. [[32_01_week_periodic|Weekly Calendar Periodic Note Template]]
3. [[32_02_week_days|Weekly and Weekdays Calendar Template]]
4. [[55_21_daily_morn_rit|Daily Morning Ritual Task Template]]
5. [[55_22_today_morn_rit|Today's Morning Ritual Task Template]]
6. [[55_23_tomorrow_morn_rit|Tomorrow's Morning Ritual Task Template]]
