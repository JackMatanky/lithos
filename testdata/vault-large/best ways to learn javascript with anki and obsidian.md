---
title: best ways to learn javascript with anki and obsidian
uuid: d2878262-c569-4129-9ddd-a7ad20315273
aliases:
  - Best Ways to Learn Javascript with Anki and Obsidian
  - Best Ways to Learn Javascript with Anki and Obsidian
  - best_ways_to_learn_javascript_with_anki_and_obsidian
  - best ways to learn javascript with anki and obsidian
main_title: Best Ways to Learn Javascript with Anki and Obsidian
subtitle: undefined
author: jenks_bryan
date_published: 2021-04-05
publisher: youtube
series: undefined
url: https://www.youtube.com/watch?v=43KOH-l-SYo&t=1002s&ab_channel=BryanJenks
cssclasses: null
status: done
type: youtube
file_class: lib_video
date_created: 2023-07-19T12:29
date_modified: 2025-10-05T14:54
tags:
---
# Best Ways to Learn Javascript with Anki and Obsidian

> [!video] Video Details
>
> - **Title**:: [Best Ways to Learn Javascript with Anki and Obsidian](https://www.youtube.com/watch?v=43KOH-l-SYo&t=1002s&ab_channel=BryanJenks)
> - **Author**:: [[jenks_bryan|Bryan Jenks]]
> - **Publisher**:: [[youtube|YouTube]]
> - **Series**:: [undefined](undefined)
> - **Date Published**:: 2021-04-05
>
> - **Completed**:: [[2023-06-07]]

---

## Video

### Embed

<iframe
		width="560"
		height="315"
		src="https://www.youtube.com/embed/43KOH-l-SYo"
		title="Best Ways to Learn Javascript with Anki and Obsidian"
		frameborder="0"
		allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
		allowfullscreen
		>
  </iframe>

### Timestamp Notes

Timestamp hotkey: `Ctrl + Alt + T`

```timestamp-url
 https://www.youtube.com/watch?v=43KOH-l-SYo&t=1002s&ab_channel=BryanJenks
```

---

## Related Personal Knowledge

| `BUTTON[button-pkm-definition]` | `BUTTON[button-pkm-concept]` | `BUTTON[button-pkm-quote]` | `BUTTON[button-pkm-idea]` |
|:------------- |:------------- |:------------- |:------------- |
| QEC | `BUTTON[button-pkm-question]` | `BUTTON[button-pkm-evidence]` | `BUTTON[button-pkm-conclusion]` |
| PSA | `BUTTON[button-pkm-problem]` | `BUTTON[button-pkm-steps]` | `BUTTON[button-pkm-answer]` |

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

### Outgoing Library Links

<!-- Link related library content here -->

### Library Content

```dataview
TABLE WITHOUT ID
	link(file.name, file.frontmatter.aliases[0]) AS Title,
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
    link(T.section,
        regexreplace(
            regexreplace(T.text, "(#task)|(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual)\s*\[.*$", ""),
        "_$", ""))
    AS Task,
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
