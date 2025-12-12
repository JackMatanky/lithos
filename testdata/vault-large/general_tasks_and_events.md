---
title: general_tasks_and_events
uuid: 9f2f2af0-7d22-42fc-87ce-b99c7c922a25
aliases:
  - General Tasks and Events
  - general tasks and events
  - general_tasks_and_events
task_start:
task_end:
due_do: do
pillar:
context: personal
goal: null
organization:
contact:
library:
status: in_progress
type: project
file_class: task_project
date_created: 2023-06-28T22:31
date_modified: 2024-05-31T14:28
tags:
---
# General Tasks and Events

> [!project] Project Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Goal**: `dv: this.file.frontmatter.goal`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) =>!contains(lower(x), "null")), " | ")`
>
> - **Dates**: `dv: join([this.file.frontmatter.task_start, this.file.frontmatter.task_end], " - ")`

---

## Prepare and Reflect

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[general_tasks_and_events#Prepare and Reflect\|Insight]] | [[general_tasks_and_events#Tasks and Events\|Tasks]] | [[general_tasks_and_events#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[general_tasks_and_events#Related Knowledge\|PKM]] | [[general_tasks_and_events#Related Library Content\|Library]] | [[general_tasks_and_events#Related Directory\|Directory]] |

> [!objective] Project Objective
>
> Write the objective as a sentence:
> - **Objective**::

### Preview

> [!task_preview] Project Preview
>
> 1. **NEED**
>     - What is the problem or need?
>     - Why does the problem matter?
>
> 2. **SPECIFIC**
>     - What needs to be accomplished?
>     - What steps need to be taken to succeed?
>
> 3. **MEASURABLE**
>     - How can I measure my progress?
>     - How will I know if I have succeeded?
>
> 4. **ACTIONABLE**
>     - Is this a realistic objective?
>     - Is the the allotted time enough considering the objective and other tasks?
>
> 5. **RELEVANT**
>     - How does the objective align with my life's demands?
>     - How does the objective align with my life's needs?
>
> 6. **TIME-BOUND**
>     - When is the start date?
>     - When is the end date?
>     - How frequently will I work toward the objective?
>
> 7. **EXCITING**
>     - Do I think this is important? Why?
>     - Does the objective interest me? Why?
>     - Am I excited to accomplish my objective?
>
> 8. **RISKY**
>     - How is the objective challenging?

### Review

> [!task_review] Project Review
>
> 1. What was the outcome? How did it make me feel?
>     - **Outcome**::
>     - **Feeling**::
>
> 2. What went well?
>     - **Keep**::
>
> 3. What can be improved?
>     - **Improve**::
>
> 4. What can be started?
>     - **Start**::
>
> 5. What can be stopped?
>     - **Stop**::

---

## Tasks and Events

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[general_tasks_and_events#Prepare and Reflect\|Insight]] | [[general_tasks_and_events#Tasks and Events\|Tasks]] | [[general_tasks_and_events#Related Tasks and Events\|Related Tasks]] |
> | :--------: | :--------: | :--------: |
> | [[general_tasks_and_events#Related Knowledge\|PKM]] | [[general_tasks_and_events#Related Library Content\|Library]] | [[general_tasks_and_events#Related Directory\|Directory]] |

| `BUTTON[button-project-task-table]` | `BUTTON[button-parent-task-table]` | `BUTTON[button-action-item-task-table]` | `BUTTON[button-meeting-task-table]` |
| :--------: | :--------: | :--------: | :--------: |

<!-- Adjust replace lines -->

```button
name 🏗️Project Tasks and Events
type append template
action 140_00_dvmd_task_sect_proj
replace [1, 2]
color blue
```

### Parent Tasks

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS "Parent Task",
    default(((x) => {
      "done": "✔️Done",
      "in_progress": "👟In progress",
      "to_do": "🔜To do",
      "schedule": "📅Schedule",
      "on_hold": "🤌On hold",
      "applied": "📨Applied💼",
      "offer": "📝Job Offer💼",
      "rejected": "🚫Rejected💼"
    }[x])(file.frontmatter.status), "❌Discarded")
	AS Status,
    choice((regextest("\d", file.frontmatter.task_start) AND regextest("\d", file.frontmatter.task_end)),
		(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → " + dateformat(date(regexreplace(file.frontmatter.task_end, "[^\d-]", "")), "yy-MM-dd")),
		choice(regextest("\d", file.frontmatter.task_start),
			(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → Present"),
			"NULL"))
	AS Dates,
    file.frontmatter.contact AS Contact,
    Objective AS Objective,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "parent")
    AND contains(file.frontmatter.project, this.file.name)
    AND contains(file.path, this.file.folder)
SORT
    file.frontmatter.task_start,
    file.frontmatter.title ASC
```

