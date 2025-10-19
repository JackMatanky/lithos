---
limit: 100
mapWithTag: false
icon: clipboard-list
tagNames:
excludes:
  - date
  - weekday
  - week
  - year_day
extends: cal
version: "2.0"
date_created: 2023-05-22T00:17
date_modified: 2023-09-05T19:18
fields:
  - id: cYNSca
    name: type
    options:
      valuesList:
        "1": month
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

type:: {"type":"Select","options":{"valuesList":{"1":"month"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
