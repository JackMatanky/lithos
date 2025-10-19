---
title: 53_10_week_pdev_review_preview
aliases:
  - Weekly PDEV Journals Review and Preview Checklist
  - Weekly PDEV Journals Review and Preview
  - weekly pdev journals review and preview checklist
  - weekly pdev journals review and preview
  - week pdev review preview
plugin: templater
language:
  - javascript
module:
  - file
cssclasses: null
type: snippet
file_class: pkm_code
date_created: 2023-06-13T13:49
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/file/include
---
# Weekly Personal Development Journals Review and Preview Checklist

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return callouts of checklists for the Weekly Personal Development Journals Review and Preview.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const week_journal_review_preview = "43_10_action_week_journal_review_preview";

//---------------------------------------------------------
// WEEKLY PDEV REVIEW AND PREVIEW CHECKLIST
//---------------------------------------------------------
// Retrieve the Weekly Journal Review and
// Preview Checklist template and content
temp_file_path = `${sys_temp_include_dir}${week_journal_review_preview}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const journal_review_preview_checklist = include_arr;
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// WEEKLY PDEV REVIEW AND PREVIEW CHECKLIST
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${week_journal_review_preview}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const journal_review_preview_checklist = include_arr;
```

#### Referenced Template

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const head = String.fromCodePoint(0x23);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const four_space = space.repeat(4);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const hr_line = String.fromCodePoint(0x2d).repeat(3);
const cmnt_ob_start = `${String.fromCodePoint(37).repeat(2)}${space}`;
const cmnt_ob_end = `${space}${String.fromCodePoint(37).repeat(2)}`;
const colon = String.fromCodePoint(0x3a);
const two_colon = colon.repeat(2);
const ul = `${String.fromCodePoint(0x2d)}${space}`;
const tbl_start =`|${space}`;
const tbl_end =`${space}|`;
const tbl_pipe = `${space}|${space}`;
const call_start = `>${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${four_space}${ul}`;
const call_check = `${call_ul}[${space}]${space}`;
const call_check_indent = `${call_ul_indent}[${space}]${space}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;

const head_one = `${hash}${space}`;
const head_two = `${hash.repeat(2)}${space}`;
const head_three = `${hash.repeat(3)}${space}`;
const head_four = `${hash.repeat(4)}${space}`;

//---------------------------------------------------------
// WEEKLY PDEV JOURNAL REVIEW AND PREVIEW CHECKLIST
//---------------------------------------------------------
const call_title = `${call_start}[!task_plan]${space}Weekly Personal Development Journal Review and Preview Plan${new_line}${call_start}${new_line}`;

const recount = `${call_start}1.${space}**Daily Recollection**${new_line}${call_check_indent}Skim through daily reflections.${new_line}${call_check_indent}Write about the past week.${new_line}${call_start}${new_line}`;

const experience = `${call_start}2.${space}**Best Experiences**${new_line}${call_check_indent}Review last week's best experiences.${new_line}${call_check_indent}Write about trends in the week's best experiences.${new_line}${call_check_indent}Write insights about the trends.${new_line}${call_start}${new_line}`;

const achievement = `${call_start}3.${space}**Achievements**${new_line}${call_check_indent}Review the last week's achievements.${new_line}${call_check_indent}Congratulate myself on my achievements.${new_line}${call_start}${new_line}`;

const gratitude = `${call_start}4.${space}**Gratitude**${new_line}${call_check_indent}Review and recall moments of gratitude.${new_line}${call_check_indent}Review and recall moments of self-gratitude.${new_line}${call_start}${new_line}`;

const blindspot = `${call_start}5.${space}**Blindspots**${new_line}${call_check_indent}Review last week's blindspots.${new_line}${call_check_indent}Write about trends in the blindspots.${new_line}${call_check_indent}Write actionable insights on the trends on how to avoid repeating blindspots.${new_line}${call_start}${new_line}`;

const detachment = `${call_start}6.${space}**Detachments**${new_line}${call_check_indent}Review last week's detachments.${new_line}${call_check_indent}Write about trends in the detachments.${new_line}${call_check_indent}Determine focus areas based on detachment trends and prep.${new_line}${call_start}${new_line}`;

const limiting_belief = `${call_start}7.${space}**Limiting Beliefs**${new_line}${call_check_indent}Review last week's limiting beliefs.${new_line}${call_check_indent}Review and clarify the limiting belief.${new_line}${call_check_indent}Review and clarify the perspective created by the limiting belief.${new_line}${call_check_indent}Review and clarify the limiting belief's effect and cost.${new_line}${call_check_indent}Review and clarify the limiting belief's background.${new_line}${call_check_indent}Review and clarify the limiting belief's reframing.${new_line}${call_check_indent}Review and clarify the limiting truth's affirmation.${new_line}${call_start}${new_line}`;

const lesson = `${call_start}8.${space}**Lessons Learned**${new_line}${call_check_indent}Review last week's lessons learned.${new_line}${call_check_indent}Distill last week's lessons learned and include lessons from other journals.`;

const pdev_review_preview = `${call_title}${recount}${experience}${achievement}${gratitude}${blindspot}${detachment}${limiting_belief}${lesson}`;

tR += pdev_review_preview;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[51_00_parent_task|General Parent Task Template]]
2. [[52_00_task_event|General Tasks and Events Template]]
3. [[53_00_action_item|Action Item Template]]
4. [[54_00_meeting|Meeting Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[53_10_action_week_journal_review_preview]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[53_10_week_pdev_review_preview|Weekly PDEV Journals Review and Preview Checklist]]
2. [[53_10_week_task_event_review|Weekly Tasks and Events Review Checklist]]
3. [[53_10_week_task_event_preview|Weekly Tasks and Events Preview Checklist]]
4. [[53_10_week_habit_rit_review_preview|Weekly Habits and Rituals Review and Preview Checklist]]
5. [[53_10_week_lib_review_preview|Weekly Library Review and Preview Checklist]]
6. [[53_10_week_pkm_review_preview|Weekly PKM Review and Preview Checklist]]
7. [[53_00_action_item_preview|Before Action Preview]]
8. [[task_event_plan|Action Plan]]
9. [[53_00_action_item_review|After Action Review]]

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	file.frontmatter.definition AS Definition
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
