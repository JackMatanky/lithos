---
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

```dataviewjs
const duration = dv.luxon.Duration;
const datetime = dv.luxon.DateTime;
const today = datetime.now().toFormat('yyyy-MM-dd');
const pages = dv.pages('-"00_system/05_templates" AND #task');
const tasks = pages.file.tasks;
const tag_inline_regex = /(#task)|\[.*$/g;
const type_regex =
  /(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)/g;
const title_regex = /^[A-Za-z0-9;:\'\s\-]*_/g;
dv.table(
  ["task", "type", "start", "end", "duration", "link"],
  tasks
    .where((t) => dv.equal(datetime.fromISO(t.completion).toFormat('yyyy-MM-dd'), today))
    .sort((t) => t.time_start)
    .map((t) => [
      t.text.replace(tag_inline_regex, "").replace(type_regex, ""),
      t.text.replace(tag_inline_regex, "").replace(title_regex, "").replaceAll(/_/g, " "),
      t.time_start,
      t.time_end,
      duration.fromMillis(duration.fromISOTime(t.time_end) - duration.fromISOTime(t.time_start)),
      t.section
    ])
);
```
