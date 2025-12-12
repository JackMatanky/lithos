---
title: 24_09_26_daily_gratitude
uuid: 3367019a-9c41-487b-aa2c-46ffac5fa2fd
aliases:
  - "Daily Gratitude"
  - "Daily Gratitude Journal"
  - "2024-09-26 Daily Gratitude Journal"
  - "2024-09-26 Daily Gratitude"
  - "2024-09-26 Gratitude"
  - "24_09_26_daily_gratitude_journal"
  - "24_09_26_daily_gratitude"
  - "24_09_26_gratitude"
date: "[[2024-09-26]]"
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
date_created: 2024-09-26T13:06
date_modified: 2024-09-26T13:06
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

1. **Gratitude**::
2. **Gratitude**::
3. **Gratitude**::

---

## I Can Thank Myself For…

1. **Self Gratitude**::
2. **Self Gratitude**::
3. **Self Gratitude**::

---

## Related

[[2024-09-25]]
