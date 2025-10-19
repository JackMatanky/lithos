---
limit: 100
mapWithTag: false
icon: newspaper
tagNames:
excludes:
  - city
  - edition
  - isbn10
  - isbn13
  - doi
  - cover_url
  - cover_path
extends: lib
version: "2.0"
date_created: 2023-06-12T21:03
date_modified: 2023-09-05T19:18
fields:
  - id: DEI3SI
    name: type
    options:
      valuesList:
        "1": magazine
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

type:: {"type":"Select","options":{"valuesList":{"1":"magazine"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