### Remaining Tasks

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type,
    choice(T.status = "x", dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))
    AS Date,
    T.time_start AS Start,
    T.time_end AS End,
    dur(choice(T.duration_est < 60, T.duration_est + " m", choice(T.duration_est % 60 = 0, (T.duration_est/60) + " h", (T.duration_est % 60) + " m " + floor(T.duration_est/60) + " h")))
    AS Estimate,
    choice(length(file.frontmatter.parent_task) < 2, file.frontmatter.parent_task[0], flat(file.frontmatter.parent_task)) AS "Parent Task"
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "child")
    AND contains(file.path, this.file.folder)
    AND T.status = " "
    AND regextest("#task", T.text)
SORT
    T.due,
    T.time_start ASC
```

### Completed Tasks

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type,
    choice(T.status = "x", dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))
    AS Date,
    (T.time_start + " - " + T.time_end) AS Time,
    choice(T.status = "-", "❌Discarded",
    (choice(dur(choice(T.duration_est < 60, T.duration_est + " m", choice(T.duration_est % 60 = 0, (T.duration_est/60) + " h", (T.duration_est % 60) + " m " + floor(T.duration_est/60) + " h"))) = dur((date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_start))), "👍On Time",
      (choice(dur(choice(T.duration_est < 60, T.duration_est + " m", choice(T.duration_est % 60 = 0, (T.duration_est/60) + " h", (T.duration_est % 60) + " m " + floor(T.duration_est/60) + " h"))) > dur((date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_start))),
        "🟢" + (dur(choice(T.duration_est < 60, T.duration_est + " m", choice(T.duration_est % 60 = 0, (T.duration_est/60) + " h", (T.duration_est % 60) + " m " + floor(T.duration_est/60) + " h"))) - dur((date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_start)))),
        "❗" + (dur((date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_end)) - (date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_start))) - dur(choice(T.duration_est < 60, T.duration_est + " m", choice(T.duration_est % 60 = 0, (T.duration_est/60) + " h", (T.duration_est % 60) + " m " + floor(T.duration_est/60) + " h"))))))
        + " (" + dur(choice(T.duration_est < 60, T.duration_est + " m", choice(T.duration_est % 60 = 0, (T.duration_est/60) + " h", (T.duration_est % 60) + " m " + floor(T.duration_est/60) + " h"))) + ")")))
    AS Accuracy,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result,
    choice(length(file.frontmatter.parent_task) < 2, file.frontmatter.parent_task[0], flat(file.frontmatter.parent_task)) AS "Parent Task"
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "child")
    AND contains(file.path, this.file.folder)
    AND T.status != " "
    AND regextest("#task", T.text)
SORT
    T.due,
    T.time_start ASC
```

---

