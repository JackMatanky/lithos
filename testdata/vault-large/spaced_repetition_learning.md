---
title: spaced_repetition_learning
uuid: 96d0f89d-ea1a-4af5-8e7e-55511f2bcdea
aliases:
  - Spaced Repetition Learning
  - spaced repetition learning
  - spaced_repetition_learning
task_start: "[[2024-05-12]]"
task_end: "[[null]]"
due_do: do
pillar:
  - "[[knowledge_expansion|Knowledge Expansion]]"
context: education
goal: null
organization:
contact:
library:
status: in_progress
type: project
file_class: task_project
date_created: 2024-05-12T10:34
date_modified: 2024-11-08T16:41
tags:
---
# Spaced Repetition Learning

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
> | [[spaced_repetition_learning#Prepare and Reflect\|Insight]] | [[spaced_repetition_learning#Tasks and Events\|Tasks and Events]] | [[spaced_repetition_learning#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[spaced_repetition_learning#Related Knowledge\|PKM]] | [[spaced_repetition_learning#Related Library Content\|Library]] | [[spaced_repetition_learning#Related Directory\|Directory]] |

> [!objective] Project Objective
>
> Write the objective as a sentence:
>
> > [!Hint] Use what-when-how-why structure
>
> - **Objective**::

### Preview

> [!task_preview] Project Preview
>
> 1. **NEED**
>     - What is the background context?
>     - What is the problem or need?
>     - Why does the problem matter?
>
> 2. **SPECIFIC**
>     - What needs to be accomplished?
>     - What steps need to be taken to succeed?
>
> 3. **TIME-BOUND**
>     - When is the start date?
>     - When is the end date?
>     - How much time per day or week is required to finish on time?
>
> 4. **ACTIONABLE**
>     - Which projects are happening concurrently and how much time do they require?
>     - Is the the allotted time enough considering the objective and other tasks?
>     - What could inhibit achieving the objective?
>     - Is this a realistic objective?
>
> 5. **MEASURABLE**
>     - How can I measure my execution? How frequently will I work toward the objective?
> 	    - **Lead**::
> 	    - **Lead**::
> 	    - **Lead**::
>     - How can I measure my progress and success?
> 	    - **Lag**::
> 	    - **Lag**::
> 	    - **Lag**::
>
> 6. **RELEVANT**
>     - How does the objective align with my life's…
> 	    - vision?
> 	    - needs?
> 	    - demands?
>
> 7. **EXCITING**
>     - Do I think this is important? Why?
>     - Does the objective interest me? Why?
>     - Am I excited to accomplish my objective?
>
> 8. **RISKY**
>     - How is the objective challenging?

### Plan

> [!task_plan] Project Plan
>
> Enumerate the project's critical actions, according to order of execution, and estimate the duration needed for completion.
>
> 1. critical_action short description
> 	- **Tactic**::
> 	- **Duration**::
> 1. critical_action short description
> 	- **Tactic**::
> 	- **Duration**::
> 1. critical_action short description
> 	- **Tactic**::
> 	- **Duration**::
> 1. critical_action short description
> 	- **Tactic**::
> 	- **Duration**::
> 1. critical_action short description
> 	- **Tactic**::
> 	- **Duration**::
> 1. critical_action short description
> 	- **Tactic**::
> 	- **Duration**::
> 1. critical_action short description
> 	- **Tactic**::
> 	- **Duration**::

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
> | [[spaced_repetition_learning#Prepare and Reflect\|Insight]] | [[spaced_repetition_learning#Tasks and Events\|Tasks and Events]] | [[spaced_repetition_learning#Related Tasks and Events\|Related Tasks]] |
> | :--------: | :--------: | :--------: |
> | [[spaced_repetition_learning#Related Knowledge\|PKM]] | [[spaced_repetition_learning#Related Library Content\|Library]] | [[spaced_repetition_learning#Related Directory\|Directory]] |

> [!task] Tasks and Events
>
> `BUTTON[button-project-task-table]`|`BUTTON[button-parent-task-table]`|`BUTTON[button-action-item-task-table]`|`BUTTON[button-meeting-task-table]`

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
    choice(regextest("\w", Outcome) AND regextest("\w", Feeling), list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)),
    choice(regextest("\w", Outcome) AND !regextest("\w", Feeling), ("**Outcome**: " + Outcome), "NULL")
    ) AS Result
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
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_|meeting|phone_call|video_call|interview|lecture|appointment|event|hangout|habit|gathering|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_video"), "📹Call",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_lecture"), "🧑‍🏫Lecture",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🦿Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))))
    AS Type,
    choice(T.status = "x", dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))
    AS Date,
    T.time_start AS Start,
    T.time_end AS End,
    choice(T.duration_est < 60, durationformat(dur(T.duration_est + "m"), "m 'min'"),
    choice(T.duration_est = 60, durationformat(dur(T.duration_est + "h"), "h 'hr'"),
    choice(T.duration_est % 60 = 0, durationformat(dur((T.duration_est/60) + "h"), "h 'hrs'"),
    choice(T.duration_est < 120,
      durationformat(dur((T.duration_est - 60) + "m 1h"), "h 'hr' m 'min'"),
      durationformat(dur((T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"), "h 'hrs' m 'min'")
    )))) AS Estimate,
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
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_|meeting|phone_call|video_call|interview|lecture|appointment|event|hangout|habit|gathering|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_video"), "📹Call",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_lecture"), "🧑‍🏫Lecture",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🦿Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))))
    AS Type,
    choice(T.status = "x", dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))
    AS Date,
    (T.time_start + " - " + T.time_end) AS Time,
    choice(T.status = "-", "❌Discarded",
      choice(T.status = "<", "⏹️Canceled",
      (choice(Estimate = Duration_ACT, "👍On Time",
      choice(Estimate > Duration_ACT,
        "🟢" + (Estimate - Duration_ACT),
        "❗" + (Duration_ACT - Estimate))
      ) + " (" + Estimate_FMT + ")")
    )) AS Accuracy,
    choice(regextest("\w", Outcome) AND regextest("\w", Feeling), list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)),
    choice(regextest("\w", Outcome) AND !regextest("\w", Feeling), ("**Outcome**: " + Outcome), "NULL")
    ) AS Result,
    choice(length(file.frontmatter.parent_task) < 2, file.frontmatter.parent_task[0], flat(file.frontmatter.parent_task)) AS "Parent Task"
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
FLATTEN
    dur(
      choice(T.duration_est < 60, T.duration_est + "m",
      choice(T.duration_est % 60 = 0,
        (T.duration_est/60) + "h",
        (T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"))
    ) AS Estimate
FLATTEN
    choice(T.duration_est < 60, durationformat(dur(T.duration_est + "m"), "m 'min'"),
    choice(T.duration_est = 60, durationformat(dur(T.duration_est + "h"), "h 'hr'"),
    choice(T.duration_est % 60 = 0, durationformat(dur((T.duration_est/60) + "h"), "h 'hrs'"),
    choice(T.duration_est < 120,
      durationformat(dur((T.duration_est - 60) + "m 1h"), "h 'hr' m 'min'"),
      durationformat(dur((T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"), "h 'hrs' m 'min'")
    )))) AS Estimate_FMT
FLATTEN
    dur(
      date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_end) -
      date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_start)
    ) AS Duration_ACT
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
> | [[spaced_repetition_learning#Prepare and Reflect\|Insight]] | [[spaced_repetition_learning#Tasks and Events\|Tasks and Events]] | [[spaced_repetition_learning#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[spaced_repetition_learning#Related Knowledge\|PKM]] | [[spaced_repetition_learning#Related Library Content\|Library]] | [[spaced_repetition_learning#Related Directory\|Directory]] |

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
    choice(regextest("\w", Outcome) AND regextest("\w", Feeling), list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)),
    choice(regextest("\w", Outcome) AND !regextest("\w", Feeling), ("**Outcome**: " + Outcome), "NULL")
    ) AS Result
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
    choice(regextest("\w", Outcome) AND regextest("\w", Feeling), list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)),
    choice(regextest("\w", Outcome) AND !regextest("\w", Feeling), ("**Outcome**: " + Outcome), "NULL")
    ) AS Result
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
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_|meeting|phone_call|video_call|interview|lecture|appointment|event|hangout|habit|gathering|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_video"), "📹Call",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_lecture"), "🧑‍🏫Lecture",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🦿Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))))
    AS Type,
    choice((T.status = "-"), "❌Discard",
      choice((T.status = "<"), "⏹️Canceled",
      choice((T.status = "x"), "✔️Done",
        "🔜To do")))
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
FLATTEN
    dur(
      choice(T.duration_est < 60, T.duration_est + "m",
      choice(T.duration_est % 60 = 0,
        (T.duration_est/60) + "h",
        (T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"))
    ) AS Estimate
FLATTEN
    choice(T.duration_est < 60, durationformat(dur(T.duration_est + "m"), "m 'min'"),
    choice(T.duration_est = 60, durationformat(dur(T.duration_est + "h"), "h 'hr'"),
    choice(T.duration_est % 60 = 0, durationformat(dur((T.duration_est/60) + "h"), "h 'hrs'"),
    choice(T.duration_est < 120,
      durationformat(dur((T.duration_est - 60) + "m 1h"), "h 'hr' m 'min'"),
      durationformat(dur((T.duration_est % 60) + "m " + floor(T.duration_est/60) + "h"), "h 'hrs' m 'min'")
    )))) AS Estimate_FMT
FLATTEN
    dur(
      date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_end) -
      date(dateformat(choice(T.status = "x", T.completion, T.due), "yyyy-MM-dd") + "T" + T.time_start)
    ) AS Duration_ACT
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
> | [[spaced_repetition_learning#Prepare and Reflect\|Insight]] | [[spaced_repetition_learning#Tasks and Events\|Tasks and Events]] | [[spaced_repetition_learning#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[spaced_repetition_learning#Related Knowledge\|PKM]] | [[spaced_repetition_learning#Related Library Content\|Library]] | [[spaced_repetition_learning#Related Directory\|Directory]] |

> [!pkm] PKM Notes
>
> - `BUTTON[button-pkm-question]`|`BUTTON[button-pkm-evidence]`|`BUTTON[button-pkm-steps]`|`BUTTON[button-pkm-conclusion]`
> - `BUTTON[button-pkm-idea]`|`BUTTON[button-pkm-summary]`|`BUTTON[button-pkm-quote]`
> - `BUTTON[button-pkm-concept]`|`BUTTON[button-pkm-definition]`

<!-- Adjust replace lines -->

```meta-bind-button
label: 🗃️Related PKM Files
tooltip: 'Replace the PKM section with MD table of linked files and a filtered DataView table'
class: mb_button_purple
style: default
hidden: false
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/100_70_dvmd_related_pkm_sect.md
    templater: true
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
    AS Type,
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
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
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
    AND contains(file.frontmatter.status, "perm")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Literature

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
      "question": "❔Question",
      "evidence": "⚖️Evidence",
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
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
    AND contains(file.frontmatter.file_class, "zettel")
    AND filter(list("question", "evidence", "step", "conclusion", "theor", "proof"),
      (x) => contains(file.frontmatter.type, x))
    AND !contains(file.frontmatter.status, "perm")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Fleeting

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
      "question": "❔Question",
      "evidence": "⚖️Evidence",
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
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
    AND contains(file.frontmatter.file_class, "zettel")
    AND filter(list("quote", "idea", "summary"),
      (x) => contains(file.frontmatter.type, x))
    AND !contains(file.frontmatter.status, "perm")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Info

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
      "question": "❔Question",
      "evidence": "⚖️Evidence",
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
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
    AND contains(file.frontmatter.file_class, "info")
    AND filter(list("def", "conc", "gen"),
      (x) => contains(file.frontmatter.type, x))
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

---

## Related Library Content

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[spaced_repetition_learning#Prepare and Reflect\|Insight]] | [[spaced_repetition_learning#Tasks and Events\|Tasks and Events]] | [[spaced_repetition_learning#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[spaced_repetition_learning#Related Knowledge\|PKM]] | [[spaced_repetition_learning#Related Library Content\|Library]] | [[spaced_repetition_learning#Related Directory\|Directory]] |

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
> | [[spaced_repetition_learning#Prepare and Reflect\|Insight]] | [[spaced_repetition_learning#Tasks and Events\|Tasks and Events]] | [[spaced_repetition_learning#Related Tasks and Events\|Related Tasks]] |
> |:--------: |:--------: |:--------: |
> | [[spaced_repetition_learning#Related Knowledge\|PKM]] | [[spaced_repetition_learning#Related Library Content\|Library]] | [[spaced_repetition_learning#Related Directory\|Directory]] |

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
