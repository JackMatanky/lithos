---
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

```dataviewjs
// FORMATTING CHARACTERS
const new_line = String.fromCodePoint(0xa);
const space = String.fromCodePoint(0x20);

const task_pages = dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks;
//const tasks = pages.file.tasks;

const duration = dv.luxon.Duration;
const datetime = dv.luxon.DateTime;
const today = datetime.now().toFormat("yyyy-MM-dd");

const regex_task_name =
  /#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+/g;
const regex_time_start = /#task.+time_start::\s(\d\d:\d\d).+/g;
const regex_time_end = /#task.+time_end::\s(\d\d:\d\d).+/g;

function task_name_format(task) {
  const name = task.text.replace(regex_task_name, "$1");
  const start = task.text.replace(regex_time_start, "$1");
  const end = task.text.replace(regex_time_end, "$1");
  const time_span = `⏳${start} →⌛${end}`;
  task.visual = dv.array([time_span, name]).join(": ");
  return task;
}
function task_time_sort(task) {
  const start = task.text.replace(regex_time_start, "$1");
  const time_start = datetime.fromFormat(start, "HH:mm").toFormat("HH:mm");
  return time_start;
}

dv.taskList(
  task_pages
    .where(
      (t) =>
        dv.equal(datetime.fromISO(t.due).toFormat("yyyy-MM-dd"), today) &&
        t.text.includes("#task") &&
        t.status == " "
    )
    .map((t) => task_name_format(t))
    .sort((t) => task_time_sort(t)),
  false
);
```