## Related Tasks and Events

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[general_tasks_and_events#Prepare and Reflect\|Insight]] | [[general_tasks_and_events#Tasks and Events\|Tasks]] | [[general_tasks_and_events#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[general_tasks_and_events#Related Knowledge\|PKM]] | [[general_tasks_and_events#Related Library Content\|Library]] | [[general_tasks_and_events#Related Directory\|Directory]] |

<!-- Adjust replace lines -->

```button
name ✅Related Tasks and Events
type append template
action 100_50_dvmd_related_task_sect
replace [1, 2]
color blue
```

### Outgoing Task and Events Links

<!-- Link related tasks and events here -->

### Projects

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS Project,
    default(((x) => {
      "done": "✔️Done",
      "in_progress": "👟In progress",
      "to_do": "🔜To do",
      "schedule": "📅Schedule",
      "on_hold": "🤌On hold",
      "applied": "📨Applied💼",
      "offer": "📝Job Offer💼",
      "rejected": "🚫Rejected💼"
    }[x])(file.frontmatter.status), "❌Discarded")
	AS Status,
    choice((regextest("\d", file.frontmatter.task_start) AND regextest("\d", file.frontmatter.task_end)),
		(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → " + dateformat(date(regexreplace(file.frontmatter.task_end, "[^\d-]", "")), "yy-MM-dd")),
		choice(regextest("\d", file.frontmatter.task_start),
			(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → Present"),
			"NULL"))
	AS Dates,
    choice(file.frontmatter.context = "habit_ritual", "Habits and Rituals",
    upper(substring(file.frontmatter.context, 0, 1)) + substring(file.frontmatter.context, 1))
    AS Context,
    Objective AS Objective,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "project")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !contains(file.path, this.file.folder)
SORT
    file.frontmatter.title ASC
```

### Parent Tasks

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS "Parent Task",
    default(((x) => {
      "done": "✔️Done",
      "in_progress": "👟In progress",
      "to_do": "🔜To do",
      "schedule": "📅Schedule",
      "on_hold": "🤌On hold",
      "applied": "📨Applied💼",
      "offer": "📝Job Offer💼",
      "rejected": "🚫Rejected💼"
    }[x])(file.frontmatter.status), "❌Discarded")
	AS Status,
    choice((regextest("\d", file.frontmatter.task_start) AND regextest("\d", file.frontmatter.task_end)),
		(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → " + dateformat(date(regexreplace(file.frontmatter.task_end, "[^\d-]", "")), "yy-MM-dd")),
		choice(regextest("\d", file.frontmatter.task_start),
			(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → Present"),
			"NULL"))
	AS Dates,
    choice(length(file.frontmatter.project) < 2, file.frontmatter.project[0], flat(file.frontmatter.project)) AS Project,
    Objective AS Objective,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "parent")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !contains(file.path, this.file.folder)
SORT
    file.frontmatter.task_start,
    file.frontmatter.title ASC
```

### Child Tasks

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type,
    choice((T.status != "-"),
        (choice((T.status = "x"), "✔️Done", "🔜To do")),
        "❌Discard")
    AS Status,
    choice(T.status = "x", dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))
    AS Date,
    choice(length(file.frontmatter.project) < 2, file.frontmatter.project[0], flat(file.frontmatter.project)) AS Project
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "child")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !contains(file.path, this.file.folder)
    AND regextest("#task", T.text)
SORT
    T.due,
    T.time_start ASC
```

---

## Related Knowledge

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[general_tasks_and_events#Prepare and Reflect\|Insight]] | [[general_tasks_and_events#Tasks and Events\|Tasks]] | [[general_tasks_and_events#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[general_tasks_and_events#Related Knowledge\|PKM]] | [[general_tasks_and_events#Related Library Content\|Library]] | [[general_tasks_and_events#Related Directory\|Directory]] |

> [!pkm] PKM Notes
>
> - `BUTTON[button-pkm-question-table]`|`BUTTON[button-pkm-evidence-table]`|`BUTTON[button-pkm-steps-table]`|`BUTTON[button-pkm-conclusion-table]`
> - `BUTTON[button-pkm-idea-table]`|`BUTTON[button-pkm-summary-table]`|`BUTTON[button-pkm-quote-table]`
> - `BUTTON[button-pkm-concept-table]`|`BUTTON[button-pkm-definition-table]`

<!-- Adjust replace lines -->

```button
name Related PKM Files
type append template
action 100_70_dvmd_related_pkm_sect
replace [1, 2]
color purple
```

### Outgoing PKM Links

<!-- Link related pkm files here -->

### Knowledge Tree

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  default(((x) => {
      "category": "🏘️Category",
	    "branch": "🪑Branch",
	    "field": "🚪Field",
	    "subject": "🗝️Subject",
	    "topic": "🧱Topic"
    }[x])(file.frontmatter.type), "🔩Subtopic")
	  AS Subtype,
	  file.frontmatter.about AS Description,
	  default(((x) => {
      "branch": file.frontmatter.category,
      "field": flat(list(file.frontmatter.category, file.frontmatter.branch)),
      "subject": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field)),
      "topic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject)),
      "subtopic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject, file.frontmatter.topic))
    }[x])(file.frontmatter.type), "")
    AS Context
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.file_class, "tree")
	  AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "category": 1,
      "branch": 2,
      "field": 3,
      "subject": 4,
      "topic": 5
    }[x])(file.frontmatter.type), 6),
	  file.frontmatter.title ASC
```

