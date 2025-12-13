---
title: 23_07_17_three_month_vision
uuid: c10ca91f-c3d7-41b3-9354-9b20efd22d8a
aliases:
  - Three Month Vision
  - 2023-07-17 Three Month Vision
  - 23-07-17 three month vision
  - three_month_vision
  - 23_07_17_three_month_vision
date: "[[2023-07-17]]"
pillar:
  - "[[mental_health|Mental Health]]"
goal:
project:
  - "[[coaching_with_nir_zer|Coaching with Nir Zer]]"
parent_task:
  - "[[coaching_assignments|Coaching Assignments]]"
type: three_month
file_class: pdev_vision
date_created: 2023-07-17T15:04
date_modified: 2023-09-05T19:13
tags:
---
# Three Month Vision

> [!vision] Three Month Vision Details
>
> - **Type**: `dv: choice(contains(this.file.frontmatter.file_class, "journal"), choice(!regextest("\w", this.file.frontmatter.subtype), "Limiting Belief", join(map([this.file.frontmatter.subtype, this.file.frontmatter.type], (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ")), join(map(split(this.file.frontmatter.type, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ") + " Vision")`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`
> - **Parent Task**: `dv: this.file.frontmatter.parent_task`
> - **Date**: `dv: this.file.frontmatter.date`

---

> [!question] Prompt
>
> Write in the present tense as if it is three months in the future.
>
> Where do I want to be and how do I want to feel in three months?

### Mental Health

- I want to feel less stressed and capable of handling a large amount of stress when is arises.
- I want to feel happy about what I am doing.
- I want to feel excited about at least a fourth of what I do each day.

### Personal

- I want to build connections with family and friends who support me.
- I want to find two things about which I am passionate.
- I want to get back to reading each day.
- I want to build up my learning habits.
- I want to cut out my unhealthy habits.

### Professional

- I want to be excited about looking for a job.
- I want to be continuing my education to feel relevant in my field.
- I want to find a job that fulfills me.
