---
title: _metadata_dictionary
aliases:
  - Metadata Dictionary
  - metadata dictionary
  - _metadata_dictionary
date_created: 2023-03-09T21:24
date_modified: 2023-09-21T12:01
tags: metadata, obsidian/metadata
fields: []
version: "2.0"
---
# Vault Metadata

> [!Rule]  
> The first alias in the array equals the file's full name

## Task Management

| Key          | Data Type | Possible Values                                       | Templates                                           |
| ------------ | --------- | ----------------------------------------------------- | --------------------------------------------------- |
| date         | date      | YYYY-MM-DD                                            | all task templates besides project and parent tasks |
| due_do       | binary    | due, do                                               | all task templates                                  |
| pillar       | string    | any pillar files in the 20_pillars directory          | all task templates                                  |
| context | string    | personal, habit_ritual, education, professional, work | all task templates                                  |
| goal         | string    | any goals files in the 30_goals directory             |                                                     |
| project      | string    | any project folder in the 40_projects directory       |                                                     |
| parent_task  | string    |                                                       |                                                     |
| status       | string    | to_do, in_progress, done, discarded, schedule         |                                                     |
|              |           |                                                       |                                                     |

## Custom Metadata

### Legend

```yaml
legend_domain: [calendar, directory, knowledge_management, system, task_management]
legend_division: {
  - calendar: [daily, weekly, monthly, quarterly, yearly]; 
  - directory: [people, organization]; 
  - pkm: [pkm_tree, library, pkm_lab, note, thought]; 
  - system: [attachment, obsidian, plugin, template]; 
  - task: [goal, project, habit]}
legend_class: 
```

### Task Management

#### General

```yaml
typegoal_outcome, project, parent_task, action_item, meeting, phone_call, event, appointment, habit, ritual_morning, ritual_work_start, ritual_work_end, ritual_evening]
context: [work, personal]
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: [due, do]
date: YYYY-MM-DD
date_start: YYYY-MM-DD
date_end: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
organization: % create dataview query for organization list %
contact: % create dataview query for contact list based on the organization chosen %
status: [to_do, in_progress, done, discarded, schedule]
```

#### Project

```yaml
file_class: project
type
context: [work, personal]
goal: % create dataview query for active goals %
due_do: [due, do]
date_start: YYYY-MM-DD
date_end: YYYY-MM-DD
organization: % create dataview query for organization list %
contact: % create dataview query for contact list based on the organization chosen %
status: [to_do, in_progress, done, discarded, schedule]
```

#### Parent Task

```yaml
file_class: task_parent
type
context: [work, personal]
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
due_do: [due, do]
date_start: YYYY-MM-DD
date_end: YYYY-MM-DD
organization: % create dataview query for organization list %
contact: % create dataview query for contact list based on the organization chosen %
status: [to_do, in_progress, done, discarded, schedule]
```

#### Action Item

```yaml
file_class: task_child
type
context: [work, personal]
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
organization: % create dataview query for organization list %
contact: % create dataview query for contact list based on the organization chosen %
status: [to_do, in_progress, done, discarded, schedule]
```

#### Meeting

```yaml
file_class: meeting
type
context: [work, personal]
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
organization: % create dataview query for organization list %
contact: % create dataview query for contact list based on the organization chosen %
status: [schedule, to_do, in_progress, awaiting_approval, done, discarded, on_hold]
```

#### Phone Call

```yaml
file_class: phone_call
type
context: [work, personal]
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
organization: % create dataview query for organization list %
contact: % create dataview query for contact list based on the organization chosen %
status: [schedule, to_do, in_progress, awaiting_approval, done, discarded, on_hold]
```

#### Habit

```yaml
file_class: habit
type
context: [work, personal]
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
status: [schedule, to_do, in_progress, awaiting_approval, done, discarded, on_hold]
```

#### Morning Ritual

```yaml
file_class: ritual_morning
typeg
context: personal
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
status: [schedule, to_do, in_progress, awaiting_approval, done, discarded, on_hold]
```

#### Workday Startup Ritual

```yaml
file_class: ritual_work_start
typetart
context: personal 
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
status: [schedule, to_do, in_progress, awaiting_approval, done, discarded, on_hold]
```

#### Workday Shutdown Ritual

```yaml
file_class: ritual_work_end
typend
context: personal 
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
status: [schedule, to_do, in_progress, awaiting_approval, done, discarded, on_hold]
```

#### Evening Ritual

```yaml
file_class: ritual_evening
typeg
context: personal 
goal: % create dataview query for active goals %
project: % create dataview query for active projects %
parent_task: % create dataview query for active parent tasks %
due_do: do
date: YYYY-MM-DD
time_start: HHmm
time_end: HHmm
reminder_date: YYYY-MM-DD
reminder_time: HHmm
status: [schedule, to_do, in_progress, awaiting_approval, done, discarded, on_hold]
```

### Directory

#### Contact

```yaml
title:
name_first:
name_last:
name_last_maiden:
phone_mobile:
phone_home:
phone_work:
email_personal:
email_work:
linkedin_url:
url:
date_birth:
date_death:
contact_connection:
contact_source:
organization_current:
organization_previous:
job_title:
address:
city:
country:
file_class: dir_contact
date_created: 
date_modified:
```

#### Organization

## Plugin Metadata

### Book Search

```yaml
title: The title of the book.
author: The name of the book author. It can be multiple people.
category: Book category
description: Book description
publisher: The publisher of the book.
totalPage: The total number of pages in the book
coverUrl: Book cover image URL
publishDate: The year the book was published.
isbn10: ISBN10
isbn13: ISBN13
---
```

### ExcaliBrain

```yaml
Children: 
Parents: 
Friends: 
Siblings: 
```

### Reminder

```yaml
reminder_date: YYYY-MM-DD
reminder_time: HHmm
```

## All Keys

| Name     | Description       | Possible Values |
| -------- | ----------------- | --------------- |
| [[vault_legend]] | relevant keywords | see [[vault_legend]]    |
|          |                   |                 |
