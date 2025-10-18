---
limit: 100
mapWithTag: false
icon: tree-pine
tagNames: 
excludes: 
extends: pkm
version: "2.1"
date_created: 2023-08-24T21:58
date_modified: 2023-09-05T19:18
fields:
  - id: liiRB9
    name: type
    options:
      valuesList:
        "1": category
        "2": branch
        "3": field
        "4": subject
        "5": topic
        "6": subtopic
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
    command:
      id: insert__pkm_tree__type
      icon: pointer
      label: Insert type field
---