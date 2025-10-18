---
mapWithTag: true
limit: 100
icon: archive
tagNames:
  - note
excludes: 
extends: pkm
version: "2.2"
date_created: 2023-07-16T21:02
date_modified: 2023-09-05T19:18
fields:
  - id: NuD4tm
    name: type
    options:
      valuesList:
        "1": quote
        "2": idea
        "3": summary
        "4": question
        "5": evidence
        "6": step
        "7": conclusion
        "8": theorem
        "9": proof
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
    command:
      id: insert__pkm_zettel__type
      icon: pointer
      label: Insert type field
---