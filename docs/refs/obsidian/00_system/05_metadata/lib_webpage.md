---
limit: 100
mapWithTag: true
icon: globe
tagNames:
  - blog
  - webpage
excludes:
  - city
  - edition
  - series
  - volume
  - issue
  - page_start
  - page_end
  - isbn10
  - isbn13
  - doi
  - cover_url
  - cover_path
extends: lib
version: "2.0"
date_created: 2023-06-25T21:10
date_modified: 2023-09-05T19:18
fields:
  - id: Rr4xhC
    name: type
    options:
      valuesList:
        "1": webpage
        "2": blog
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

type:: {"type":"Select","options":{"valuesList":{"1":"webpage","2":"blog"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
