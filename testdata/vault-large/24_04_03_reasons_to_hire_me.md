---
title: 24_04_03_reasons_to_hire_me
uuid: 55b105f1-c22d-465e-b4d6-da242b968569
aliases:
  - Reasons to Hire Me
  - 2024-04-03 Reasons to Hire Me
  - reasons to hire me
  - reasons_to_hire_me
  - 24_04_03_reasons_to_hire_me
date: "[[2024-04-03]]"
pillar:
  - "[[mental_health|Mental Health]]"
  - "[[career_development|Career Development]]"
goal: null
project:
  - "[[job_hunting_2023|Job Hunting 2023]]"
parent_task:
  - "[[pontera_financial_operation_analyst|Financial Operation Analyst at Pontera]]"
subtype: general
type: prompt
file_class: pdev_journal
date_created: 2024-04-03T13:22
date_modified: 2024-04-03T13:26
tags:
---
# Reasons to Hire Me

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
> What are seven reasons why a company would want to hire me?

1. I am a learner and I do it well
2. I make sure the job gets done.
3. I am always looking to improve and grow.
4. I am organized and enable scalable solutions.
5. I work well on a team.
6. I look for ways to innovate.
