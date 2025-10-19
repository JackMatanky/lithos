---
title: daily_tasks_done
aliases:
  - Daily Tasks Done Timeline
  - daily tasks done timeline
  - daily tasks done
cssclasses:
  - inline_title_hide
  - read_narrow_margin
file_class: task
date_created: 2023-09-03T19:26
date_modified: 2023-09-27050T3419:17
---
```dataviewjs
await dv.view("tasks_timeline", {
        pages: "dv.pages().file.tasks.filter((t) => t.completion >= dv.luxon.DateTime.now().minus({days: 1}))",
        globalTaskFilter: "#task",
        dailyNoteFolder: "10_calendar/11_days",
        section: "## Tasks",
        dateFormat: "YYYY-MM-DD",
        options: "noCounters noQuickEntry noYear noRelative noTag noFile noHeader",
        sort: "t => t.time_start"})
```
