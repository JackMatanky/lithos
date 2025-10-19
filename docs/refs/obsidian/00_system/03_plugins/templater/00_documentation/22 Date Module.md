---
title:
  - 22 Date Module
aliases:
  - 22 Date Module
  - templater_documentation_22_date_module
application: templater
url:
file_class: lib_documentation
date_created: 2023-03-10T15:44
date_modified: 2023-10-25T16:22
tags:
---
# Date Module

This module contains every internal function related to dates.

## Documentation

Function documentation is using a specific syntax. More information [here](https://silentvoid13.github.io/Templater/syntax.html#function-documentation-syntax)

### `tp.date.now(format: String = "YYYY-MM-DD", Offset?: number⎮string, Reference?: String, reference_format?: string)`

Retrieves the date.

#### Arguments

- `format`: Format for the date, refer to [format reference](https://momentjs.com/docs/#/displaying/format/)

- `offset`: Offset for the day, e.g. set this to `-7` to get last week's date. You can also specify the offset as a string using the ISO 8601 format

- `reference`: The date referential, e.g. set this to the note's title

- `reference_format`: The date reference format.

### `tp.date.tomorrow(format: String = "YYYY-MM-DD")`

Retrieves tomorrow's date.

#### Arguments

- `format`: Format for the date, refer to [format reference](https://momentjs.com/docs/#/displaying/format/)

### `tp.date.weekday(format: String = "YYYY-MM-DD", Weekday: Number, Reference?: String, reference_format?: string)`

#### Arguments

- `format`: Format for the date, refer to [format reference](https://momentjs.com/docs/#/displaying/format/)

- `reference`: The date referential, e.g. set this to the note's title

- `reference_format`: The date reference format.

- `weekday`: Week day number. If the locale assigns Monday as the first day of the week, `0` will be Monday, `-7` will be last week's day.

### `tp.date.yesterday(format: String = "YYYY-MM-DD")`

Retrieves yesterday's date.

#### Arguments

- `format`: Format for the date, refer to [format reference](https://momentjs.com/docs/#/displaying/format/)

## Moment.js

Templater gives you access to the `moment` object, with all of its functionalities.

More information on moment.js [here](https://momentjs.com/docs/#/displaying/)

## Examples

```javascript
Date now: <% tp.date.now() %>
Date now with format: <% tp.date.now("Do MMMM YYYY") %>

Last week: <% tp.date.now("dddd Do MMMM YYYY", -7) %>
Today: <% tp.date.now("dddd Do MMMM YYYY, ddd") %>
Next week: <% tp.date.now("dddd Do MMMM YYYY", 7) %>

Last month: <% tp.date.now("YYYY-MM-DD", "P-1M") %>
Next year: <% tp.date.now("YYYY-MM-DD", "P1Y") %>

File's title date + 1 day (tomorrow): <% tp.date.now("YYYY-MM-DD", 1, tp.file.title, "YYYY-MM-DD") %>
File's title date - 1 day (yesterday): <% tp.date.now("YYYY-MM-DD", -1, tp.file.title, "YYYY-MM-DD") %>

Date tomorrow with format: <% tp.date.tomorrow("Do MMMM YYYY") %>

This week's monday: <% tp.date.weekday("YYYY-MM-DD", 0) %>
Next monday: <% tp.date.weekday("YYYY-MM-DD", 7) %>
File's title monday: <% tp.date.weekday("YYYY-MM-DD", 0, tp.file.title, "YYYY-MM-DD") %>
File's title next monday: <% tp.date.weekday("YYYY-MM-DD", 7, tp.file.title, "YYYY-MM-DD") %>

Date yesterday with format: <% tp.date.yesterday("Do MMMM YYYY") %>
```
