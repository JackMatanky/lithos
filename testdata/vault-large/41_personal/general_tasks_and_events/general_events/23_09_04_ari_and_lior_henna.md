---
title: 23_09_04_ari_and_lior_henna
uuid: daae78cc-a6c3-4e91-a1c6-2afcbbb09178
aliases:
  - Ari and Lior Henna
  - 23-09-04 Ari and Lior Henna
  - ari and lior henna
  - ari_and_lior_henna
  - 23_09_04_ari_and_lior_henna
date: "[[2023-09-04]]"
due_do: do
pillar:
  - friends_social_life
context: personal
goal:
project:
  - "[[general_tasks_and_events|General Tasks and Events]]"
parent_task:
  - "[[general_events|General Events]]"
organization: []
contact:
  - brodkey_ari
  - baruch_lior
library:
subtype: event
type: meeting
file_class: task_child
date_created: 2023-09-04T13:14
date_modified: 2024-05-31T14:33
tags: task
---
# Ari and Lior Henna

> [!meeting] Meeting Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`

---

## Meeting

- [x] #task Ari and Lior Henna_meeting [time_start:: 15:30]  [time_end:: 23:30]  [duration_est:: 480] ⏰ 2023-09-04 15:25 ➕ 2023-09-04 📅 2023-09-04 ✅ 2023-09-05

---

## Prepare and Reflect

### Preview

> [!task_preview] Action Item Preview
>
> 1. What is the problem to solve?
>     - **Problem**::
>
> 2. What do I want?
>     - **Desire**::
>
> 3. What will I do?
>     - **Plan**::
>
> 4. What won't I do?
>     - **Refrain**::

### Plan

> [!task_plan] Execution Plan
>
> What is the detailed execution plan?
>
> - subtask short description
>     - [ ] subtask
> - subtask short description
>     - [ ] subtask
> - subtask short description
>     - [ ] subtask
> - subtask short description
>     - [ ] subtask

### Review

> [!task_review] Action Item Review
>
> 1. What was the outcome? How did it make me feel?
>     - **Outcome**::
>     - **Feeling**::
>
> 2. What went well?
>     - **Keep**::
>
> 3. What can be improved?
>     - **Improve**::
>
> 4. What can be started?
>     - **Start**::
>
> 5. What can be stopped?
>     - **Stop**::

---

## Related Tasks and Events

> [!task] Tasks and Events
>
> `BUTTON[button-project-task-table]`|`BUTTON[button-parent-task-table]`|`BUTTON[button-action-item-task-table]`|`BUTTON[button-meeting-task-table]`

<!-- Adjust replace lines -->

```meta-bind-button
label: 🔨Child Task Tasks and Events🤝
tooltip: "Replace the child task's tasks and events section with MD table of linked files and a filtered DataView table"
class: mb_button_blue
style: default
hidden: false
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/142_00_dvmd_task_sect_child.md
    templater: true
```

### Outgoing Task and Events Links

<!-- Link related tasks and events here -->

### Project and Parent Task

| Title | Type | Status | Dates | Objective | Result |
| ----- | ---- | ------ | ----- | --------- | ------ |

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    choice(contains(file.frontmatter.type, "project"), "🏗️Project", "⚒️Parent Task") AS Type,
    default(((x) => {
      "done": "✔️Done",
      "in_progress": "👟In progress",
      "to_do": "🔜To do",
      "schedule": "📅Schedule",
      "on_hold": "🤌On hold",
      "applied": "📨Applied💼",
      "offer": "📝Job Offer💼",
      "rejected": "🚫Rejected💼"
    }[x])(file.frontmatter.status), "❌Discarded")
	AS Status,
    choice((regextest("\d", file.frontmatter.task_start) AND regextest("\d", file.frontmatter.task_end)),
		(file.frontmatter.task_start + " → " + file.frontmatter.task_end),
		choice(regextest("\d", file.frontmatter.task_start),
			(file.frontmatter.task_start + " → Present"),
			"null"))
	AS Dates,
    Objective AS Objective,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND (contains(file.frontmatter.file_class, "project")
    OR contains(file.frontmatter.file_class, "parent"))
    AND (contains(this.file.frontmatter.project, file.name)
    OR contains(this.file.frontmatter.parent_task, file.name))
    AND !contains(file.inlinks, this.file.link)
SORT
    choice(contains(file.frontmatter.type, "project"), 1, 2) ASC
```

### Sibling Child Tasks

