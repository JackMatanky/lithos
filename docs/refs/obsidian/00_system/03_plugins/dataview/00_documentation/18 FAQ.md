---
title:
  - 18 FAQ
aliases:
  - 18 FAQ
  - dataview_documentation_18_faq
application: dataview
url:
file_class: lib_documentation
date_created: 2023-03-09T18:20
date_modified: 2023-10-25T16:22
tags:
---
# [Frequently Asked Questions](https://blacksmithgu.github.io/obsidian-dataview/resources/faq/)

A collection of frequently asked questions for Dataview queries and the expression language.

## How Do I Use Fields with the Same name as Keywords (like "from", "where")?

Dataview provides a special "fake" field called `row` which can be indexed into to obtain fields which conflict with Dataview keywords:

```javascript
row.from /* Same as "from" */
row.where /* Same as "where" */
```

## How Do I Access Fields with Spaces in the Name?

There are two ways:

1. Use the normalized Dataview name for such a field - just convert the name to lowercase and replace whitespace with dashes ("-"). Something like `Field With Space In It` becomes `field-with-space-in-it`.
2. Use the implicit `row` field:

```javascript
row["Field With Space In It"]
```

## Do You Have a List of Resources to Learn From?

Yes! Please see the [Resources](../resources/resources-and-support.md) page.

## Can I save the Result of a Query for Reusability?

You can write reusable Javascript Queries with the [dv.view](../../api/code-reference/#dvviewpath-input) function. In DQL, beside the possibility of writing your Query inside a Template and using this template (either with the [Core Plugin Templates](https://help.obsidian.md/Plugins/Templates) or the popular Community Plugin [Templater](https://obsidian.md/plugins?id=templater-obsidian)), you can **save calculations in metadata fields via [Inline DQL](../../queries/dql-js-inline#inline-dql)**, for example:

```markdown
start:: 07h00m
end:: 18h00m
pause:: 01h30m
duration:: `= this.end - this.start - this.pause`
```

You can list the value (9h 30m in our example) then i.e. in a TABLE without needing to repeat the calculation:

```sql
```dataview
TABLE start, end, duration
WHERE duration
```

Gives you

| File (1) | start   | end      | duration            |
| -------- | ------- | -------- | ------------------- |
| Example  | 7 hours | 18 hours | 9 hours, 30 minutes |

**But storing a Inline DQL in a field comes with a limitation**: While the value that gets displayed in the result is the calculated one, **the saved value inside your metadata field is still your Inline DQL calculation**. The value is literally `= this.end - this.start - this.pause`. This means you cannot filter for the Inlines' result like:

```sql
```dataview
TABLE start, end, duration
WHERE duration > dur("10h")
```

This will give you back the example page, even though the result doesn't fulfill the `WHERE` clause, because the value you are comparing against is `= this.end - this.start - this.pause` and is not a duration.

## How Can I Hide the Result Count on TABLE Queries?

With Dataview 0.5.52, you can hide the result count on TABLE and TASK Queries via a setting. Go to Dataview's settings -> Display result count.
