---
mapWithTag: true
limit: 100
icon: heart-handshake
tagNames:
  - meeting
excludes:
  - date_start
  - date_end
extends: task
version: "2.1"
date_created: 2023-06-13T21:14
date_modified: 2023-09-05T19:18
fields:
  - id: 9I8bAi
    name: subtype
    options:
      valuesList:
        "1": meeting
        "2": phone_call
        "3": video_call
        "4": interview
        "5": appointment
        "6": lecture
        "7": tutorial
        "8": event
        "9": gathering
        "10": hangout
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

subtype:: {"type":"Select","options":{"valuesList":{"1":"meeting","2":"phone_call","3":"interview","4":"appointment","5":"event","6":"gathering","7":"hangout"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
