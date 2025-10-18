---
tags: parent_task
limit: 100
mapWithTag: true
icon: list-checks
tagNames:
  - parent_task
excludes:
  - date
  - parent_task
extends: task
version: "2.2"
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - id: 2rawia
    name: type
    options:
      valuesList:
        "1": parent_task
        "2": habit
        "3": morning_ritual
        "4": workday_startup_ritual
        "5": workday_shutdown_ritual
        "6": evening_ritual
        "7": job_application
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: niZ2xU
    command:
      id: insert__task__status
      icon: square-stack
      label: Insert status field
    name: status
    options:
      valuesList:
        "1": to_do
        "2": in_progress
        "3": done
        "4": discarded
        "5": schedule
        "6": on_hold
        "7": applied
        "8": offer
        "9": rejected
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

type:: {"type":"Select","options":{"valuesList":{"1":"parent_task","2":"habit","3":"morning_ritual","4":"workday_startup_ritual","5":"workday_shutdown_ritual","6":"evening_ritual","7":"job_application"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

status:: {"type":"Select","options":{"valuesList":{"1":"to_do","2":"in_progress","3":"done","4":"discarded","5":"schedule","6":"on_hold","7":"review","8":"applied","9":"offer","10":"rejected"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""},"command":{"id":"insert__task__status","icon":"square-stack","label":"Insert status field"}}