### Permanent

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  default(((x) => {
      "question": "❔Question",
	    "evidence": "⚖️Evidence",
	    "conclusion": "🏁Conclusion",
	    "problem": "🪨Problem",
	    "step": "🪜Step",
	    "answer": "🎱Answer",
	    "quote": "⏺️Quote",
	    "idea": "💭Idea",
	    "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
	  AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
	    "review": "🔜Review",
	    "clarify": "🌱Clarify",
	    "develop": "🪴Develop",
	    "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	  AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "permanent")
	  AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "qec_question": 1,
    	"qec_evidence": 2,
    	"qec_conclusion": 3,
    	"psa_problem": 4,
    	"psa_step": 5,
    	"psa_answer": 6,
    	"quote": 7,
    	"idea": 8,
    	"concept": 9
    }[x])(file.frontmatter.subtype), 10),
	file.frontmatter.title ASC
```

### Literature

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  default(((x) => {
      "question": "❔Question",
	    "evidence": "⚖️Evidence",
	    "conclusion": "🏁Conclusion",
	    "problem": "🪨Problem",
	    "step": "🪜Step",
	    "answer": "🎱Answer",
	    "quote": "⏺️Quote",
	    "idea": "💭Idea",
	    "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
	  AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
	    "review": "🔜Review",
	    "clarify": "🌱Clarify",
	    "develop": "🪴Develop",
	    "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	  AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "literature")
	  AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "qec_question": 1,
    	"qec_evidence": 2,
    	"qec_conclusion": 3,
    	"psa_problem": 4,
    	"psa_step": 5,
    	"psa_answer": 6,
    	"quote": 7,
    	"idea": 8,
    	"concept": 9
    }[x])(file.frontmatter.subtype), 10),
	file.frontmatter.title ASC
```

### Fleeting

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  default(((x) => {
      "question": "❔Question",
	    "evidence": "⚖️Evidence",
	    "conclusion": "🏁Conclusion",
	    "problem": "🪨Problem",
	    "step": "🪜Step",
	    "answer": "🎱Answer",
	    "quote": "⏺️Quote",
	    "idea": "💭Idea",
	    "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
	  AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
	    "review": "🔜Review",
	    "clarify": "🌱Clarify",
	    "develop": "🪴Develop",
	    "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	  AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "fleeting")
	  AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "qec_question": 1,
    	"qec_evidence": 2,
    	"qec_conclusion": 3,
    	"psa_problem": 4,
    	"psa_step": 5,
    	"psa_answer": 6,
    	"quote": 7,
    	"idea": 8,
    	"concept": 9
    }[x])(file.frontmatter.subtype), 10),
	file.frontmatter.title ASC
```

### Info

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  default(((x) => {
      "question": "❔Question",
	    "evidence": "⚖️Evidence",
	    "conclusion": "🏁Conclusion",
	    "problem": "🪨Problem",
	    "step": "🪜Step",
	    "answer": "🎱Answer",
	    "quote": "⏺️Quote",
	    "idea": "💭Idea",
	    "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
	  AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
	    "review": "🔜Review",
	    "clarify": "🌱Clarify",
	    "develop": "🪴Develop",
	    "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	  AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "info")
	  AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "qec_question": 1,
    	"qec_evidence": 2,
    	"qec_conclusion": 3,
    	"psa_problem": 4,
    	"psa_step": 5,
    	"psa_answer": 6,
    	"quote": 7,
    	"idea": 8,
    	"concept": 9
    }[x])(file.frontmatter.subtype), 10),
	file.frontmatter.title ASC
```

