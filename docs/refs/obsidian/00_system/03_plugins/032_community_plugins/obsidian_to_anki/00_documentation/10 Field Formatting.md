---
title:
  - 10 Field Formatting
aliases:
  - 10 Field Formatting
  - Field Formatting
  - field_formatting
  - obsidian_to_anki_field_formatting
application: obsidian_to_anki
url: https://github.com/Pseudonium/Obsidian_to_Anki/wiki/Field-formatting
file_class: lib_documentation
date_created: 2023-05-30T19:13
date_modified: 2023-10-25T16:22
tags:
---
# Field Formatting

Apart from the first field, each field must have a prefix to indicate to the program which field to add text into. The prefix is just a colon (":") added onto the field name. For example:

```
START
Basic
Front: This is a test.
Back: Test successful!
END
```

This produces the following card in Anki:

![[Pasted image 20230530193551.webp]]

You can omit the prefix for the first field for convenience:

```
START
Basic
This is a test.
Back: Test successful!
END
```

And you can continue on a new line for the same field:

```
START
Basic
This is a test.
And the test is continuing.
Back: Test successful!
END
```

![[Pasted image 20230530193603.webp]]

You must start each new field on a new line. But otherwise you are free to omit as many or as few fields as you wish, or change up the order of fields!
