---
date_created: 2023-06-12T08:14
date_modified: 2023-10-25T16:22
---

```javascript
const task_type = `choice(contains(T.text, "_action_item"),
	"🔨Task",
	choice(contains(T.text, "_meeting"),
		"🤝Meeting",
		choice(contains(T.text, "_habit"),
			"🤖Habit",
			choice(contains(T.text, "_morning_ritual"),
				"🍵Rit.",
				choice(contains(T.text, "_workday_startup_ritual"),
					"🌇Rit.",
					choice(contains(T.text, "_workday_shutdown_ritual"),
						"🌆Rit.",
						"🛌Rit."))))))
AS Type`
```