---

## Related Library Content

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[general_tasks_and_events#Prepare and Reflect\|Insight]] | [[general_tasks_and_events#Tasks and Events\|Tasks]] | [[general_tasks_and_events#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[general_tasks_and_events#Related Knowledge\|PKM]] | [[general_tasks_and_events#Related Library Content\|Library]] | [[general_tasks_and_events#Related Directory\|Directory]] |

<!-- Adjust replace lines -->

```meta-bind-button
label: 🏫Related Library Content
tooltip: "Replace the library section with MD table of linked files and a filtered DataView table"
class: mb_button_green
hidden: false
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/100_60_dvmd_related_lib_sect.md
    templater: true
```

### Outgoing Library Links

<!-- Link related library files here -->

### Library Content

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	choice(length(choice(contains(file.frontmatter.type, "course"), file.frontmatter.lecturer, file.frontmatter.author)) < 2,
      choice(contains(file.frontmatter.type, "course"), file.frontmatter.lecturer[0], file.frontmatter.author[0]),
      flat(choice(contains(file.frontmatter.type, "course"), file.frontmatter.lecturer, file.frontmatter.author)))
    AS Creator,
	choice(contains(file.frontmatter.type, "book"),
		file.frontmatter.year_published,
		file.frontmatter.date_published)
	AS "Date Published",
	default(((x) => {
      "book": "📚Book",
      "book_chapter": "📑Book Chapter",
      "course": "🧑‍🏫Course",
      "course_lecture": "🧑‍🎓Course Lecture",
      "journal": "📜️Journal",
      "report": "📈Report",
      "news": "🗞️News",
      "magazine": "📰️Magazine",
      "webpage": "🌐Webpage",
      "blog": "💻Blog",
      "video": "🎥️Video",
      "youtube": "▶YouTube",
      "documentary": "🖼️Documentary",
      "audio": "🔉Audio",
      "podcast": "🎧️Podcast"
    }[x])(file.frontmatter.type), "📃Documentation")
	AS Type,
	default(((x) => {
      "undetermined": "❓Undetermined",
      "to_do": "🔜To do",
      "in_progress": "👟In progress",
      "done": "✔️Done",
      "resource": "🗃️Resource",
      "schedule": "📅Schedule"
    }[x])(file.frontmatter.status), "🤌On hold")
    AS Status,
	file.etags AS Tags
FROM
	"60_library"
	OR "90_inbox"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "lib")
	AND contains(file.frontmatter.file_class, "")
SORT
	file.frontmatter.type,
	file.frontmatter.title ASC
```

---

## Related Directory

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[general_tasks_and_events#Prepare and Reflect\|Insight]] | [[general_tasks_and_events#Tasks and Events\|Tasks]] | [[general_tasks_and_events#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[general_tasks_and_events#Related Knowledge\|PKM]] | [[general_tasks_and_events#Related Library Content\|Library]] | [[general_tasks_and_events#Related Directory\|Directory]] |

<!-- Adjust replace lines -->

```meta-bind-button
label: 📇Related Directory Files
tooltip: "Replace the directory section with MD table of linked files and a filtered DataView table"
class: mb_button_pink
style: default
hidden: false
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/100_50_dvmd_related_dir_sect.md
    templater: true
```

### Outgoing Contact Links

<!-- Link related contacts here -->

### Contacts

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS Name,
    file.frontmatter.job_title AS "Job Title",
    file.frontmatter.organization AS Organization,
    file.etags AS Tags
FROM
    "51_contacts"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "dir")
    AND contains(file.frontmatter.file_class, "contact")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    file.frontmatter.title ASC
```

### Outgoing Organization Links

<!-- Link related organizations here -->

### Organizations

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS Name,
    elink(file.frontmatter.url, "Website") AS Website,
    elink(file.frontmatter.linkedin_url, "LinkedIn") AS LinkedIn,
    file.frontmatter.about AS About,
    file.etags AS Tags
FROM
    "52_organizations"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "dir")
    AND contains(file.frontmatter.file_class, "organization")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    file.frontmatter.title ASC
```

---
