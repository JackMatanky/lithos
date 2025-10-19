---
date_created: 2023-06-15T09:25
date_modified: 2023-10-25T16:22
---

```js
choice(
(dur(T.duration_est + " minutes") =
dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_end), 0, 2) + ":" + substring(string(T.time_end), 2))) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_start), 0, 2) + ":" + substring(string(T.time_start), 2))))),
"👍On Time",
choice(
(dur(T.duration_est + " minutes") >
dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_end), 0, 2) + ":" + substring(string(T.time_end), 2))) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_start), 0, 2) + ":" + substring(string(T.time_start), 2))))),
"🟢" +
(dur(T.duration_est + " minutes") -
dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_end), 0, 2) + ":" + substring(string(T.time_end), 2))) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_start), 0, 2) + ":" + substring(string(T.time_start), 2))))),
"❗" +
(dur((date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_end), 0, 2) + ":" + substring(string(T.time_end), 2))) - (date(dateformat(T.completion, "yyyy-MM-dd") + "T" + substring(string(T.time_start), 0, 2) + ":" + substring(string(T.time_start), 2)))) -
dur(T.duration_est + " minutes")))) AS Accuracy,

substring(string(T.time_start), 0, 2) + ":" + substring(string(T.time_start), 2)
substring(string(T.time_end), 0, 2) + ":" + substring(string(T.time_end), 2)
```
