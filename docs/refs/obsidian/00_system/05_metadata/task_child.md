---
mapWithTag: true
limit: 100
icon: check-square
tagNames:
  - action_item
  - meeting
excludes:
  - date_start
  - date_end
extends: task
version: "2.0"
date_created: 2023-05-22T00:17
date_modified: 2023-09-05T19:18
fields:
  - id: YmUbOg
    name: type
    options:
      valuesList:
        "1": action_item
        "2": meeting
        "3": phone_call
        "4": video_call
        "5": interview
        "6": appointment
        "7": event
        "8": gathering
        "9": hangout
        "10": habit
        "11": morning_ritual
        "12": workday_startup_ritual
        "13": workday_shutdown_ritual
        "14": evening_ritual
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: Ulm6pU
    command:
      id: insert__presetField__time_start
      icon: play-circle
      label: Insert time_start field
    name: time_start
    options:
      dateFormat: HH:mm
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: YEe1hz
    command:
      id: insert__presetField__time_end
      icon: stop-circle
      label: Insert time_end field
    name: time_end
    options:
      dateFormat: HH:mm
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: GMjpst
    command:
      id: insert__presetField__duration_est
      icon: hourglass
      label: Insert duration_est field
    name: duration_est
    options: {}
    type: Number
    path: ""
---

type:: {"type":"Select","options":{"valuesList":{"1":"action_item","2":"meeting","3":"phone_call","4":"video_call","5":"interview","6":"appointment","7":"event","8":"gathering","9":"hangout","10":"habit","11":"morning_ritual","12":"workday_startup_ritual","13":"workday_shutdown_ritual","14":"evening_ritual"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

time_start:: {"type":"Date","options":{"dateFormat":"HH:mm","defaultInsertAsLink":"false"},"command":{"id":"insert__presetField__time_start","icon":"play-circle","label":"Insert time_start field"}}

time_end:: {"type":"Date","options":{"dateFormat":"HH:mm","defaultInsertAsLink":"false"},"command":{"id":"insert__presetField__time_end","icon":"stop-circle","label":"Insert time_end field"}}

duration_est:: {"type":"Number","options":{},"command":{"id":"insert__presetField__duration_est","icon":"hourglass","label":"Insert duration_est field"}}
