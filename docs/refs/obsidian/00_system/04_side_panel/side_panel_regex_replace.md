---
title: side_panel_regex_replace
aliases:
  - Side Panel RegEx Replace
  - Common RegEx Replacements
  - side_panel_regex_replace
cssclasses:
  - inline_title_hide
  - side_panel_narrow
  - read_hide_properties
  - read_narrow_margin
  - font_size_small
  - list_narrow
date_created: 2023-09-20T11:09
date_modified: 2024-08-28T11:54
---
- Table of Contents Callout Links:
	- `(Context|Reflect|Goals|Events|Directory|Content|Knowledge|Notes|Information|Compass|cards|Code|Snippets)\|(Re|In|Go|Ta|Di|Li|PKM|Tr|No|Id|Fl|Co|Sn)`⇨`$1\|$2`
- Task Date:
	- `((title:\s|\[\[|-\s|-\s"|⏰\s|📅\s)(\d){1,4}(-|_))(\d\d)(-|_)\d\d`⇨`$1$5$6##`
- Cancel Tasks:
	- `(\n-\s\[)\s(\]\s#task.*\n)`⇨`<!--$1<$2-->`
- Cancel Specific Tasks:
	- `(\n)(\-\s\[)\s(\]\s#task.*?(Late|Mov|Med|Aff|Ach).*\d\])(\s⏰.*?)?(\s➕.*)(\n)`⇨`$1<!--$1$2<$3$6$7-->$7`
- Discard Tasks  
	- `- [ ] #task ⇒ - [-] #task`  
	- `(-\s\[)\s(\]\s#task)` ⇒ `$1-$2`
- Remove Empty Inline Data
	- `-\s.*((:){2}\s|(:){2})$`
- Remove Empty Inline Data
	- `\\(\[|\])`⇨`\$\$`
	- `\\(\(|\))`⇨`\$`