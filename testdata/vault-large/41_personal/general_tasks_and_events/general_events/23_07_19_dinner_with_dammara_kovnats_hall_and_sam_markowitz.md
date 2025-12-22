---
title: 23_07_19_dinner_with_dammara_kovnats_hall_and_sam_markowitz
uuid: ddd58fd7-3da9-4142-beb0-9fcdbd60efab
aliases:
  - Dinner with Dammara Kovnats-hall and Sam Markowitz
  - 2023-07-19 Dinner with Dammara Kovnats-hall and Sam Markowitz
  - dinner with dammara kovnats-hall and sam markowitz
  - dinner_with_dammara_kovnats_hall_and_sam_markowitz
  - 2023-07-19_dinner_with_dammara_kovnats_hall_and_sam_markowitz
date: "[[2023-07-19]]"
due_do: do
pillar:
  - "[[friends_social_life|Friends and Social Life]]"
context: personal
goal: null
project:
  - "[[general_tasks_and_events|General Tasks and Events]]"
parent_task:
  - "[[general_events|General Events]]"
organization:
contact:
  - "[[markowitz_sam|Sam Markowitz]]"
library:
type: hangout
file_class: task_child
date_created: 2023-07-18T19:07
date_modified: 2024-05-31T14:33
tags: task
---
# Dinner with Dammara Kovnats-hall and Sam Markowitz

> [!meeting] Meeting Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

## Meeting

- [x] #task Dinner with Dammara Kovnats-hall and Sam Markowitz_hangout [time_start:: 20:30]  [time_end:: 23:00]  [duration_est:: 150] ⏰ 2023-07-19 20:25 ➕ 2023-07-18 📅 2023-07-19 ✅ 2023-07-19

---

## Prepare and Reflect

### Preview

> [!task_preview] Action Item Preview
>
> 1. What is the problem to solve?
>     - **Problem**::
>
> 2. What are possible solutions?
>     - **Solution**::

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

### Outgoing Task and Events Links

<!-- Link related tasks and events here -->

### Project and Parent Task

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    choice(contains(file.frontmatter.type, "project"), "🏗️Project", "⚒️Parent Task") AS Type,
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
    Objective AS Objective,
    outcome AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND (contains(this.file.frontmatter.project, file.name)
            OR contains(this.file.frontmatter.parent_task, file.name))
    AND contains(file.frontmatter.file_class, "task")
    AND (contains(file.frontmatter.type, "project")
            OR contains(file.frontmatter.type, "parent"))
SORT
    choice(contains(file.frontmatter.type, "project"), 1, 2) ASC
```

### Sibling Child Tasks

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
    outcome AS Result
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
    AND contains(this.file.frontmatter.parent_task, file.frontmatter.parent_task)
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "child")
    AND regextest("(#task)", T.text)
SORT
    dateformat(T.due, "yy-MM-dd"),
    T.time_start ASC
```

### General Child Tasks

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

## Related Personal Knowledge

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