| Task                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | Type        | Status   | Date     | Result                                                                                                                                                                                                                                                                                                            |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- | -------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [[23_03_13_morning_email_and_slack_review#[[23_03_13_morning_email_and_slack_review|23_03_13_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_fix_hibob-tableau_integration#[[23_03_12_fix_hibob_tableau_integration|23_03_12_fix_hibob-tableau_integration\|fix elementor hibob-tableau integration]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_fix_hibob-tableau_integration#[[23_03_12_fix_hibob_tableau_integration|23_03_12_fix_hibob-tableau_integration\|Continue to fix Elementor HiBob-Tableau integration]]                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_schedule_id_social_media_content#[[23_03_13_schedule_id_social_media_content|23_03_12_schedule_id_social_media_content\|schedule ID content on Buffer]]                                                                                                                                                                                                                                                                                          | 🔨Task      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_12_work_on_kn_nam_dashboards.md\|23_03_12_work_on_kn_nam_dashboards\|double check filters in Name headcount dashboard [time_start:: 15:20]  [time_end:: 15:45] 📅 2023-03-12 ⏫ ✅ 2023-03-12]]                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_12_work_on_kn_nam_dashboards.md\|23_03_12_work_on_kn_nam_dashboards\|remove country-specific dashboards from KN server [time_start:: 15:45]  [time_end:: 16:05] 📅 2023-03-12 ✅ 2023-03-12]]                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_12_work_on_kn_nam_dashboards.md\|23_03_12_work_on_kn_nam_dashboards\|check data blending issue [time_start:: 16:05]  [time_end:: 16:10] 📅 2023-03-12 ✅ 2023-03-12]]                                                                                                                                                                                                                           | 🌇Rit.      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_12_work_on_kn_nam_dashboards.md\|23_03_12_work_on_kn_nam_dashboards\|change null BU/FU back to undefined and export records marked null [time_start:: 16:10]  [time_end:: 16:20] 📅 2023-03-12 ✅ 2023-03-12]]                                                                                                                                                                                  | 🌇Rit.      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_12_work_on_kn_nam_dashboards.md\|23_03_12_work_on_kn_nam_dashboards\|create brokerage filter from business title field [time_start:: 16:20]  [time_end:: 16:25] 📅 2023-03-12 ✅ 2023-03-12]]                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_morning_email_and_slack_review#[[23_03_12_morning_email_and_slack_review|23_03_12_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_team_lunch#[[23_03_12_team_lunch|23_03_12_team_lunch\|Team lunch [time_start:: 12:30]  [time_end:: 13:30] 📅 2023-03-12 ✅ 2023-03-12]]                                                                                                                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_12_evening_email_and_slack_review#[[23_03_12_evening_email_and_slack_review|23_03_12_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_check_extract_issue_on_headcount_dashboard#[[44_work/Elementor HR Dashboard Building/tasks/23_03_13_check_extract_issue_on_headcount_dashboard.md\|23_03_13_check_extract_issue_on_headcount_dashboard\|Check extract issue on headcount dashboard]]                                                                                                                                                                                                                                    | 🔨Task      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_13_work_on_kn_nam_dashboards.md\|23_03_13_work_on_kn_nam_dashboards\|write an email to Nico about how often the data is updated [time_start:: 10:30]  [time_end:: 10:40] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                                                                          | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_13_work_on_kn_nam_dashboards.md\|23_03_13_work_on_kn_nam_dashboards\|Republish KN country-specific dashboards [time_start:: 10:40]  [time_end:: 11:00] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                                                                                            | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_13_work_on_kn_nam_dashboards.md\|23_03_13_work_on_kn_nam_dashboards\|start to add all missing graphs regardless of how they look [time_start:: 11:00]  [time_end:: 12:50] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                                                                         | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_15_work_on_kn_nam_dashboards.md\|23_03_15_work_on_kn_nam_dashboards\|add missing graphs to USA HC DB [time_start::1215]  [time_end::1300] 📅 2023-03-13 ✅ 2023-03-15]]                                                                                                                                                                                                                       | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_13_work_on_kn_nam_dashboards.md\|23_03_13_work_on_kn_nam_dashboards\|export list of cost centers [time_start:: 12:50]  [time_end:: 12:55] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                                                                                                         | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_13_work_on_kn_nam_dashboards.md\|23_03_13_work_on_kn_nam_dashboards\|create collapsible filter toggle for country-specific dashboards [time_start:: 12:55]  [time_end:: 15:20] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                                                                    | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_onboarding_nimrod_to_platform_interview_upload_template#[[23_03_13_onboarding_nimrod_to_platform_interview_upload_template|23_03_13_onboarding_nimrod_to_platform_interview_upload_template\|Onboarding Nimrod to Platform Interview Upload Template [time_start:: 15:20]  [time_end:: 15:50] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                               | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_publish_id_social_media_content#[[44_work/ID Content Marketing/tasks/23_03_13_publish_id_social_media_content.md\|23_03_13_publish_id_social_media_content\|Publish ID Social Media Content]]                                                                                                                                                                                                                                                                                           | 🔨Task      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_meeting_to_work_on_tableau_prep_builder_flow#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_13_meeting_to_work_on_tableau_prep_builder_flow.md\|23_03_13_meeting_to_work_on_tableau_prep_builder_flow\|Meeting to work on Tableau Prep Builder Flow [time_start:: 16:00]  [time_end:: 17:30] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                    | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_call_with_ifat_about_programmatic_export_form#[[44_work/Elementor HR Dashboard Building/tasks/23_03_13_call_with_ifat_about_programmatic_export_form.md\|23_03_13_call_with_ifat_about_programmatic_export_form\|Call with Ifat about Programmatic Export Form]]                                                                                                                                                                                                                        | 🔨Task      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_kn_nam_dashboard_onboarding_continued#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_13_kn_nam_dashboard_onboarding_continued.md\|23_03_13_kn_nam_dashboard_onboarding_continued\|KN Name Dashboard Onboarding Continued [time_start:: 17:30]  [time_end:: 18:30] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                                              | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_miscellaneous_managerial_tasks#[[23_03_13_miscellaneous_managerial_tasks|23_03_13_miscellaneous_managerial_tasks\|Miscellaneous managerial tasks [time_start:: 18:30]  [time_end:: 19:15] 📅 2023-03-13 ✅ 2023-03-13]]                                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_13_evening_email_and_slack_review#[[23_03_13_evening_email_and_slack_review|23_03_13_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_review_user_feedback_for_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_15_review_user_feedback_for_kn_nam_dashboards.md\|23_03_15_review_user_feedback_for_kn_nam_dashboards\|Review User Feedback for KN Name Dashboards [time_start::1030]  [time_end::1130] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                              | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_check_dashboards_for_extract_issue#[[44_work/Elementor HR Dashboard Building/tasks/23_03_15_check_dashboards_for_extract_issue.md\|23_03_15_check_dashboards_for_extract_issue\|Check dashboards for extract issue]]                                                                                                                                                                                                                                                                    | 🔨Task      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_onboarding_nimrod_kn_exit_survey#[[23_03_15_onboarding_nimrod_kn_exit_survey|23_03_15_onboarding_nimrod_kn_exit_survey\|Onboarding Nimrod_KN Exit Survey [time_start::1300]  [time_end::1425] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                                                                                             | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_continue_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_15_continue_work_on_kn_nam_dashboards.md\|23_03_15_continue_work_on_kn_nam_dashboards\|add missing graphs regardless of how they look [time_start::1425]  [time_end::1530] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                                                             | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_continue_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_15_continue_work_on_kn_nam_dashboards.md\|23_03_15_continue_work_on_kn_nam_dashboards\|Check dashboard filters [time_start::1530]  [time_end::1600] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                                                                                    | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_exit_survey_monthly_updates#[[44_work/KN CA Exit Survey/Meetings/23_03_15_exit_survey_monthly_updates.md\|23_03_15_exit_survey_monthly_updates\|Exit Interview Monthly Updates [time_start::1600]  [time_end::1620] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                                                                                                                        | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_continue_to_work_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_15_continue_to_work_on_kn_nam_dashboards.md\|23_03_15_continue_to_work_on_kn_nam_dashboards\|work on KN Name Dashboards [time_start::1620]  [time_end::1800] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                                                                         | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_work_with_yi_on_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_15_work_with_yi_on_kn_nam_dashboards.md\|23_03_15_work_with_yi_on_kn_nam_dashboards\|Work with Yi on KN Name Dashboards [time_start::1800]  [time_end::1820] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                                                                             | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_morning_email_and_slack_review#[[23_03_15_morning_email_and_slack_review|23_03_15_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_miscellaneous_managerial_tasks#[[23_03_15_miscellaneous_managerial_tasks|23_03_15_miscellaneous_managerial_tasks\|Miscellaneous managerial tasks [time_start:: 18:20]  [time_end:: 18:50] 📅 2023-03-15 ✅ 2023-03-15]]                                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_15_evening_email_and_slack_review#[[23_03_15_evening_email_and_slack_review|23_03_15_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_16_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_16_work_on_kn_nam_dashboard.md\|23_03_16_work_on_kn_nam_dashboard\|Work on KN Name Dashboards [time_start::0900]  [time_end::1000] 📅 2023-03-16 ✅ 2023-03-16]]                                                                                                                                                                                                                                | 🌇Rit.      | ✔️Done   | 23-03-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_16_meeting_with_nico_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/meetings_weekly/23_03_16_meeting_with_nico_on_kn_nam_dashboard.md\|23_03_16_meeting_with_nico_on_kn_nam_dashboard\|Meeting with Nico on KN Name Dashboard [time_start::1000]  [time_end::1030] 📅 2023-03-16 ✅ 2023-03-16]]                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_16_continue_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_16_continue_work_on_kn_nam_dashboard.md\|23_03_16_continue_work_on_kn_nam_dashboard\|Continue Work on KN Name Dashboard [time_start::1030]  [time_end::1600] 📅 2023-03-16 ✅ 2023-03-16]]                                                                                                                                                                                             | 🌇Rit.      | ✔️Done   | 23-03-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_16_sync_tableau_prep_data_source#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_16_sync_tableau_prep_data_source.md\|23_03_16_sync_tableau_prep_data_source\|Tableau Prep Data Source Sync [time_start::1630]  [time_end::1800] 📅 2023-03-16 ✅ 2023-03-16]]                                                                                                                                                                                                  | 🌇Rit.      | ✔️Done   | 23-03-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_16_morning_email_and_slack_review#[[23_03_16_morning_email_and_slack_review|23_03_16_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_16_miscellaneous_managerial_tasks#[[23_03_16_miscellaneous_managerial_tasks|23_03_16_miscellaneous_managerial_tasks\|Miscellaneous managerial tasks [time_start:: 18:00]  [time_end:: 18:30] 📅 2023-03-16 ✅ 2023-03-16]]                                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_16_evening_email_and_slack_review#[[23_03_16_evening_email_and_slack_review|23_03_16_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_19_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_19_work_on_kn_nam_dashboard.md\|23_03_19_work_on_kn_nam_dashboard\|Check the data source from prep builder [time_start::1110]  [time_end::1145] ⏰2023-03-19-11:00 📅 2023-03-19 ✅ 2023-03-19]]                                                                                                                                                                                                | 🌇Rit.      | ✔️Done   | 23-03-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_19_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_19_work_on_kn_nam_dashboard.md\|23_03_19_work_on_kn_nam_dashboard\|Add country specific graphs to Name dashboard [time_start::1145]  [time_end::1330] ⏰2023-03-19-11:45 📅 2023-03-19 ✅ 2023-03-19]]                                                                                                                                                                                           | 🌇Rit.      | ✔️Done   | 23-03-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_19_sales_linkedin_outreach_strategy#[[44_work/ID Content Marketing/meetings/23_03_19_sales_linkedin_outreach_strategy.md\|23_03_19_sales_linkedin_outreach_strategy\|Sales LinkedIn outreach strategy]]                                                                                                                                                                                                                                                                                    | 🤝Meeting   | ✔️Done   | 23-03-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_19_continue_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_19_continue_work_on_kn_nam_dashboard.md\|23_03_19_continue_work_on_kn_nam_dashboard\|Check data validation issues I have already checked [time_start::1430]  [time_end::1530] ⏰2023-03-19-14:30 📅 2023-03-19 ✅ 2023-03-19]]                                                                                                                                                         | 🌇Rit.      | ✔️Done   | 23-03-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_19_continue_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_19_continue_work_on_kn_nam_dashboard.md\|23_03_19_continue_work_on_kn_nam_dashboard\|Review Prep Builder data source for missing fields [time_start::1530]  [time_end::1700] ⏰2023-03-19-15:30 📅 2023-03-19 ✅ 2023-03-19]]                                                                                                                                                          | 🌇Rit.      | ✔️Done   | 23-03-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_19_morning_email_and_slack_review#[[23_03_19_morning_email_and_slack_review|23_03_19_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_19_evening_email_and_slack_review#[[23_03_19_evening_email_and_slack_review|23_03_19_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_schedule_meeting_with_piotr_about_comeet-tableau_integration_and_its_steps_for_elementor_hr_dashboards#[[23_03_20_schedule_meeting_with_piotr_about_comeet_tableau_integration_and_its_steps_for_elementor_hr_dashboards|23_03_20_schedule_meeting_with_piotr_about_comeet-tableau_integration_and_its_steps_for_elementor_hr_dashboards\|Schedule meeting with Piotr about Elementor's Comeet-Tableau integration and its steps]] | 🔨Task      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_offboarding_meeting_with_shiran#[[23_03_20_offboarding_meeting_with_shiran|23_03_20_offboarding_meeting_with_shiran\|Offboarding Meeting with Shiran [time_start::1015]  [time_end::1100] ⏰2023-03-20-10:10 📅 2023-03-20 ✅ 2023-03-20]]                                                                                                                                                                                               | 🌇Rit.      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_20_work_on_kn_nam_dashboard.md\|23_03_20_work_on_kn_nam_dashboard\|Export Areas from KN Name Dashboard [time_start::1100]  [time_end::1115] 📅 2023-03-20 ✅ 2023-03-20]]                                                                                                                                                                                                                      | 🌇Rit.      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_20_work_on_kn_nam_dashboard.md\|23_03_20_work_on_kn_nam_dashboard\|Start to Create Navigation Page for KN Name Dashboard [time_start::1115]  [time_end::1200] 📅 2023-03-20 ✅ 2023-03-20]]                                                                                                                                                                                                    | 🌇Rit.      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_meeting_with_piotr_about_comeet-tableau_integration_and_its_steps_for_elementor_hr_dashboards#[[23_03_20_meeting_with_piotr_about_comeet_tableau_integration_and_its_steps_for_elementor_hr_dashboards|23_03_20_meeting_with_piotr_about_comeet-tableau_integration_and_its_steps_for_elementor_hr_dashboards\|Meeting with Piotr about Comeet-Tableau integration]]                                                               | 🤝Meeting   | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_continue_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_20_continue_work_on_kn_nam_dashboard.md\|23_03_20_continue_work_on_kn_nam_dashboard\|Continue to Create Navigation Page for KN Name Dashboard [time_start::1400]  [time_end::1530] 📅 2023-03-20 ✅ 2023-03-20]]                                                                                                                                                                       | 🌇Rit.      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_post_simon_sinek_poll_on_id_linkedin#[[44_work/ID Content Marketing/tasks/23_03_20_post_simon_sinek_poll_on_id_linkedin.md\|23_03_20_post_simon_sinek_poll_on_id_linkedin\|Post Simon Sinek Poll on ID LinkedIn]]                                                                                                                                                                                                                                                                       | 🔨Task      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_post_simon_sinek_poll_response_on_id_linkedin#[[44_work/ID Content Marketing/tasks/23_03_23_post_simon_sinek_poll_response_on_id_linkedin.md\|23_03_23_post_simon_sinek_poll_response_on_id_linkedin\|Post Simon Sinek Poll Response on ID LinkedIn]]                                                                                                                                                                                                                                   | 🔨Task      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_continue_to_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_20_continue_to_work_on_kn_nam_dashboard.md\|23_03_20_continue_to_work_on_kn_nam_dashboard\|Revise Navigation Page for KN Name Dashboard [time_start::1700]  [time_end::1830] 📅 2023-03-20 ✅ 2023-03-20]]                                                                                                                                                                          | 🌇Rit.      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_meeting_on_nam_tableau_dashboards#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_20_meeting_on_nam_tableau_dashboards.md\|23_03_20_meeting_on_nam_tableau_dashboards\|Name Tableau Dashboards [time_start::2000]  [time_end::2110] ⏰2023-03-20-19:50 📅 2023-03-20 ✅ 2023-03-20]]                                                                                                                                                                           | 🌇Rit.      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_morning_email_and_slack_review#[[23_03_20_morning_email_and_slack_review|23_03_20_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_miscellaneous_managerial_tasks#[[23_03_20_miscellaneous_managerial_tasks|23_03_20_miscellaneous_managerial_tasks\|Miscellaneous managerial tasks [time_start:: 16:30]  [time_end:: 17:00] 📅 2023-03-20 ✅ 2023-03-20]]                                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_20_evening_email_and_slack_review#[[23_03_20_evening_email_and_slack_review|23_03_20_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_meeting_with_timea_on_nam_tableau_dashboards#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_22_meeting_with_timea_on_nam_tableau_dashboards.md\|23_03_22_meeting_with_timea_on_nam_tableau_dashboards\|Meeting with Timea on Name Tableau Dashboards [time_start::0930]  [time_end::1000]  📅 2023-03-22 ✅ 2023-03-22]]                                                                                                                                     | 🌇Rit.      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_updates_from_meeting_with_timea_on_nam_tableau_dashboards#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_22_updates_from_meeting_with_timea_on_nam_tableau_dashboards.md\|23_03_22_updates_from_meeting_with_timea_on_nam_tableau_dashboards\|Updates from Meeting with Timea on Name Tableau Dashboards [time_start::1000]  [time_end::1045]  📅 2023-03-22 ✅ 2023-03-22]]                                                                                 | 🌇Rit.      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_22_work_on_kn_nam_dashboard.md\|23_03_22_work_on_kn_nam_dashboard\|Work on KN Name Dashboard [time_start::1045]  [time_end::1630] 📅 2023-03-22 ✅ 2023-03-22]]                                                                                                                                                                                                                                 | 🌇Rit.      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_work_on_kn_nam_dashboard#[[44_work/KN Name People Analytics Dashboard Building/tasks/23_03_23_work_on_kn_nam_dashboard.md\|23_03_23_work_on_kn_nam_dashboard\|Work on KN Name Dashboard [time_start::1415]  [time_end::1500] 📅 2023-03-22 ✅ 2023-03-22]]                                                                                                                                                                                                                                 | 🌇Rit.      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_nam_dashboard_sync_with_timea#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_22_nam_dashboard_sync_with_timea.md\|23_03_22_nam_dashboard_sync_with_timea\|Name Dashboard Sync [time_start::1630]  [time_end::1700] ⏰ 2023-03-22-16:20 📅 2023-03-22 ✅ 2023-03-22]]                                                                                                                                                                                          | 🌇Rit.      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_sync_with_nimrod_before_kn_meeting#[[23_03_22_sync_with_nimrod_before_kn_meeting|23_03_22_sync_with_nimrod_before_kn_meeting\|Sync with Nimrod Before KN Meeting [time_start:: 17:00]  [time_end:: 17:30] 📅 2023-03-22 ✅ 2023-03-22]]                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_kn_nam_dashboards_weekly#[[44_work/KN Name People Analytics Dashboard Building/meetings_weekly/23_03_22_kn_nam_dashboards_weekly.md\|23_03_22_kn_nam_dashboards_weekly\|KN Name Dashboards Weekly [time_start::1730]  [time_end::1830] ⏰2023-03-22-17:20 📅 2023-03-22 ✅ 2023-03-22]]                                                                                                                                                                                                     | 🌇Rit.      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_morning_email_and_slack_review#[[23_03_22_morning_email_and_slack_review|23_03_22_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_22_evening_email_and_slack_review#[[23_03_22_evening_email_and_slack_review|23_03_22_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_update_on_comeet-tableau_integration#[[23_03_23_update_on_comeet_tableau_integration|23_03_23_update_on_comeet-tableau_integration\|Update on Comeet-Tableau Integration]]                                                                                                                                                                                                                                                         | 🤝Meeting   | ✔️Done   | 23-03-23 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_fix_privacy_issue_on_kn_ca_ees_dashboard#[[44_work/KN CA Employee Engagement Survey 2022/tasks/23_03_23_fix_privacy_issue_on_kn_ca_ees_dashboard.md\|23_03_23_fix_privacy_issue_on_kn_ca_ees_dashboard\|Fix Privacy Issue on KN CA EES Dashboard]]                                                                                                                                                                                                                                      | 🔨Task      | ✔️Done   | 23-03-23 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_platform_scoring_mechanism_qa_offboarding#[[23_03_23_platform_scoring_mechanism_qa_offboarding|23_03_23_platform_scoring_mechanism_qa_offboarding\|Platform Scoring Mechanism QA Offboarding [time_start:: 12:30]  [time_end:: 13:40] 📅 2023-03-23 ✅ 2023-03-23]]                                                                                                                                                                       | 🌇Rit.      | ✔️Done   | 23-03-23 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_qa_privacy_issue_on_kn_ca_ees_dashboard#[[44_work/KN CA Employee Engagement Survey 2022/tasks/23_03_23_qa_privacy_issue_on_kn_ca_ees_dashboard.md\|23_03_23_qa_privacy_issue_on_kn_ca_ees_dashboard\|QA Privacy Issue on KN CA EES Dashboard]]                                                                                                                                                                                                                                          | 🔨Task      | ✔️Done   | 23-03-23 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_dayforce_integration_for_kn_nam_dashboards#[[44_work/KN Name People Analytics Dashboard Building/meetings_general/23_03_23_dayforce_integration_for_kn_nam_dashboards.md\|23_03_23_dayforce_integration_for_kn_nam_dashboards\|Dayforce Integration for KN Name Dashboards [time_start::1630]  [time_end::1645] ⏰2023-03-23-16:30 📅 2023-03-23 ✅ 2023-03-23]]                                                                                                                            | 🌇Rit.      | ✔️Done   | 23-03-23 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_morning_email_and_slack_review#[[23_03_23_morning_email_and_slack_review|23_03_23_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-23 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_evening_email_and_slack_review#[[23_03_23_evening_email_and_slack_review|23_03_23_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-23 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_26_check_mail_merge_for_sending_kn_ca_ees_manager_report#[[44_work/KN CA Employee Engagement Survey 2022/tasks/23_03_26_check_mail_merge_for_sending_kn_ca_ees_manager_report.md\|23_03_26_check_mail_merge_for_sending_kn_ca_ees_manager_report\|Check mail merge for sending KN CA EES Manager Report]]                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-03-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_26_offboarding_kn_ca_ees_and_pulse#[[23_03_26_offboarding_kn_ca_ees_and_pulse|23_03_26_offboarding_kn_ca_ees_and_pulse\|Offboarding_KN CA EES and Pulse [time_start::1400]  [time_end::1600] ⏰2023-03-26-13:00 📅 2023-03-26 ✅ 2023-03-26]]                                                                                                                                                                                               | 🌇Rit.      | ✔️Done   | 23-03-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_26_morning_email_and_slack_review#[[23_03_26_morning_email_and_slack_review|23_03_26_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_26_team_lunch#[[23_03_26_team_lunch|23_03_26_team_lunch\|Team lunch [time_start:: 13:00]  [time_end:: 14:00] 📅 2023-03-26 ✅ 2023-03-26]]                                                                                                                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_26_evening_email_and_slack_review#[[23_03_26_evening_email_and_slack_review|23_03_26_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_27_offboarding_kn_ca_exit_survey#[[23_03_27_offboarding_kn_ca_exit_survey|23_03_27_offboarding_kn_ca_exit_survey\|Offboarding_KN CA Exit Survey [time_start::0920]  [time_end::1100] ⏰2023-03-27-13:00 📅 2023-03-27 ✅ 2023-03-27]]                                                                                                                                                                                                       | 🌇Rit.      | ✔️Done   | 23-03-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_27_morning_email_and_slack_review#[[23_03_27_morning_email_and_slack_review|23_03_27_morning_email_and_slack_review\|Morning Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_23_offboarding_for_id_content_marketing#[[23_03_23_offboarding_for_id_content_marketing|23_03_23_offboarding_for_id_content_marketing\|Offboarding for ID Content Marketing]]                                                                                                                                                                                                                                                             | 🤝Meeting   | ✔️Done   | 23-03-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_27_evening_email_and_slack_review#[[23_03_27_evening_email_and_slack_review|23_03_27_evening_email_and_slack_review\|Evening Email and Slack Review]]                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-03-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_27_offboarding_hibob_tableau_integration#[[23_03_27_offboarding_hibob_tableau_integration|23_03_27_offboarding_hibob_tableau_integration\|Offboarding_HiBob Tableau Integration [time_start::09:30]  [time_end::10:45] ⏰2023-03-29-09:20 📅 2023-03-29 ✅ 2023-03-29]]                                                                                                                                                                     | 🌇Rit.      | ✔️Done   | 23-03-29 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_30_check_email_automation_options_supporting_attachments#[[44_work/KN CA Employee Engagement Survey 2022/tasks/23_03_30_check_email_automation_options_supporting_attachments.md\|23_03_30_check_email_automation_options_supporting_attachments\|Check email automation options supporting attachments]]                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-03-30 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_30_offboarding_kn_nam_dashboards#[[23_03_30_offboarding_kn_nam_dashboards|23_03_30_offboarding_kn_nam_dashboards\|Offboarding_KN Name Dashboards [time_start:: 13:15]  [time_end:: 14:15] ⏰2023-03-30-13:05 📅 2023-03-30 ✅ 2023-03-30]]                                                                                                                                                                                                   | 🌇Rit.      | ✔️Done   | 23-03-30 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_03_30_continue_to_check_email_automation_options_supporting_attachments#[[44_work/KN CA Employee Engagement Survey 2022/tasks/23_03_30_continue_to_check_email_automation_options_supporting_attachments.md\|23_03_30_continue_to_check_email_automation_options_supporting_attachments\|Continue to check email automation options supporting attachments]]                                                                                                                                  | 🔨Task      | ✔️Done   | 23-03-30 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_15 clean up the house and help sam get ready#Tasks\|Clean up the house and help Sam get ready]]                                                                                                                                                                                                                                                                                                                                                                                                        | 🔨Task      | ✔️Done   | 23-04-15 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_16 wash dishes#Tasks\|Wash Dishes]]                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | 🔨Task      | ✔️Done   | 23-04-16 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_17_fix_hibob_google_sheets_for_elementor#Tasks\|Fix HiBob Google Sheets for Elementor]]                                                                                                                                                                                                                                                                                                                                                                                                                | 🔨Task      | ✔️Done   | 23-04-17 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_17_check_how_to_add_questions_to_kn_ca_ees_2022_dashboard#Tasks\|Check How to Add Questions to KN CA EES 2022 Dashboard]]                                                                                                                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-04-17 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_17_revise_kn_ca_ees_2022_tableau_data_to_include_business_unit_and_functional_unit#Tasks\|Revise KN CA EES 2022 Tableau Data to Include Business Unit and Functional Unit]]                                                                                                                                                                                                                                                                                                                            | 🔨Task      | ✔️Done   | 23-04-17 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_17_offboarding_kn_ca_ees_and_pulse#[[23_04_17_offboarding_kn_ca_ees_and_pulse|23_04_17_offboarding_kn_ca_ees_and_pulse\|Offboarding KN CA EES and Pulse]]                                                                                                                                                                                                                                                                                 | 🤝Meeting   | ✔️Done   | 23-04-17 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_20 buy groceries and mold remover#Tasks\|Buy Groceries and Mold Remover]]                                                                                                                                                                                                                                                                                                                                                                                                                              | 🔨Task      | ✔️Done   | 23-04-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_24_update_hibob_tableau_integration#Tasks|Update HiBob-Tableau Integration]]                                                                                                                                                                                                                                                                                                                                                                                                                          | 🔨Task      | ✔️Done   | 23-04-24 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_24_update_kn_ca_ees_2022_tableau_dashboard_for_business_unit_and_functional_unit#Tasks\|Update KN CA EES 2022 Tableau Dashboard for Business Unit and Functional Unit]]                                                                                                                                                                                                                                                                                                                                | 🔨Task      | ✔️Done   | 23-04-24 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_04_25_check_hibob_tableau_integration_and_headcount_dashboard#Tasks|Check HiBob-Tableau Integration]]                                                                                                                                                                                                                                                                                                                                                                                                    | 🔨Task      | ✔️Done   | 23-04-25 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_05_17 laundry#Tasks\|Laundry]]                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | 🔨Task      | ✔️Done   | 23-05-17 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_05_17 wash dishes#Tasks\|Wash Dishes]]                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | 🔨Task      | ✔️Done   | 23-05-17 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_05_17 organize desk cables#Tasks\|Organize desk cables]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-05-17 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_02_load_washing_machine#Tasks\|Load Washing Machine]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-06-02 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_02_organize_boydem_storage#Tasks\|Organize Boydem Storage]]                                                                                                                                                                                                                                                                                                                                                                                                                                            | 🔨Task      | ✔️Done   | 23-06-02 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_02_hang_laundry#Tasks\|Hang Laundry]]                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-06-02 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_04_fold_laundry#Tasks\|Fold Laundry]]                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-06-04 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_12 wash the dishes and clean the sink#Action Item\|Wash the Dishes and Clean the Sink]]                                                                                                                                                                                                                                                                                                                                                                                                                | 🔨Task      | ✔️Done   | 23-06-12 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_21 bi-daily exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-06-21 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_21 bi-daily exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ✔️Done   | 23-06-21 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_21_continue_formatting_ml_pocket_reference_for_srs#Tasks\|Continue Formatting Machine Learning Pocket Reference for SRS]]                                                                                                                                                                                                                                                                                                                                                                | 🔨Task      | ✔️Done   | 23-06-21 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_22_empty_markdownload_folder_into_obsidian#Action Item\|Empty Markdownload Folder into Obsidian]]                                                                                                                                                                                                                                                                                                                                                                                                      | 🔨Task      | ✔️Done   | 23-06-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_22 bi-daily exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-06-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_22 bi-daily exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ✔️Done   | 23-06-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_22 birthday party for mikki syrkin#Meeting\|Birthday Party for Mikki Syrkin]]                                                                                                                                                                                                                                                                                                                                                                                                                          | ✉️Gathering | ✔️Done   | 23-06-22 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_25 bi-daily exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-06-25 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_25 bi-daily exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ❌Discard | ❌Discard | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_26 bi-daily exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-06-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_26 bi-daily exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ✔️Done   | 23-06-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_26 wedding of eran ziv and shoval cohen#Meeting\|Wedding of Eran Ziv and Shoval Cohen]]                                                                                                                                                                                                                                                                                                                                                                                                                | 🎊Event     | ✔️Done   | 23-06-26 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_27 figure out why action item template was not working and fix it#Action Item\|Figure Out Why Action Item Template Was Not Working and Fix It]]                                                                                                                                                                                                                                                                                                                                                        | 🔨Task      | ✔️Done   | 23-06-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_27 bi-daily exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-06-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_27_set_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Set Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                                          | 🔨Task      | ✔️Done   | 23-06-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_27 bi-daily exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ✔️Done   | 23-06-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_27_set_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Continue Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                             | 🔨Task      | ✔️Done   | 23-06-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_27_check_message_from_pheonix_recruiter_about_compensation_and_benefits_analyst_position#Action Item\|Check Message from Pheonix Recruiter About Compensation and Benefits Analyst Position]]                                                                                                                                                                                                                                                                                                          | 🔨Task      | ✔️Done   | 23-06-27 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_28_continue_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Continue Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                | 🔨Task      | ✔️Done   | 23-06-28 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_28_bi_daily_exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-06-28 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_28_continue_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Continue Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                | 🔨Task      | ✔️Done   | 23-06-28 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_28_bi_daily_exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ✔️Done   | 23-06-28 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_28_continue_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Continue Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                | 🔨Task      | ✔️Done   | 23-06-28 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_29 start filling out american tax forms#Action Item|Start Filling Out American Tax Forms]]                                                                                                                                                                                                                                                                                                                                                                                                            | 🔨Task      | ✔️Done   | 23-06-29 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_29_bi_daily_exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-06-29 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_06_29_bi_daily_exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ✔️Done   | 23-06-29 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_02_work_on_apartment_water_issue_and_backyard_screen#Action Item\|Work on Apartment Water Issue and Backyard Screen]]                                                                                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-07-02 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_03_bi_daily_exercise#Habit\|Early Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 🤖Habit     | ✔️Done   | 23-07-03 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_03_bi_daily_exercise#Habit\|Late Afternoon Movement]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🤖Habit     | ✔️Done   | 23-07-03 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_05_review_american_tax_documents#Action Item\|Review American Tax Documents]]                                                                                                                                                                                                                                                                                                                                                                                                                          | 🔨Task      | ✔️Done   | 23-07-05 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_05_finish_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Finish Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-07-05 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_05_finish_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Finish Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-07-05 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_05_finish_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Finish Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-07-05 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_06_finish_reviewing_american_tax_documents#Action Item\|Finish Reviewing American Tax Documents]]                                                                                                                                                                                                                                                                                                                                                                                                      | 🔨Task      | ✔️Done   | 23-07-06 | <ul><li>**Outcome**: I reviewed the documents and they seem in order, but I did not know where to sign, so I sent [[zwebner_asher\|Asher Zwebner]] an email asking him about it.</li><li>**Feeling**: I feel good that I am taking charge of my financial stability.</li></ul>                                   |
| [[23_07_05_finish_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Finish Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-07-06 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_05_finish_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Finish Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-07-06 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_05_finish_setting_up_machine_learning_education_project_and_parent_task_directories#Action Item\|Finish Setting Up Machine Learning Education Project and Parent Task Directories]]                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-07-06 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_19_first_job_search_workshop#Meeting\|First Job Search Workshop]]                                                                                                                                                                                                                                                                                                                                                                                                                                      | 🤝Meeting   | ✔️Done   | 23-07-19 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_20_update_people_analyst_resume#Action Item\|Update People Analyst Resume]]                                                                                                                                                                                                                                                                                                                                                                                                                            | 🔨Task      | ✔️Done   | 23-07-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_07_25_second_shalem_job_search_workshop#Meeting\|Second Shalem Job Search Workshop]]                                                                                                                                                                                                                                                                                                                                                                                                                      | 🤝Meeting   | ✔️Done   | 23-07-25 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_08_10_buy_prescriptions#Action Item\|Buy Prescriptions]]                                                                                                                                                                                                                                                                                                                                                                                                                                                  | 🔨Task      | ✔️Done   | 23-08-10 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_08_10_buy_groceries#Action Item\|Buy Groceries]]                                                                                                                                                                                                                                                                                                                                                                                                                                                          | 🔨Task      | ✔️Done   | 23-08-10 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_08_13_fold_laundry#Action Item\|Fold Laundry]]                                                                                                                                                                                                                                                                                                                                                                                                                                                            | 🔨Task      | ✔️Done   | 23-08-13 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_08_20_transfer_sams_bank_account_to_leumi#Action Item\|Transfer Sam's Bank Account to Leumi]]                                                                                                                                                                                                                                                                                                                                                                                                             | 🔨Task      | ✔️Done   | 23-08-20 | <ul><li>**Outcome**: \-</li><li>**Feeling**: \-</li></ul>                                                                                                                                                                                                                                                         |
| [[23_08_21_review_links_from_ilan#Action Item\|Review Links from Ilan]]                                                                                                                                                                                                                                                                                                                                                                                                                                        | 🔨Task      | ✔️Done   | 23-08-21 | <ul><li>**Outcome**: I reached out to everyone, except Tahg, and hope to hear back.</li><li>**Feeling**: I feel good that I actually reached out, but I am so damn tired.</li></ul>                                                                                                                               |
| [[23_08_24_reach_out_to_linkedin_contact_from_pola#Action Item\|Reach Out to Linkedin Contact from Pola]]                                                                                                                                                                                                                                                                                                                                                                                                      | 🔨Task      | ✔️Done   | 23-08-24 | <ul><li>**Outcome**: I reached out to [[haitin_zacharya\|Zacharya Haitin]] asking about the HR tech world in Israel.</li><li>**Feeling**: I feel alright about reaching out, but I am less hopeful because LInkedIn requires sending connection requests to send a message if you do not have premium.</li></ul> |

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type,
    choice((T.status != "-"),
        (choice((T.status = "x"), "✔️Done", "🔜To do")),
        "❌Discard")
    AS Status,
    choice((T.status != "-"),
        (choice((T.status = "x"), dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))),
        "❌Discard")
    AS Date,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND (contains(file.frontmatter.file_class, "action")
    OR contains(file.frontmatter.file_class, "meeting")
    OR contains(file.frontmatter.file_class, "habit_ritual"))
    AND filter(file.frontmatter.project, (project) =>
      contains(this.file.frontmatter.project, project))
    AND filter(file.frontmatter.parent_task, (parent) =>
      contains(this.file.frontmatter.parent_task, parent))
    AND !contains(file.inlinks, this.file.link)
    AND regextest("(#task)", T.text)
SORT
    T.due,
    T.time_start ASC
```

### General Child Tasks

| Task | Type | Status | Date | Project |
| ---- | ---- | ------ | ---- | ------- |

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type,
    choice((T.status != "-"),
        (choice((T.status = "x"), "✔️Done", "🔜To do")),
        "❌Discard")
    AS Status,
    choice((T.status != "-"),
        (choice((T.status = "x"), dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))),
        "❌Discard")
    AS Date,
    choice(length(file.frontmatter.project) < 2, file.frontmatter.project[0], flat(file.frontmatter.project)) AS Project
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND (contains(file.frontmatter.file_class, "action")
    OR contains(file.frontmatter.file_class, "meeting")
    OR contains(file.frontmatter.file_class, "habit_ritual"))
    AND contains(file.outlinks, this.file.link)
    AND !contains(file.inlinks, this.file.link)
    AND !contains(file.path, this.file.folder)
    AND regextest("(#task)", T.text)
SORT
    T.due,
    T.time_start ASC
```

---

## Related Knowledge

> [!pkm] PKM Notes
>
> - `BUTTON[button-pkm-question-table]`|`BUTTON[button-pkm-evidence-table]`|`BUTTON[button-pkm-steps-table]`|`BUTTON[button-pkm-conclusion-table]`
> - `BUTTON[button-pkm-idea-table]`|`BUTTON[button-pkm-summary-table]`|`BUTTON[button-pkm-quote-table]`
> - `BUTTON[button-pkm-concept-table]`|`BUTTON[button-pkm-definition-table]`

<!-- Adjust replace lines -->

```button
name Related PKM Files
type append template
action 100_70_dvmd_related_pkm_sect
replace [1, 2]
color purple
```

### Outgoing PKM Links

<!-- Link related pkm files here -->

### Knowledge Tree

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
      "category": "🏘️Category",
      "branch": "🪑Branch",
      "field": "🚪Field",
      "subject": "🗝️Subject",
      "topic": "🧱Topic"
    }[x])(file.frontmatter.type), "🔩Subtopic")
	AS Type,
	file.frontmatter.about AS Description,
	default(((x) => {
      "branch": file.frontmatter.category,
      "field": flat(list(file.frontmatter.category, file.frontmatter.branch)),
      "subject": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field)),
      "topic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject)),
      "subtopic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject, file.frontmatter.topic))
    }[x])(file.frontmatter.type), "")
	AS Context
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.file_class, "tree")
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "category": 1,
      "branch": 2,
      "field": 3,
      "subject": 4,
      "topic": 5
    }[x])(file.frontmatter.type), 6),
	file.frontmatter.title ASC
