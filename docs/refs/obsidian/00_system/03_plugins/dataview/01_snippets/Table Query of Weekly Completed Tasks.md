---
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

```dataview
TABLE WITHOUT ID 
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "(_action_item)|(_meeting)|(_habit)|(_morning_ritual)|(_workday_startup_ritual)|(_workday_shutdown_ritual)|(_evening_ritual)", "") AS Task,
	regexreplace(regexreplace(T.text, "(#task)|\[.*$", ""), "^[A-Za-z0-9\'\-\s]*_", "") AS Type,
	T.completion AS Completed,
	T.time_start AS Start,
	T.time_end AS End,
	T.duration_est + " minutes" AS Estimate,
	(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) - 
	date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start)) AS Duration,
	number(string(date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_end) -  date(dateformat(T.completion, "yyyy-MM-dd") + "T" + T.time_start))) - T.duration_est AS Accuracy,
	T.section AS Link
FROM -"00_system/05_templates" AND #task
FLATTEN file.tasks AS T
WHERE any(file.tasks, (t) => t.completion >= date(2023-05-28) 
	AND t.completion <= date(2023-06-03))
SORT T.completion, T.time_start ASC
```
