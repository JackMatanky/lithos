---
title: 24_02_26_what_will_my_job_look_like_in_three_months
uuid: a8f2c7d5-df8e-4ef5-b184-e9c820570e8b
aliases:
  - What Will My Job Look Like in Three Months
  - 2024-02-26 What Will My Job Look Like in Three Months
  - what will my job look like in three months
  - what_will_my_job_look_like_in_three_months
  - 24_02_26_what_will_my_job_look_like_in_three_months
date: "[[2024-02-26]]"
pillar:
  - "[[mental_health|Mental Health]]"
  - "[[career_development|Career Development]]"
goal: null
project:
  - "[[coaching_with_nir_zer|Coaching with Nir Zer]]"
parent_task:
  - "[[coaching_assignments|Coaching Assignments]]"
subtype: general
type: prompt
file_class: pdev_journal
date_created: 2024-02-26T13:37
date_modified: 2024-02-26T13:39
tags:
---
# What Will My Job Look Like in Three Months

> [!prompt] General Prompt Journal Details
>
> - **Type**: `dv: choice(contains(this.file.frontmatter.file_class, "journal"), choice(!regextest("\w", this.file.frontmatter.subtype), "Limiting Belief", join(map([this.file.frontmatter.subtype, this.file.frontmatter.type], (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ")), join(map(split(this.file.frontmatter.type, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ") + " Vision")`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Project**: `dv: this.file.frontmatter.project`
> - **Parent Task**: `dv: this.file.frontmatter.parent_task`
> - **Date**: `dv: this.file.frontmatter.date`

---

## Prompt

> [!question] Prompt
>
> What will my job look like in three months

In three months, I will be working as an analyst at a job that fulfills me and provides me with a respectable income. I will be challenged and find purpose in accomplishing the tasks put in front of me. I will learn from the analysts who surround me.
