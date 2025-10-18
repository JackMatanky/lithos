---
title: side_panel_buttons
aliases:
  - Side Panel Buttons
  - side panel buttons
  - side_panel_buttons
cssclasses:
  - inline_title_hide
  - list_narrow
  - paragraph_narrow
  - side_panel_style
date_created: 2023-07-05T08:00
date_modified: 2024-11-08T13:04
tags: 
---
- **`dv: link((dateformat(date(today), "yyyy-MM-dd") + "_pdev"), "PDEV")`**:
	- `BUTTON[button-journal-daily]`|`dv: link((dateformat(date(today), "yy_MM_dd") + "_daily_reflection"), "Reflect")`|`dv: link((dateformat(date(today), "yy_MM_dd") + "_daily_gratitude"), "Gratitude")`|`dv: link((dateformat(date(today), "yy_MM_dd") + "_daily_detachment"), "Detach")`
- **`dv: link((dateformat(date(today), "yyyy-MM-dd") + "_task_event"), "Tasks and Events")`**:
	- `BUTTON[button-project-task-table]`|`BUTTON[button-parent-task-table]`|`BUTTON[button-action-item-task-table]`|`BUTTON[button-meeting-task-table]`
- **`dv: link((dateformat(date(today), "yyyy-MM-dd") + "_pkm"), "PKM")`**:
	- `BUTTON[button-pkm-question-table]`|`BUTTON[button-pkm-evidence-table]`|`BUTTON[button-pkm-steps-table]`|`BUTTON[button-pkm-conclusion-table]`
	- `BUTTON[button-pkm-idea-table]`|`BUTTON[button-pkm-summary-table]`|`BUTTON[button-pkm-quote-table]`
	- `BUTTON[button-pkm-concept-table]`|`BUTTON[button-pkm-definition-table]`