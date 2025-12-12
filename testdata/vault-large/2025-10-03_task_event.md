---
title: 2025-10-03_task_event
uuid: 6222a6ae-63bd-4024-bf1f-042bf7249198
aliases:
  - Tasks and Events for Friday, October 3, 2025
  - Tasks and Events for October 3, 2025
  - 25-10-3_task_event
  - 2025-10-03_task_event
date: 2025-10-03
year: 2025
year_day: 276
quarter: 4
month_name: October
month_number: 10
month_day: 03
week_number: 40
weekday_name: Friday
weekday_number: 05
type: task_event
file_class: cal_day
cssclasses:
  - /inline_title_hide
  - /read_view_zoom
  - /read_wide_margin
date_created: 2025-09-24T15:14
date_modified: 2025-09-24T15:22
---
# Tasks and Events for Friday, October 3, 2025

> [!day] Day `dv: link(this.file.frontmatter.date, "Context")`
>
> | Year | Quarter | Month | Week |
> |:--------: |:--------: |:--------: |:--------: |
> | [[2025]] | [[2025-Q4]] | [[2025-10\|Oct '25]] | [[2025-W40]] |
>
> > [!dir] Subfiles
>
> | [[2025-10-03_pdev\|PDEV]] | [[2025-10-03_pkm\|PKM]] | [[2025-10-03_task_event\|Tasks and Events]] |
> |:--------: |:--------: |:--------: |:--------: |

<< `dvjs: dv.fileLink(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date).minus({days: 1}).toFormat("yyyy-MM-dd") + "_" + dv.current().file.frontmatter.type, false, dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date).minus({days: 1}).toFormat("yyyy-MM-dd") + " " + (dv.current().file.frontmatter.type.includes("_")? dv.current().file.frontmatter.type.split("_").map((x) => x[0].toUpperCase() + x.slice(1) + "s").join(" and "): (dv.current().file.frontmatter.type == "lib"? "Library": dv.current().file.frontmatter.type.toUpperCase())))` | `dvjs: dv.fileLink(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date).plus({days: 1}).toFormat("yyyy-MM-dd") + "_" + dv.current().file.frontmatter.type, false, dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date).plus({days: 1}).toFormat("yyyy-MM-dd") + " " + (dv.current().file.frontmatter.type.includes("_")? dv.current().file.frontmatter.type.split("_").map((x) => x[0].toUpperCase() + x.slice(1) + "s").join(" and "): (dv.current().file.frontmatter.type == "lib"? "Library": dv.current().file.frontmatter.type.toUpperCase())))` >>

---

## Tasks and Events

> [!task] Tasks and Events
>
> `BUTTON[button-project-task-table]`|`BUTTON[button-parent-task-table]`|`BUTTON[button-action-item-task-table]`|`BUTTON[button-meeting-task-table]`

### Planned for Today

- plan_total:: `dvjs: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks.filter((t) => dv.equal(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date), dv.luxon.DateTime.fromISO(t.due))).length` | plan_task_event:: `dvjs: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks.filter((t) => dv.equal(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date), dv.luxon.DateTime.fromISO(t.due)) &&!(t.text.includes("ritual") || t.text.includes("habit"))).length` | plan_habit_rit:: `dvjs: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.tasks.filter((t) => dv.equal(dv.luxon.DateTime.fromISO(dv.current().file.frontmatter.date), dv.luxon.DateTime.fromISO(t.due)) && (t.text.includes("ritual") || t.text.includes("habit"))).length`

> [!toc] Day Tasks and Events `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[2025-10-03_task_event#Planned for Today\|Plan]] | [[2025-10-03_task_event#Due Today\|Due]] | [[2025-10-03_task_event#Completed Today\|Done]] | [[2025-10-03_task_event#Created Today\|New]] |
> |:--------: |:--------: |:--------: |:--------: |

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
    T.time_start AS Start,
    T.time_end AS End,
    choice(length(file.frontmatter.parent_task) < 2, file.frontmatter.parent_task[0], flat(file.frontmatter.parent_task)) AS "Parent Task",
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
    contains(file.frontmatter.file_class, "task")
    AND regextest("#task", T.text)
    AND date(T.due) = date(this.file.frontmatter.date)
    AND !(T.status = "-"
      OR T.status = "<")
SORT
    T.due,
    T.time_start ASC
```

### Due Today

> [!toc] Day Tasks and Events `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[2025-10-03_task_event#Planned for Today\|Plan]] | [[2025-10-03_task_event#Due Today\|Due]] | [[2025-10-03_task_event#Completed Today\|Done]] | [[2025-10-03_task_event#Created Today\|New]] |
> |:--------: |:--------: |:--------: |:--------: |

### Completed Today

> [!toc] Day Tasks and Events `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[2025-10-03_task_event#Planned for Today\|Plan]] | [[2025-10-03_task_event#Due Today\|Due]] | [[2025-10-03_task_event#Completed Today\|Done]] | [[2025-10-03_task_event#Created Today\|New]] |
> |:--------: |:--------: |:--------: |:--------: |

### Created Today

> [!toc] Day Tasks and Events `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[2025-10-03_task_event#Planned for Today\|Plan]] | [[2025-10-03_task_event#Due Today\|Due]] | [[2025-10-03_task_event#Completed Today\|Done]] | [[2025-10-03_task_event#Created Today\|New]] |
> |:--------: |:--------: |:--------: |:--------: |
