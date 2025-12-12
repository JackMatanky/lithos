---
title: 23_10_30_clean_up_laptop_storage_and_fix_other_issues
uuid: 77024ba8-0298-45af-8251-3c344133c26b
aliases:
  - Clean Up Laptop Storage and Fix Other Issues
  - 23-10-30 Clean Up Laptop Storage and Fix Other Issues
  - clean up laptop storage and fix other issues
  - clean_up_laptop_storage_and_fix_other_issues
  - 23_10_30_clean_up_laptop_storage_and_fix_other_issues
date: "[[2023-10-30]]"
due_do: do
pillar:
context: personal
goal: null
project:
  - "[[general_tasks_and_events|General Tasks and Events]]"
parent_task:
  - "[[general_tasks|General Tasks]]"
organization:
contact:
library:
type: action_item
file_class: task_child
date_created: 2023-10-30T18:04
date_modified: 2024-05-31T14:34
tags: task
---
# Clean Up Laptop Storage and Fix Other Issues

> [!action_item] Action Item Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

## Tasks and Events

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Tasks and Events\|Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Prepare and Reflect\|Insight]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Tasks and Events\|Related Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Knowledge\|PKM]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Library Content\|Library]] |
> |:--------: |:--------: |:--------: |:--------: |:--------: |

- [x] #task Clean Up Laptop Storage and Fix Other Issues_action_item [time_start:: 11:00]  [time_end:: 16:15]  [duration_est:: 315] ⏰ 2023-10-30 10:55 ➕ 2023-10-30 📅 2023-10-30 ✅ 2023-10-30

---

## Prepare and Reflect

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Tasks and Events\|Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Prepare and Reflect\|Insight]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Tasks and Events\|Related Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Knowledge\|PKM]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Library Content\|Library]] |
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
> 4. What don't I do while performing the task?
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
> | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Tasks and Events\|Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Prepare and Reflect\|Insight]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Tasks and Events\|Related Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Knowledge\|PKM]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Library Content\|Library]] |
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
		(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → " + dateformat(date(regexreplace(file.frontmatter.task_end, "[^\d-]", "")), "yy-MM-dd")),
		choice(regextest("\d", file.frontmatter.task_start),
			(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → Present"),
			"NULL"))
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
    link(T.link, regexreplace(T.text, "#tasks(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
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
    AND regextest("#task", T.text)
SORT
    T.due,
    T.time_start ASC
```

### General Child Tasks

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#tasks(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
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
> | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Tasks and Events\|Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Prepare and Reflect\|Insight]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Tasks and Events\|Related Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Knowledge\|PKM]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Library Content\|Library]] |
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
> | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Tasks and Events\|Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Prepare and Reflect\|Insight]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Tasks and Events\|Related Tasks]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Knowledge\|PKM]] | [[23_10_30_clean_up_laptop_storage_and_fix_other_issues#Related Library Content\|Library]] |
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
