---
title: 24_09_10_daily_detachment
uuid: cbed92ba-3df6-4bc9-b498-2d9b61b26f27
aliases:
  - Daily Detachment
  - Daily Detachment Journal
  - 2024-09-10 Daily Detachment Journal
  - 2024-09-10 Daily Detachment
  - 2024-09-10 Detachment
  - 24_09_10_daily_detachment_journal
  - 24_09_10_daily_detachment
  - 24_09_10_detachment
date: "[[2024-09-10]]"
pillar:
  - "[[mental_health|Mental Health]]"
goal: null
project:
  - "[[2024-09_habit_ritual|September '24 Habits and Rituals]]"
parent_task:
  - "[[2024-09_01_habits|September '24 Habits]]"
subtype: daily
type: detachment
file_class: pdev_journal
date_created: 2024-09-10T11:59
date_modified: 2024-09-10T15:59
tags:
---
# Daily Detachment Journal

> [!detachment] Daily Detachment Journal Details
>
> - **Type**: `dv: choice(contains(this.file.frontmatter.file_class, "journal"), choice(!regextest("\w", this.file.frontmatter.subtype), "Limiting Belief", join(map([this.file.frontmatter.subtype, this.file.frontmatter.type], (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ")), join(map(split(this.file.frontmatter.type, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ") + " Vision")`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

> [!definition] Detachment Definition and Prompts
>
> Detachment refers to the separation of oneself from an outside occurrence. The object of detachment can be anything about which I can ask if it was correct for me.
>
> **Questions:**
> 1. What was/is the action, thought, or feeling?
> 2. What was/is the effect? Beneficial, Detrimental, Uncertain? Why?
> 3. How can I reframe the situation?
> 4. How can I prepare myself for the situation's recurrence?

---

## 1st Detachment

- **Object**:: I am pretty worried about how I am going to possibly be ready for when school starts.
- **Effect**:: Detrimental; I even forgot to call the dermatologist today.
- **Reframe**:: I am just anxious to start something new and time-consuming because of how I acted in the past, but the past is something I have learned from and I do not need to assume the present will be the same.
- **Prep**::

## 2nd Detachment

- **Object**::
- **Effect**::
- **Reframe**::
- **Prep**::

## 3rd Detachment

- **Object**::
- **Effect**::
- **Reframe**::
- **Prep**::

## 4th Detachment

- **Object**::
- **Effect**::
- **Reframe**::
- **Prep**::
