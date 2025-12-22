---
title: 23_09_20_multidenominational_selichot_at_the_first_station
uuid: ae8c11f0-7b1c-4810-97e8-d78d3b8587f5
aliases:
  - Multidenominational Selichot at the First Station
  - 23-09-20 Multidenominational Selichot at the First Station
  - multidenominational selichot at the first station
  - multidenominational_selichot_at_the_first_station
  - 23_09_20_multidenominational_selichot_at_the_first_station
date: "[[2023-09-20]]"
due_do: do
pillar:
  - "[[partner|Partner]]"
context: personal
goal:
project:
  - "[[general_tasks_and_events|General Tasks and Events]]"
parent_task:
  - "[[general_events|General Events]]"
organization:
  - "[[jerusalem_first_station|Jerusalem First Station]]"
contact:
  - "[[rothschild_samantha|Samantha Rothschild]]"
library:
type: event
file_class: task_child
date_created: 2023-09-21T10:11
date_modified: 2024-05-31T14:33
tags: task
---
# Multidenominational Selichot at the First Station

> [!meeting] Meeting Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

## Tasks and Events

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[23_09_20_multidenominational_selichot_at_the_first_station#Tasks and Events\|Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Prepare and Reflect\|Preview and Review]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Tasks and Events\|Related Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Knowledge\|PKM]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Library Content\|Library]] |
> |:--------: |:--------: |:--------: |:--------: |:--------: |

- [x] #task Multidenominational Selichot at the First Station_event [time_start:: 20:00]  [time_end:: 22:00]  [duration_est:: 120] ⏰ 2023-09-20 19:55 ➕ 2023-09-21 📅 2023-09-20 ✅ 2023-09-20

---

## Prepare and Reflect

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[23_09_20_multidenominational_selichot_at_the_first_station#Tasks and Events\|Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Prepare and Reflect\|Preview and Review]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Tasks and Events\|Related Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Knowledge\|PKM]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Library Content\|Library]] |
> |:--------: |:--------: |:--------: |:--------: |:--------: |

### Preview

> [!task_preview] Action Item Preview
>
> 1. What is the problem to solve?
>     - **Problem**::
>
> 2. What do I want?
>     - **Desire**::
>
> 3. What will I do?
>     - **Plan**::
>
> 4. What won't I do?
>     - **Refrain**::

### Plan

> [!task_plan] Execution Plan
>
> What is the detailed execution plan?
>
> - subtask short description
>     - [ ] subtask
> - subtask short description
>     - [ ] subtask
> - subtask short description
>     - [ ] subtask
> - subtask short description
>     - [ ] subtask

### Review

> [!task_review] Action Item Review
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

## Related Tasks and Events

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[23_09_20_multidenominational_selichot_at_the_first_station#Tasks and Events\|Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Prepare and Reflect\|Preview and Review]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Tasks and Events\|Related Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Knowledge\|PKM]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Library Content\|Library]] |
> |:--------: |:--------: |:--------: |:--------: |:--------: |

> [!task] Tasks and Events
>
> `BUTTON[button-project-task-table]`|`BUTTON[button-parent-task-table]`|`BUTTON[button-action-item-task-table]`|`BUTTON[button-meeting-task-table]`

<!-- Adjust replace lines -->

```meta-bind-button
label: 🔨Child Task Tasks and Events🤝
tooltip: "Replace the child task's tasks and events section with MD table of linked files and a filtered DataView table"
class: mb_button_blue
style: default
hidden: false
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/142_00_dvmd_task_sect_child.md
    templater: true
```

### Outgoing Task and Events Links

<!-- Link related tasks and events here -->

### Project and Parent Task

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS Title,
    choice(contains(file.frontmatter.type, "project"), "🏗️Project", "⚒️Parent Task") AS Type,
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
		(file.frontmatter.task_start + " → " + file.frontmatter.task_end),
		choice(regextest("\d", file.frontmatter.task_start),
			(file.frontmatter.task_start + " → Present"),
			"null"))
	AS Dates,
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
    AND (contains(file.frontmatter.file_class, "project")
    OR contains(file.frontmatter.file_class, "parent"))
    AND (contains(this.file.frontmatter.project, file.name)
    OR contains(this.file.frontmatter.parent_task, file.name))
SORT
    choice(contains(file.frontmatter.type, "project"), 1, 2),
    file.frontmatter.title ASC
```

### Sibling Child Tasks

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
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
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
    AND filter(file.frontmatter.project, (project) =>
      contains(this.file.frontmatter.project, project))
    AND filter(file.frontmatter.parent_task, (parent) =>
      contains(this.file.frontmatter.parent_task, parent))
    AND regextest("(#task)", T.text)
SORT
    T.due,
    T.time_start ASC
```

### General Child Tasks

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
    file.frontmatter.project
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
    AND regextest("(#task)", T.text)
SORT
    T.due,
    T.time_start ASC
```

---

## Related Knowledge

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[23_09_20_multidenominational_selichot_at_the_first_station#Tasks and Events\|Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Prepare and Reflect\|Preview and Review]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Tasks and Events\|Related Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Knowledge\|PKM]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Library Content\|Library]] |
> |:--------: |:--------: |:--------: |:--------: |:--------: |

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
	  choice(file.frontmatter.subtype = "qec_question", list(Context, Question),
	  choice(file.frontmatter.subtype = "qec_evidence", Evidence,
	  choice(file.frontmatter.subtype = "qec_conclusion", Conclusion,
	  choice(file.frontmatter.subtype = "psa_problem", list(Context, Problem),
	  choice(file.frontmatter.subtype = "psa_step", Step,
	  choice(file.frontmatter.subtype = "psa_answer", Answer,
	  choice(file.frontmatter.subtype = "quote", Quote,
	  choice(file.frontmatter.subtype = "idea", Idea,
	  choice(file.frontmatter.subtype = "concept", Description, Definition)))))))))
	  AS Content,
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
	  choice(file.frontmatter.subtype = "qec_question", list(Context, Question),
	  choice(file.frontmatter.subtype = "qec_evidence", Evidence,
	  choice(file.frontmatter.subtype = "qec_conclusion", Conclusion,
	  choice(file.frontmatter.subtype = "psa_problem", list(Context, Problem),
	  choice(file.frontmatter.subtype = "psa_step", Step,
	  choice(file.frontmatter.subtype = "psa_answer", Answer,
	  choice(file.frontmatter.subtype = "quote", Quote,
	  choice(file.frontmatter.subtype = "idea", Idea,
	  choice(file.frontmatter.subtype = "concept", Description, Definition)))))))))
	  AS Content,
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
	  choice(file.frontmatter.subtype = "qec_question", list(Context, Question),
	  choice(file.frontmatter.subtype = "qec_evidence", Evidence,
	  choice(file.frontmatter.subtype = "qec_conclusion", Conclusion,
	  choice(file.frontmatter.subtype = "psa_problem", list(Context, Problem),
	  choice(file.frontmatter.subtype = "psa_step", Step,
	  choice(file.frontmatter.subtype = "psa_answer", Answer,
	  choice(file.frontmatter.subtype = "quote", Quote,
	  choice(file.frontmatter.subtype = "idea", Idea,
	  choice(file.frontmatter.subtype = "concept", Description, Definition)))))))))
	  AS Content,
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
	  choice(file.frontmatter.subtype = "qec_question", list(Context, Question),
	  choice(file.frontmatter.subtype = "qec_evidence", Evidence,
	  choice(file.frontmatter.subtype = "qec_conclusion", Conclusion,
	  choice(file.frontmatter.subtype = "psa_problem", list(Context, Problem),
	  choice(file.frontmatter.subtype = "psa_step", Step,
	  choice(file.frontmatter.subtype = "psa_answer", Answer,
	  choice(file.frontmatter.subtype = "quote", Quote,
	  choice(file.frontmatter.subtype = "idea", Idea,
	  choice(file.frontmatter.subtype = "concept", Description, Definition)))))))))
	  AS Content,
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
> | [[23_09_20_multidenominational_selichot_at_the_first_station#Tasks and Events\|Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Prepare and Reflect\|Preview and Review]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Tasks and Events\|Related Tasks]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Knowledge\|PKM]] | [[23_09_20_multidenominational_selichot_at_the_first_station#Related Library Content\|Library]] |
> |:--------: |:--------: |:--------: |:--------: |:--------: |

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
	Author AS Author,
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
