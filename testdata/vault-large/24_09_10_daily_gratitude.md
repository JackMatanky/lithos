---
title: 24_09_10_daily_gratitude
uuid: 753feb35-72a7-465f-bfb4-324cd94e50ee
aliases:
  - Daily Gratitude
  - Daily Gratitude Journal
  - 2024-09-10 Daily Gratitude Journal
  - 2024-09-10 Daily Gratitude
  - 2024-09-10 Gratitude
  - 24_09_10_daily_gratitude_journal
  - 24_09_10_daily_gratitude
  - 24_09_10_gratitude
date: "[[2024-09-10]]"
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
date_created: 2024-09-10T11:59
date_modified: 2024-09-10T16:01
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

1. **Gratitude**:: Being able to work on Hive's survey platform instead of continuing to work on research assistant tasks.
2. **Gratitude**:: Sam being nice when I came home late and upset from work.
3. **Gratitude**::

---

## I Can Thank Myself For…

1. **Self Gratitude**:: I started learning Khan Academy's Calculus prep course today.
2. **Self Gratitude**:: I took notes while learning.
3. **Self Gratitude**::

---

## Related

[[2024-09-09]]
