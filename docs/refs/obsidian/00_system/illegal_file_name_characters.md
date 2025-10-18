---
title: illegal_file_name_characters
aliases:
  - Illegal File Name Characters
  - illegal_file_name_characters
tags: regex
date_created: 2024-06-02T09:49
date_modified: 2024-09-24T20:18
---
# Illegal File Name Characters

The following are characters that are not allowed to be used in Windows file names  
\#: *? " < > | /  
Regex: `\[#:\*?"<>\|/\\]`  
Underscore regex: \#:\\\*<>\|\\/

1. # : Change to underscore (\_)
2. :: Change to underscore (\_)
3. \*: Change to underscore (\_)
4. <: Change to underscore (\_)
5. > : Change to underscore (\_)
6. |: Change to underscore (\_)
7. \\: Change to underscore (\_)
8. /: Change to underscore (\_)
9. ?: Remove
10. ": Change to underscore single quote (')

#regex

`* " \ / < > : | ?`