```

### Permanent

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  choice(file.frontmatter.subtype = "question", "❔Question",
	choice(file.frontmatter.subtype = "evidence", "⚖️Evidence",
	choice(file.frontmatter.subtype = "conclusion", "🏁Conclusion",
	choice(file.frontmatter.subtype = "problem", "🪨Problem",
	choice(file.frontmatter.subtype = "step", "🪜Step",
	choice(file.frontmatter.subtype = "answer", "🎱Answer",
	choice(file.frontmatter.subtype = "quote", "⏺️Quote",
	choice(file.frontmatter.subtype = "idea", "💭Idea",
	choice(file.frontmatter.subtype = "concept", "🎞️Concept", "🪟Definition")))))))))
	AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
      "review": "🔜Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "permanent")
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Literature

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  choice(file.frontmatter.subtype = "question", "❔Question",
	choice(file.frontmatter.subtype = "evidence", "⚖️Evidence",
	choice(file.frontmatter.subtype = "conclusion", "🏁Conclusion",
	choice(file.frontmatter.subtype = "problem", "🪨Problem",
	choice(file.frontmatter.subtype = "step", "🪜Step",
	choice(file.frontmatter.subtype = "answer", "🎱Answer",
	choice(file.frontmatter.subtype = "quote", "⏺️Quote",
	choice(file.frontmatter.subtype = "idea", "💭Idea",
	choice(file.frontmatter.subtype = "concept", "🎞️Concept", "🪟Definition")))))))))
	AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
      "review": "🔜Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "literature")
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Fleeting

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  choice(file.frontmatter.subtype = "question", "❔Question",
	choice(file.frontmatter.subtype = "evidence", "⚖️Evidence",
	choice(file.frontmatter.subtype = "conclusion", "🏁Conclusion",
	choice(file.frontmatter.subtype = "problem", "🪨Problem",
	choice(file.frontmatter.subtype = "step", "🪜Step",
	choice(file.frontmatter.subtype = "answer", "🎱Answer",
	choice(file.frontmatter.subtype = "quote", "⏺️Quote",
	choice(file.frontmatter.subtype = "idea", "💭Idea",
	choice(file.frontmatter.subtype = "concept", "🎞️Concept", "🪟Definition")))))))))
	AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
      "review": "🔜Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "fleeting")
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Info

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	  choice(file.frontmatter.subtype = "question", "❔Question",
	choice(file.frontmatter.subtype = "evidence", "⚖️Evidence",
	choice(file.frontmatter.subtype = "conclusion", "🏁Conclusion",
	choice(file.frontmatter.subtype = "problem", "🪨Problem",
	choice(file.frontmatter.subtype = "step", "🪜Step",
	choice(file.frontmatter.subtype = "answer", "🎱Answer",
	choice(file.frontmatter.subtype = "quote", "⏺️Quote",
	choice(file.frontmatter.subtype = "idea", "💭Idea",
	choice(file.frontmatter.subtype = "concept", "🎞️Concept", "🪟Definition")))))))))
	AS Subtype,
	  default(((x) => {
      "schedule": "🤷Unknown",
      "review": "🔜Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "done": "🌳Done"
    }[x])(file.frontmatter.status), "🗄️Resource")
	AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\\n"), (x) => regextest("\\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	AND contains(file.frontmatter.file_class, "pkm")
	AND contains(file.frontmatter.type, "info")
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

---

## Related Library Content

<!-- Adjust replace lines -->

```meta-bind-button
label: 🏫Related Library Content
tooltip: "Replace the library section with MD table of linked files and a filtered DataView table"
class: mb_button_green
hidden: false
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/100_60_dvmd_related_lib_sect.md
    templater: true
