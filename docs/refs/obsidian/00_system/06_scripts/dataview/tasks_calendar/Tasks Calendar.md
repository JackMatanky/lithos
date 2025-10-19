---
cssclasses: tasks_calendar
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:17
---

```dataviewjs
await dv.view("tasks_calendar",
			  {pages: "",
			  dailyNoteFolder: "10_calendar/11_days",
			  globalTaskFilter: "#task",
			  view: "list",
			  firstDayOfWeek: "0",
			  options: "style5 noIcons filter"})
```
