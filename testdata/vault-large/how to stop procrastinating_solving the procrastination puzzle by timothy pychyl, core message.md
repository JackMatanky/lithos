---
title: how to stop procrastinating_solving the procrastination puzzle by timothy pychyl, core message
uuid: 2687f4a5-3f26-4253-ae1f-15cac1e24521
aliases:
  - "How to Stop Procrastinating: Solving the Procrastination Puzzle by Timothy Pychyl, Core Message"
  - How to Stop Procrastinating_Solving the Procrastination Puzzle by Timothy Pychyl
  - Core Message
  - how_to_stop_procrastinating_solving_the_procrastination_puzzle_by_timothy_pychyl
  - _core_message
  - how to stop procrastinating_solving the procrastination puzzle by timothy pychyl
  - core message
main_title: How to Stop Procrastinating
subtitle: Solving the Procrastination Puzzle by Timothy Pychyl, Core Message
author: lozeron_nathan
date_published: 2023-08-21
publisher:
  - productivity_game
  - youtube
series: undefined
url: https://www.youtube.com/watch?v=6a7NdW4sFIU&ab_channel=ProductivityGame
cssclasses: null
status: resource
type: youtube
file_class: lib_video
date_created: 2023-08-21T08:46
date_modified: 2025-10-05T14:54
tags:
---
# How to Stop Procrastinating: Solving the Procrastination Puzzle by Timothy Pychyl, Core Message

> [!video] Video Details
>
> - **Title**:: [How to Stop Procrastinating: Solving the Procrastination Puzzle by Timothy Pychyl, Core Message](https://www.youtube.com/watch?v=6a7NdW4sFIU&ab_channel=ProductivityGame)
> - **Author**:: [[lozeron_nathan|Nathan Lozeron]]
> - **Publisher**:: [[productivity_game|Productivity Game]], [[youtube|YouTube]]
> - **Series**:: [undefined](undefined)
> - **Date Published**:: 2023-08-21
>
> - **Completed**::

---

## Video

### Embed

![How to Stop Procrastinating: Solving the Procrastination Puzzle by Timothy Pychyl, Core Message](https://www.youtube.com/watch?v=6a7NdW4sFIU&ab_channel=ProductivityGame)

![[Lozeron_Insights from Solving the Procrastination Puzzle by Timothy Pychyl.pdf]]

### Timestamp Notes

Timestamp hotkey: `Ctrl + Alt + T`

```timestamp-url
 https://www.youtube.com/watch?v=6a7NdW4sFIU&ab_channel=ProductivityGame
```

---

## Related Knowledge

> [!pkm] PKM Notes
>
> - `BUTTON[button-pkm-question-table]`|`BUTTON[button-pkm-evidence-table]`|`BUTTON[button-pkm-steps-table]`|`BUTTON[button-pkm-conclusion-table]`
> - `BUTTON[button-pkm-idea-table]`|`BUTTON[button-pkm-summary-table]`|`BUTTON[button-pkm-quote-table]`
> - `BUTTON[button-pkm-concept-table]`|`BUTTON[button-pkm-definition-table]`

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
	AS Context,
	file.etags AS Tags
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
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "perm")
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
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "lit")
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
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "fleet")
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
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	file.etags AS Tags
FROM
	"70_pkm"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "info")
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

<!-- Link related library content here -->

### Library Content

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	Author AS Author,
	choice(contains(file.frontmatter.type, "book"), file.frontmatter.year_published, file.frontmatter.date_published) AS "Date Published",
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

## Related Tasks and Events

| `BUTTON[button-project-task-table]` | `BUTTON[button-parent-task-table]` | `BUTTON[button-action-item-task-table]` | `BUTTON[button-meeting-task-table]` |
|:----------:|:----------:|:----------:|:----------:|

```button
name ✅Related Tasks and Events
type append template
action 100_40_dvmd_related_task_sect
replace [1, 2]
color blue
```

### Outgoing Task and Events Links

<!-- Link related tasks and events here -->

### Projects

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Project,
    choice(contains(file.frontmatter.status, "done"), "✔️Done",
    choice(contains(file.frontmatter.status, "in_progress"), "👟In progress",
    choice(contains(file.frontmatter.status, "to_do"), "🔜To do",
    choice(contains(file.frontmatter.status, "schedule"), "📅Schedule",
    choice(contains(file.frontmatter.status, "on_hold"), "🤌On hold", "❌Discarded")))))
    AS Status,
    choice((regextest(".", file.frontmatter.task_start) AND regextest(".", file.frontmatter.task_end)),
        (file.frontmatter.task_start + " → " + file.frontmatter.task_end),
        choice(regextest(".", file.frontmatter.task_start),
            (file.frontmatter.task_start + " → Present"),
            "null"))
    AS Dates,
    choice(file.frontmatter.context = "habit_ritual", "Habits and Rituals",
    upper(substring(file.frontmatter.context, 0, 1)) + substring(file.frontmatter.context, 1))
    AS Context,
    Objective AS Objective
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND !contains(file.path, this.file.folder)
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.type, "project")
SORT
    file.frontmatter.title ASC
```

### Parent Tasks

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS "Parent Task",
    choice(contains(file.frontmatter.status, "done"), "✔️Done",
    choice(contains(file.frontmatter.status, "in_progress"), "👟In progress",
    choice(contains(file.frontmatter.status, "to_do"), "🔜To do",
    choice(contains(file.frontmatter.status, "schedule"), "📅Schedule",
    choice(contains(file.frontmatter.status, "on_hold"), "🤌On hold", "❌Discarded")))))
    AS Status,
    choice((regextest(".", file.frontmatter.task_start) AND regextest(".", file.frontmatter.task_end)),
        (file.frontmatter.task_start + " → " + file.frontmatter.task_end),
        choice(regextest(".", file.frontmatter.task_start),
            (file.frontmatter.task_start + " → Present"),
            "null"))
    AS Dates,
    choice(length(file.frontmatter.project) < 2, file.frontmatter.project[0], flat(file.frontmatter.project)) AS Project,
    Objective AS Objective
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND !contains(file.path, this.file.folder)
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.type, "parent_task")
SORT
    file.frontmatter.task_start,
    file.frontmatter.title ASC
```

### Child Tasks

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_action_item"), "🔨Task",
    choice(contains(T.text, "_meeting"), "🤝Meeting",
    choice(contains(T.text, "_phone_call"), "📞Call",
    choice(contains(T.text, "_interview"), "💼Interview",
    choice(contains(T.text, "_appointment"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gathering"), "✉️Gathering",
    choice(contains(T.text, "_hangout"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morning_ritual"),    "🍵Rit.",
    choice(contains(T.text, "_workday_startup_ritual"), "🌇Rit.",
    choice(contains(T.text, "_workday_shutdown_ritual"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type,
    choice((T.status != "-"),
        (choice((T.status = "x"),
            "✔️Done",
            "🔜To do")),
        "❌Discard")
    AS Status,
    choice((T.status != "-"),
        (choice((T.status = "x"),
            dateformat(T.completion, "yy-MM-dd"),
            dateformat(T.due, "yy-MM-dd"))),
        "❌Discard")
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
    AND !contains(file.path, this.file.folder)
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "child")
    AND regextest("(#task)", T.text)
SORT
    dateformat(T.due, "yy-MM-dd"),
    T.time_start ASC
```

---

## Related Directory

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
    Organization AS Organization,
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
    file.frontmatter.website AS Website,
    file.frontmatter.linkedin AS LinkedIn,
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
