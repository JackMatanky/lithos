---
title: 24_09_02_daily_gratitude
uuid: 1776f06a-4970-4bf1-bf7e-a193d716faeb
aliases:
  - Daily Gratitude
  - Daily Gratitude Journal
  - 2024-09-02 Daily Gratitude Journal
  - 2024-09-02 Daily Gratitude
  - 2024-09-02 Gratitude
  - 24_09_02_daily_gratitude_journal
  - 24_09_02_daily_gratitude
  - 24_09_02_gratitude
date: "[[2024-09-02]]"
pillar:
  - "[[mental_health|Mental Health]]"
goal: null
project:
  - "[[2024-09_habit_ritual|September '24 Habits and Rituals]]"
parent_task:
  - "[[2024-09_02_morning_rituals|September '24 Morning Rituals]]"
subtype: daily
type: gratitude
file_class: pdev_journal
date_created: 2024-09-02T11:34
date_modified: 2024-09-02T16:05
tags:
---
# Daily Gratitude Journal

> [!gratitude] Daily Gratitude Journal Details
>
> - **Type**: `dv: choice(contains(this.file.frontmatter.file_class, "journal"), choice(!regextest("\w", this.file.frontmatter.subtype), "Limiting Belief", join(map([this.file.frontmatter.subtype, this.file.frontmatter.type], (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ")), join(map(split(this.file.frontmatter.type, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ") + " Vision")`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

## I Am Grateful For…

1. **Gratitude**:: The chance to work on Hive's survey platform.
2. **Gratitude**:: [[kalkstein_kayla|Kayla Kalkstein]] sending me a message.
3. **Gratitude**:: Sam doing her best to give me space to prepare for school beyond errands and bureaucracy.

---

## I Can Thank Myself For…

1. **Self Gratitude**:: Accepting that today is just not my day, and that does not mean that I need to beat myself up.
2. **Self Gratitude**::
3. **Self Gratitude**::

---

## Related

[[2024-09-01]]
