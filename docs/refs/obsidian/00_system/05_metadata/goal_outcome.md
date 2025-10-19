---
limit: 100
mapWithTag: true
icon: rocket
tagNames:
  - outcome_goal
excludes:
extends:
version: "2.0"
date_created: 2023-06-12T21:03
date_modified: 2023-09-05T19:18
fields:
  - id: qvwZf0
    name: pillar
    options:
      valuesList: {}
      sourceType: ValuesFromDVQuery
      valuesListNotePath: ""
      valuesFromDVQuery: dv.pages('"20_pillars"').file.name
    type: Multi
    path: ""
---

pillar:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"20_pillars\"').file.name"}}
