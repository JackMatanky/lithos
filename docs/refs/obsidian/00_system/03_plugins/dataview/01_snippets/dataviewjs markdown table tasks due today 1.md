---
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

[Show modified task name in dataview that's still connected to original task - Help - Obsidian Forum](https://forum.obsidian.md/t/show-modified-task-name-in-dataview-thats-still-connected-to-original-task/25126)
[TaskList Text Postprocessor · Issue #1254 · blacksmithgu/obsidian-dataview (github.com)](https://github.com/blacksmithgu/obsidian-dataview/issues/1254)

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
const table = dv.markdownTable(
  ["task", "type", "start", "end", "estimate", "link"],
  tasks
    .where((t) => dv.equal(datetime.fromISO(t.due).toFormat('yyyy-MM-dd'), today))
    .sort((t) => t.time_start)
    .map((t) => [
      t.text.replace(tag_inline_regex, "").replace(type_regex, ""),
      t.text.replace(tag_inline_regex, "").replace(title_regex, "").replaceAll(/_/g, " "),
      t.time_start,
      t.time_end,
      t.duration_est + " min",
      t.section
    ])
);

dv.paragraph(table);
```
