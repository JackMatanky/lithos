---
limit: 100
mapWithTag: true
icon: edit
tagNames:
  - journal
  - limiting_belief
excludes:
extends: pdev
version: "2.0"
date_created: 2023-06-21T19:57
date_modified: 2023-09-05T19:18
fields:
  - id: eCXgKs
    name: subtype
    options:
      valuesList:
        "1": daily
        "2": weekly
        "3": monthly
        "4": quarterly
        "5": yearly
        "6": hirability
        "7": self_appreciation
        "8": general
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: 8kRdIi
    name: type
    options:
      valuesList:
        "1": detachment
        "2": gratitude
        "3": limiting_belief
        "4": prompt
        "5": reflection
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

subtype:: {"type":"Select","options":{"valuesList":{"1":"daily","2":"weekly","3":"monthly","4":"quarterly","5":"yearly","6":"hirability","7":"self_appreciation","8":"general"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

type:: {"type":"Select","options":{"valuesList":{"1":"detachment","2":"gratitude","3":"limiting_belief","4":"prompt","5":"reflection"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