```

### Outgoing Library Links

<!-- Link related library files here -->

### Library Content

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	Author AS Author,
	choice(contains(file.frontmatter.type, "book"), file.frontmatter.year_published, file.frontmatter.date_published) AS "Date Published",
	default(((x) => {
      "book": "📚Book",
      "book_chapter": "📑Book Chapter",
      "course": "🧑‍🏫Course",
      "course_lecture": "🧑‍🎓Course Lecture",
      "journal": "📜️Journal",
      "report": "📈Report",
      "news": "🗞️News",
      "magazine": "📰️Magazine",
      "webpage": "🌐Webpage",
      "blog": "💻Blog",
      "video": "🎥️Video",
      "youtube": "▶YouTube",
      "documentary": "🖼️Documentary",
      "audio": "🔉Audio",
      "podcast": "🎧️Podcast"
    }[x])(file.frontmatter.type), "📃Documentation")
	AS Type,
	default(((x) => {
      "undetermined": "❓Undetermined",
      "to_do": "🔜To do",
      "in_progress": "👟In progress",
      "done": "✔️Done",
      "resource": "🗃️Resource",
      "schedule": "📅Schedule"
    }[x])(file.frontmatter.status), "🤌On hold")
    AS Status,
	file.etags AS Tags
FROM
	"60_library"
	OR "90_inbox"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "lib")
	AND contains(file.frontmatter.file_class, "")
SORT
	file.frontmatter.type,
	file.frontmatter.title ASC
```

---
