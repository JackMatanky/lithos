---
aliases:
  - callouts_obsidian
title: Callouts
subtitle: 
author: 
date_published: 
publisher: Obsidian Help
url: https://publish.obsidian.md/
type: documentation
file_class: lib_documentation
cssclasses:
date_created: 2023-03-21T11:59
date_modified: 2023-09-05T19:18
tags: obsidian, obsidian/markdown, markdown, markdown/callout
---
# Callouts

Use callouts to include additional content without breaking the flow of your notes.

To create a callout, add `[!info]` to the first line of a blockquote, where `info` is the *type identifier*. The type identifier determines how the callout looks and feels. To see all available types, refer to [Supported types](https://help.obsidian.md/Editing+and+formatting/Callouts#Supported%20types).

```
> [!info]
> Here's a callout block.
> It supports **Markdown**, [[Internal link|Wikilinks]], and [[Embed files|embeds]]!
> ![[og-image.png]]
```

> [!info]  
> Here's a callout block.  
> It supports **Markdown**, [[Internal link|Wikilinks]], and [[Embed Files|embeds]]!  
> ![[og-image.png]]

> [!Note]  
> If you're also using the Admonitions plugin, you should update it to at least version 8.0.0 to avoid problems with the new callout feature.

## Change the Title

By default, the title of the callout is its type identifier in title case. You can change it by adding text after the type identifier:

```
> [!tip] Callouts can have custom titles
> Like this one.
```

> [!tip] Callouts can have custom titles  
> Like this one.

You can even omit the body to create title-only callouts:

```
> [!tip] Title-only callout
```

> [!tip] Title-only callout

## Foldable Callouts

You can make a callout foldable by adding a plus (+) or a minus (-) directly after the type identifier.

A plus sign expands the callout by default, and a minus sign collapses it instead.

```
> [!faq]- Are callouts foldable?
> Yes! In a foldable callout, the contents are hidden when the callout is collapsed.
```

> [!faq]- Are callouts foldable?  
> Yes! In a foldable callout, the contents are hidden when the callout is collapsed.

## Nested Callouts

You can nest callouts in multiple levels.

```
> [!question] Can callouts be nested?
> > [!todo] Yes!, they can.
> > > [!example]  You can even use multiple layers of nesting.
```

> [!question] Can callouts be nested?
> 
> > [!todo] Yes!, they can.
> > 
> > > [!example] You can even use multiple layers of nesting.

## Customize Callouts

To define a custom callout, create the following CSS block:

```css
.callout[data-callout="custom-question-type"] {
    --callout-color: 0, 0, 0;
    --callout-icon: lucide-alert-circle;
}
```

The value of the `data-callout` attribute is the type identifier you want to use, for example `[!custom-question-type]`.

- `--callout-color` defines the background color using numbers (0–255) for red, green, and blue.
- `--callout-icon` can be an icon ID from [lucide.dev](https://lucide.dev/), or an SVG element.

> [!tip] SVG icons  
> Instead of using a Lucide icon, you can also use a SVG element as the callout icon.
> 
> ```
> --callout-icon: '<svg> …custom svg…</svg> ';
> ```

## Supported Types

You can use several callout types and aliases. Each type comes with a different background color and icon.

To use these default styles, replace `info` in the examples with any of these types, such as `[!tip]` or `[!warning]`.

Unless you [Customize callouts](https://help.obsidian.md/Editing+and+formatting/Callouts#Customize%20callouts), any unsupported type defaults to the `note` type. The type identifier is case-insensitive.

> [!note]
> 
> ```
> [!note]
> Lorem ipsum dolor sit amet
> ```

---

> [!abstract]
> 
> ```
> [!abstract]
> Lorem ipsum dolor sit amet
> ```

Aliases: `summary`, `tldr`

---

> [!info]-
> 
> ```
> [!info]
> Lorem ipsum dolor sit amet
> ```

---

> [!todo]-
> 
> ```
> [!todo]
> Lorem ipsum dolor sit amet
> ```

---

> [!tip]-
> 
> ```
> [!tip]
> Lorem ipsum dolor sit amet
> ```

Aliases: `hint`, `important`

---

> [!success]-
> 
> ```
> [!success]
> Lorem ipsum dolor sit amet
> ```

Aliases: `check`, `done`

---

> [!question]-
> 
> ```
> [!question]
> Lorem ipsum dolor sit amet
> ```

Aliases: `help`, `faq`

---

> [!warning]-
> 
> ```
> [!warning]
> Lorem ipsum dolor sit amet
> ```

Aliases: `caution`, `attention`

---

> [!failure]-
> 
> ```
> [!failure]
> Lorem ipsum dolor sit amet
> ```

Aliases: `fail`, `missing`

---

> [!danger]-
> 
> ```
> [!danger]
> Lorem ipsum dolor sit amet
> ```

Alias: `error`

---

> [!bug]-
> 
> ```
> [!bug]
> Lorem ipsum dolor sit amet
> ```

---

> [!example]-
> 
> ```
> [!example]
> Lorem ipsum dolor sit amet
> ```

---

> [!quote]-
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `cite`

---

## Custom Callouts

### Insight

> [!Insight]-
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `insight`

---

> [!vision_purpose]- Vision and Purpose
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `vision_purpose`, `vision`, `purpose`

---

> [!principle_value]- Principles and Values
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `principle_value`, `principle`, `value`

---

> [!mindset]- Mindset
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `mindset`

---

> [!limiting_belief]- Limiting Belief
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `limiting_belief`

---

#### Journals

> [!reflection]- Reflection Journal
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `reflection`

---

> [!gratitude]- Gratitude Journal
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `gratitude`

---

> [!detachment]- Detachment Journal
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `detachment`

---

> [!prompt]- Prompt Journal
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `prompt`

---

### Calendar

> [!year]- Year Context
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `year`

---

> [!quarter]- Quarter Context
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `quarter`

---

> [!month]- Month Context
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `month`

---

> [!week]- Week Context
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `week`

---

> [!day]- Day Context
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `day`

---

### Task Management

> [!project]- Project Details
> 
> ```
> [!project]
> Lorem ipsum dolor sit amet
> ```

Alias: `project`

---

> [!parent_task]- Parent Task Details
> 
> ```
> [!parent_task]
> Lorem ipsum dolor sit amet
> ```

Alias: `parent_task`

---

> [!action_item]- Action Item Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `action_item`

---

> [!meeting]- Meeting Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `meeting`

---

> [!habit_ritual]- Habit Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `habit_ritual`

---

> [!morning_ritual]- Morning Ritual Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `morning_ritual`

---

> [!workday_startup_ritual]- Workday Startup Ritual Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `workday_startup_ritual`

---

> [!workday_shutdown_ritual]- Workday Shutdown Ritual Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `workday_shutdown_ritual`

---

> [!evening_ritual]- Evening Ritual Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `evening_ritual`

---

> [!task_preview]- Before Action Preview
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `task_preview`

---

> [!task_plan]- Action Plan
> 
> What is the solution's detailed plan?

Alias: `task_plan`

---

> [!task_review]- After Action Review
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `task_review`

---

### Directory

> [!contact]- Contact Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `contact`

---

> [!organization]- Organization Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `organization`

---

### Library

> [!video]- Video Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `video`

---

### Knowledge Tree

> [!tree]- Knowledge Tree  
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `tree`

---

#### Programming

> [!code]- Code Details  
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `code`

---

> [!function]- Function Details  
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `function`

---

> [!snippet]- Snippet Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `snippet`

---

> [!param]- Parameter Details
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `param`

---

### Knowledge Lab

> [!note_relation]- Note Relations  
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `note_relation`

---

> [!Quote]
> 
> quote::

Alias: `quote`

---

> [!Thought]
> 
> idea::

Alias: `thought`

---

#### Question-Evidence-Conclusion

> [!Question]
> 
> question::

Alias: `question`

---

> [!Evidence]
> 
> evidence::

Alias: `evidence`

---

> [!Conclusion]
> 
> conclusion::

Alias: `conclusion`

---

#### Problem-Solution-Answer

> [!Problem]
> 
> problem::

Alias: `problem`

---

> [!Steps]
> 
> step::

Alias: `steps`

---

> [!Answer]
> 
> answer::

Alias: `answer`

---

> [!Definition]
> 
> term::
> 
> definition::

Alias: `definition`

---

> [!Concept]
> 
> term::
> 
> description::

Alias: `concept`

---

#### Idea Compass

> [!north] North: Where is the origin?
> 
> origin::

Alias: `north`

---

> [!west] West: What is similar?
> 
> similar::

Alias: `west`

---

> [!east] East: What is opposite?
> 
> opposite::

Alias: `east`

---

> [!south] South: Where does this lead?
> 
> destination::

Alias: `south`

---

### General

> [!toc]- Table of Contents  
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `toc`, `contents`

---

> [!dir]- Directory and Subfiles  
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `directory`, `dir`, `subfile`

---

> [!exercise]- Exercise  
> 
> ```
> [!quote]
> Lorem ipsum dolor sit amet
> ```

Alias: `exercise`

---
