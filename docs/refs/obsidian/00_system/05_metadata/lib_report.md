---
limit: 100
mapWithTag: true
icon: file-line-chart
tagNames:
  - report
excludes:
  - city
  - edition
  - isbn10
  - isbn13
  - cover_url
  - cover_path
  - type
extends: lib
version: "2.0"
date_created: 2023-06-12T21:03
date_modified: 2023-09-05T19:18
fields:
  - id: g8fX8D
    name: type
    options:
      valuesList:
        "1": report
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

type:: {"type":"Select","options":{"valuesList":{"1":"report"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
