---
title: baruch_lior
uuid: 3b26a30d-7219-41ca-bcc1-a9f200c06325
aliases:
  - Lior Baruch
  - Baruch, Lior
  - ליאור ברוך
  - ברוך, ליאור
  - baruch_lior
name_first: Lior
name_last: Baruch
name_last_maiden: null
gender: female
date_birth: null
date_death:
country: israel
city: haifa
address:
utc: +02:00
dst: +03:00
phone_mobile: 050-391-9883
phone_home:
phone_work:
email_personal:
email_work:
url:
linkedin_url: https://www.linkedin.com/in/lior-baruch-903917192/
organization:
job_title:
connection:
  - personal
source:
  - "[[brodkey_ari|Ari Brodkey]]"
picture_path: baruch_lior_pic.jpg
file_class: dir_contact
date_created: 2023-11-20T11:17
date_modified: 2023-11-20T11:19
tags:
---
# Lior Baruch

![[baruch_lior_pic.webp|200]]

> [!contact] Contact Details
>
> **PERSONAL**
>
> - **Connection**: `dv: map(this.file.frontmatter.connection, (x) => (upper(substring(x, 0, 1)) + substring(x, 1)))`
> - **Source**: `dv: map(this.file.frontmatter.source, (x) => choice(x = "job_application", "Job Application", choice(regexmatch("^\w", x), (upper(substring(x, 0, 1)) + substring(x, 1)), x)))`
> - **Birthday**: `dv: choice(regextest("0001", this.file.frontmatter.date_birth), dateformat(date(this.file.frontmatter.date_birth), "MMMM dd"), dateformat(date(this.file.frontmatter.date_birth), "MMMM dd, yyyy"))`
>
> **CONTACT**
>
> - **Mobile**: `dv: this.file.frontmatter.phone_mobile`
> - **Personal Email**: `dv: this.file.frontmatter.email_personal`
> - **Work Email**: `dv: this.file.frontmatter.email_work`
>
> **PROFESSIONAL**
>
> - **Website**: `dv: elink(this.file.frontmatter.url, this.file.frontmatter.aliases[0] + " Website")`
> - **LinkedIn**: `dv: elink(this.file.frontmatter.linkedin_url, this.file.frontmatter.aliases[0] + " LinkedIn")`
> - **Job Title**: `dv: this.file.frontmatter.job_title`
> - **Organization**: `dv: this.file.frontmatter.organization`

---

## Work Experience

> [!toc] [[baruch_lior#Lior Baruch|Contents]]
>
> | [[baruch_lior#Work Experience\|Experience]] | [[baruch_lior#Education\|Education]] | [[baruch_lior#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |
> | [[baruch_lior#Related Tasks and Events\|Tasks]] | [[baruch_lior#Related Knowledge\|PKM]] | [[baruch_lior#Related Library Content\|Library]] |

<!-- Copy experience details from LinkedIn here  -->

---

## Education

> [!toc] [[baruch_lior#Lior Baruch|Contents]]
>
> | [[baruch_lior#Work Experience\|Experience]] | [[baruch_lior#Education\|Education]] | [[baruch_lior#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |
> | [[baruch_lior#Related Tasks and Events\|Tasks]] | [[baruch_lior#Related Knowledge\|PKM]] | [[baruch_lior#Related Library Content\|Library]] |

<!-- Copy education details from LinkedIn here  -->

---

## Related Directory

> [!toc] [[baruch_lior#Lior Baruch|Contents]]
>
> | [[baruch_lior#Work Experience\|Experience]] | [[baruch_lior#Education\|Education]] | [[baruch_lior#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |
> | [[baruch_lior#Related Tasks and Events\|Tasks]] | [[baruch_lior#Related Knowledge\|PKM]] | [[baruch_lior#Related Library Content\|Library]] |

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

<!-- Link related contacts here  -->

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

<!-- Link related organizations here  -->

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

## Related Tasks and Events

> [!toc] [[baruch_lior#Lior Baruch|Contents]]
>
> | [[baruch_lior#Work Experience\|Experience]] | [[baruch_lior#Education\|Education]] | [[baruch_lior#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |
> | [[baruch_lior#Related Tasks and Events\|Tasks]] | [[baruch_lior#Related Knowledge\|PKM]] | [[baruch_lior#Related Library Content\|Library]] |

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

<!-- Link related tasks and events here  -->

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

> [!toc] [[baruch_lior#Lior Baruch|Contents]]
>
> | [[baruch_lior#Work Experience\|Experience]] | [[baruch_lior#Education\|Education]] | [[baruch_lior#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |
> | [[baruch_lior#Related Tasks and Events\|Tasks]] | [[baruch_lior#Related Knowledge\|PKM]] | [[baruch_lior#Related Library Content\|Library]] |

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

<!-- Link related pkm files here  -->

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
    AND contains(file.frontmatter.type, "tree")
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

> [!toc] [[baruch_lior#Lior Baruch|Contents]]
>
> | [[baruch_lior#Work Experience\|Experience]] | [[baruch_lior#Education\|Education]] | [[baruch_lior#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |
> | [[baruch_lior#Related Tasks and Events\|Tasks]] | [[baruch_lior#Related Knowledge\|PKM]] | [[baruch_lior#Related Library Content\|Library]] |

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

<!-- Link related library files here  -->

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
