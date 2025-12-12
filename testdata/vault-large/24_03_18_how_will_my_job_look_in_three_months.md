---
title: 24_03_18_how_will_my_job_look_in_three_months
uuid: ed5b202c-7acf-4038-8355-51a8367efaba
aliases:
  - How Will My Job Look in Three Months?
  - 2024-03-18 How Will My Job Look in Three Months?
  - how will my job look in three months
  - how_will_my_job_look_in_three_months
  - 24_03_18_how_will_my_job_look_in_three_months
date: "[[2024-03-18]]"
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
date_created: 2024-03-18T11:08
date_modified: 2024-03-18T11:12
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

1. I will feel fulfilled
2. I will be excited to interact with colleagues.
