---
limit: 100
mapWithTag: false
icon: landmark
tagNames:
excludes:
extends:
version: "2.0"
date_created: 2023-05-22T00:17
date_modified: 2023-09-05T19:18
fields:
  - id: uPOCOs
    name: status
    options:
      valuesList:
        "1": active
        "2": on_hold
        "3": inactive
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: jkmlXt
    name: type
    options:
      valuesList:
        "1": growth
        "2": interpersonal
        "3": personal
        "4": professional
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

status:: {"type":"Select","options":{"valuesList":{"1":"active","2":"on_hold","3":"inactive"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

type:: {"type":"Select","options":{"valuesList":{"1":"growth","2":"interpersonal","3":"personal","4":"professional"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
