---
title: 24_03_04_how_will_my_job_look_in_three_months
uuid: af968781-0a6d-4bbe-b0e9-a11f925cbf51
aliases:
  - How Will My Job Look in Three Months?
  - 2024-03-04 How Will My Job Look in Three Months?
  - how will my job look in three months
  - how_will_my_job_look_in_three_months
  - 24_03_04_how_will_my_job_look_in_three_months
date: "[[2024-03-04]]"
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
date_created: 2024-03-04T11:27
date_modified: 2024-03-04T11:34
tags:
---
# How Will My Job Look in Three Months?

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
> How will my job look in three months?

1. I will work on a team with professionally minded people from whom I can learn a lot.
2. I will feel satisfied by my day to day activities.
3. I will earn an income that enables me to provide for me and Sam.
4. I will look forward to coming to work.
