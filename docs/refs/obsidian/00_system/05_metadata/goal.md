---
limit: 100
mapWithTag: false
icon: trophy
tagNames: 
excludes: 
extends: 
version: "2.0"
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - id: rtvOAz
    name: title
    options: {}
    type: Input
    path: ""
  - id: mHVXMj
    name: aliases
    options: {}
    type: Input
    path: ""
  - id: SON4OO
    name: duration
    options:
      valuesList:
        "1": long_>1_year
        "2": medium_6-12_months
        "3": short_4-6_months
        "4": immediate_2-4_months
        "5": challenge_1-2_months
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: 15vKdP
    name: date_start
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: hPZOB2
    name: date_end
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: Zn6Upy
    name: context
    options:
      valuesList:
        "1": personal
        "2": habit_ritual
        "3": education
        "4": professional
        "5": work
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: Hcxa1j
    name: pillar
    options:
      valuesList: {}
      sourceType: ValuesFromDVQuery
      valuesListNotePath: ""
      valuesFromDVQuery: dv.pages('"20_pillars"').file.name
    type: Multi
    path: ""
  - id: K4T0Zs
    name: organization
    options:
      valuesList: {}
      sourceType: ValuesFromDVQuery
      valuesListNotePath: ""
      valuesFromDVQuery: dv.pages('"52_organizations"').file.name.sort()
    type: Multi
    path: ""
  - id: fUYaS8
    name: contact
    options:
      valuesList: {}
      sourceType: ValuesFromDVQuery
      valuesListNotePath: ""
      valuesFromDVQuery: dv.pages('"51_contacts"').file.name.sort()
    type: Multi
    path: ""
  - id: fX5sSd
    name: priority
    options:
      valuesList:
        "1": 1st_highest
        "2": 2nd_high
        "3": 3rd_medium
        "4": 4th_low
        "5": 5th_lowest
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: phw8Yz
    name: status
    options:
      valuesList:
        "1": to_do
        "2": in_progress
        "3": done
        "4": discarded
        "5": schedule
        "6": on_hold
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: zHLDi3
    name: type
    options:
      valuesList:
        "1": value
        "2": outcome
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

title:: {"type":"Input","options":{}}

aliases:: {"type":"Input","options":{}}

duration:: {"type":"Select","options":{"valuesList":{"1":"long_>1_year","2":"medium_6-12_months","3":"short_4-6_months","4":"immediate_2-4_months","5":"challenge_1-2_months"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

date_start:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

date_end:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

context:: {"type":"Select","options":{"valuesList":{"1":"personal","2":"habit_ritual","3":"education","4":"professional","5":"work"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

pillar:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"20_pillars\"').file.name"}}

organization:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"52_organizations\"').file.name.sort()"}}

contact:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"51_contacts\"').file.name.sort()"}}

priority:: {"type":"Select","options":{"valuesList":{"1":"1st_highest","2":"2nd_high","3":"3rd_medium","4":"4th_low","5":"5th_lowest"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

status:: {"type":"Select","options":{"valuesList":{"1":"to_do","2":"in_progress","3":"done","4":"discarded","5":"schedule","6":"on_hold"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

type:: {"type":"Select","options":{"valuesList":{"1":"value","2":"outcome"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
