---
title: 11_appendix_b_real_analysis
uuid: 741266ef-88e9-42d5-8b40-196e521441ee
aliases:
  - "Real Analysis: Appendix B, Peculiar and Pathological Examples"
  - "Appendix B: Peculiar and Pathological Examples"
  - "11. Appendix B: Peculiar and Pathological Examples"
  - Appendix B
  - appendix_b
  - appendix_b_peculiar_and_pathological_examples
  - real_analysis_appendix_b
  - 11_appendix_b_real_analysis
main_title: Appendix B
subtitle: Peculiar and Pathological Examples
author:
  - "[[cummings_jay|Jay Cummings]]"
editor:
translator:
year_published: 2019
publisher:
page_start: 379
page_end: 428
doi:
url: https://longformmath.com/analysis-home
library:
  - "[[cummings_2019_real_analysis|Real Analysis: A Long-form Mathematics Textbook]]"
cssclasses:
status: undetermined
type: book_chapter
file_class: lib_book_chapter
date_created: 2024-12-22T19:42
date_modified: 2025-10-05T17:48
tags:
---
# 11. Appendix B: Peculiar and Pathological Examples

> [!book_chapter] Book Chapter Details
>
> - **Author**: `dv: this.file.frontmatter.author`
> - **Chapter**: `dv: this.file.frontmatter.aliases[0]`
> - **Book**: `dv: this.file.frontmatter.library[0]`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Date Published**: `dv: this.file.frontmatter.year_published`
> - **Pages**: `dv: this.file.frontmatter.page_start + " - " + this.file.frontmatter.page_end`
>
> **Completed**::

---

<!-- Insert chapter content here -->

![[Cummings_2019_Real Analysis_11_Appendix B Peculiar and Pathological Examples.pdf]]

---

## Related Knowledge

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[11_appendix_b_real_analysis#Related Knowledge\|PKM]] | [[11_appendix_b_real_analysis#Related Library Content\|Library]] | [[11_appendix_b_real_analysis#Related Tasks and Events\|Related Tasks]] | [[11_appendix_b_real_analysis#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

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

- [[calculus|Calculus]]

### Knowledge Tree

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
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
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
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
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "quote": 5,
      "idea": 6,
      "summary": 7,
      "concept": 8
    }[x])(file.frontmatter.type), 9),
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
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "quote": 5,
      "idea": 6,
      "summary": 7,
      "concept": 8
    }[x])(file.frontmatter.type), 9),
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
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "quote": 5,
      "idea": 6,
      "summary": 7,
      "concept": 8
    }[x])(file.frontmatter.type), 9),
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
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "quote": 5,
      "idea": 6,
      "summary": 7,
      "concept": 8
    }[x])(file.frontmatter.type), 9),
	file.frontmatter.title ASC
```

---

## Related Library Content

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[11_appendix_b_real_analysis#Related Knowledge\|PKM]] | [[11_appendix_b_real_analysis#Related Library Content\|Library]] | [[11_appendix_b_real_analysis#Related Tasks and Events\|Related Tasks]] | [[11_appendix_b_real_analysis#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

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
	OR "inbox"
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

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[11_appendix_b_real_analysis#Related Knowledge\|PKM]] | [[11_appendix_b_real_analysis#Related Library Content\|Library]] | [[11_appendix_b_real_analysis#Related Tasks and Events\|Related Tasks]] | [[11_appendix_b_real_analysis#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

> [!task] Tasks and Events
>
> `BUTTON[button-project-task-table]`|`BUTTON[button-parent-task-table]`|`BUTTON[button-action-item-task-table]`|`BUTTON[button-meeting-task-table]`

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
    join(map(split(file.frontmatter.context, "_"),
      (x) => upper(x[0]) + substring(x, 1)),
      " and ")
    AS Context,
    Objective AS Objective,
    choice(regextest("\w", Outcome) AND regextest("\w", Feeling),
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)),
    choice(regextest("\w", Outcome) AND !regextest("\w", Feeling),
      ("**Outcome**: " + Outcome),
      "NULL")
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
    choice(regextest("\w", Outcome) AND regextest("\w", Feeling),
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)),
    choice(regextest("\w", Outcome) AND !regextest("\w", Feeling),
      ("**Outcome**: " + Outcome),
      "NULL")
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
    link(T.link, regexreplace(T.text, "#task \s(.+)_(action_|meeting|phone_call|video_call|interview|lecture|appointment|event|hangout|habit|gathering|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
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

## Related Directory

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[11_appendix_b_real_analysis#Related Knowledge\|PKM]] | [[11_appendix_b_real_analysis#Related Library Content\|Library]] | [[11_appendix_b_real_analysis#Related Tasks and Events\|Related Tasks]] | [[11_appendix_b_real_analysis#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

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
