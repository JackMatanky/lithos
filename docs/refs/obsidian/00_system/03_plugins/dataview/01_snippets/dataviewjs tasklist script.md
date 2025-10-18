---
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

```dataviewjs
const duration = dv.luxon.Duration;
let today = dv.date("2023-04-12");
const pages = dv.pages('#task AND -"00_system"');
const tasks = pages.file.tasks.where((t) => dv.equal(t.completion, today));
const tag_inline_regex = /(#task)|\[.*$/g;
const type_regex =
  /(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)/g;
const title_regex = /^[A-Za-z0-9\'\-\s]*_/g;
dv.taskList(
  tasks,
  false
);
```
