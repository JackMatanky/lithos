---
title: 24_03_18_what_are_my_disagreements_with_sam
uuid: d786d15a-fb94-41a6-9df1-0c6680233515
aliases:
  - "What Are My Disagreements with Sam?"
  - "2024-03-18 What Are My Disagreements with Sam?"
  - "what are my disagreements with sam"
  - "what_are_my_disagreements_with_sam"
  - "24_03_18_what_are_my_disagreements_with_sam"
date: "[[2024-03-18]]"
pillar:
  - "[[mental_health|Mental Health]]"
  - "[[partner|Partner]]"
goal: null
project:
  - "[[coaching_with_nir_zer|Coaching with Nir Zer]]"
parent_task:
  - "[[coaching_assignments|Coaching Assignments]]"
subtype: general
type: prompt
file_class: pdev_journal
date_created: 2024-03-18T12:09
date_modified: 2024-03-18T12:09
tags:
---
# What Are My Disagreements with Sam?

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
> What are my disagreements with Sam?

1. We disagree about finances
2. Number of children
