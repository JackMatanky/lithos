---
title: 24_02_21_reasons_to_hire_me
uuid: 3900a9fd-9e8e-4cb1-ada8-e977fb181a79
aliases:
  - Reasons to Hire Me
  - 2024-02-21 Reasons to Hire Me
  - reasons to hire me
  - reasons_to_hire_me
  - 24_02_21_reasons_to_hire_me
date: "[[2024-02-21]]"
pillar:
  - "[[mental_health|Mental Health]]"
  - "[[career_development|Career Development]]"
goal: null
project:
  - "[[job_hunting_2023|Job Hunting 2023]]"
parent_task:
  - "[[forter_fraud_analyst|Fraud Analyst at Forter]]"
subtype: general
type: prompt
file_class: pdev_journal
date_created: 2024-02-21T12:29
date_modified: 2024-02-21T12:32
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

- I come ready and excited to learn.
- I can look at a problem and see multiple perspectives.
- I look to understand problems deeply.
- I aim to understand processes to structure them toward innovation.
- I work with scalability in mind.
- I am organized
- I work hard and diligently.
