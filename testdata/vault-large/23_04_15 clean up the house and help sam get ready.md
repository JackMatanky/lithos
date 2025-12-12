---
title: 23_04_15 clean up the house and help sam get ready
uuid: cb9d30fb-5cd2-47b3-b1b3-b6623af04853
aliases:
  - Clean up the house and help Sam get ready
  - clean up the house and help sam get ready
date: "[[2023-04-15]]"
due_do: do
pillar:
  - "[[partner|Partner]]"
context: personal
goal:
project:
  - "[[general_tasks_and_events|General Tasks and Events]]"
parent_task:
  - "[[house_chores|House Chores]]"
organization:
contact:
library:
type: action_item
file_class: task_child
date_created: 2023-04-15T00:00
date_modified: 2024-05-31T14:35
tags: task
---
# Clean up the House and Help Sam Get Ready

> [!action_item] Action Item Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

## Tasks

- [x] #task Clean up the house and help Sam get ready_action_item [time_start:: 11:45]  [time_end:: 12:30]  [duration_est:: 45] ⏰ 2023-04-15 11:35 ➕ 2023-04-15 📅 2023-04-15 ✅ 2023-04-15

## Notes
