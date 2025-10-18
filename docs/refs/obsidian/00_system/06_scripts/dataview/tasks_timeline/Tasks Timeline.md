---
cssclasses: inline_title_hide
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:17
---

```dataviewjs
await dv.view("tasks_timeline", {
        pages: "dv.pages().file.tasks.filter((t) => t.due <= dv.luxon.DateTime.now().plus({days: 7}))",
        globalTaskFilter: "#task",
        dailyNoteFolder: "10_calendar/11_days",
        section: "## Tasks",
        dateFormat: "YYYY-MM-DD",
        options: "noYear noTag noFile",
        sort: "t => t.time_start"})
```
