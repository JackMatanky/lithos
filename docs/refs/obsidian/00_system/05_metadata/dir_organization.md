---
limit: 100
mapWithTag: true
icon: building
tagNames:
  - organization
excludes:
extends: dir
version: "2.1"
tags: organization
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - id: MH1xbW
    name: phone
    options: {}
    type: Input
    path: ""
  - id: SVSvvU
    name: email
    options: {}
    type: Input
    path: ""
  - id: jOZhAr
    name: about
    options: {}
    type: Input
    path: ""
  - id: Yw1ZoM
    name: industry
    options:
      valuesList: {}
      sourceType: ValuesListNotePath
      valuesListNotePath: 00_system/03_metadata/_metadata_values/industries.md
      valuesFromDVQuery: ""
    type: Multi
    path: ""
  - id: fhrDKN
    name: specialties
    options: {}
    type: Input
    path: ""
---

phone:: {"type":"Input","options":{}}

email:: {"type":"Input","options":{}}

about:: {"type":"Input","options":{}}

industry:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesListNotePath","valuesListNotePath":"00_system/04_metadata_values/industries.md","valuesFromDVQuery":""}}

specialties:: {"type":"Input","options":{}}
