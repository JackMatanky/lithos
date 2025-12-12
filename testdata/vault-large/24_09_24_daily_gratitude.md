---
title: 24_09_24_daily_gratitude
uuid: 8e6540aa-c30c-4fd0-ae87-9ce20bf13811
aliases:
  - Daily Gratitude
  - Daily Gratitude Journal
  - 2024-09-24 Daily Gratitude Journal
  - 2024-09-24 Daily Gratitude
  - 2024-09-24 Gratitude
  - 24_09_24_daily_gratitude_journal
  - 24_09_24_daily_gratitude
  - 24_09_24_gratitude
date: "[[2024-09-24]]"
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
date_created: 2024-09-24T12:18
date_modified: 2024-09-24T13:29
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

1. **Gratitude**:: Getting to see [[muller_carli|Carli Muller]] last night at Sarona Market.
2. **Gratitude**:: The chance to use Python at Hive.
3. **Gratitude**:: The rest of the week to learn and prepare for school.

---

## I Can Thank Myself For…

1. **Self Gratitude**::
2. **Self Gratitude**::
3. **Self Gratitude**::

---

## Related

[[2024-09-23]]
