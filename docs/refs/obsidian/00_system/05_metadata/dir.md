---
limit: 100
mapWithTag: false
icon: clipboard-list
tagNames: 
excludes: 
extends: 
version: "2.2"
date_created: 2023-06-12T08:02
date_modified: 2023-09-21T12:05
fields:
  - id: 8UsRDt
    name: title
    options: {}
    type: Input
    path: ""
  - id: oQluKz
    name: aliases
    options: {}
    type: Input
    path: ""
  - id: GFBBW2
    name: country
    options:
      valuesList: {}
      sourceType: ValuesListNotePath
      valuesListNotePath: 00_system/03_metadata/_metadata_values/countries.md
      valuesFromDVQuery: ""
    type: Multi
    path: ""
    command:
      id: insert__GFBBW2
      icon: globe-2
      label: Insert country field
  - id: UW7W2S
    name: city
    options:
      valuesList: {}
      sourceType: ValuesListNotePath
      valuesListNotePath: 00_system/03_metadata/_metadata_values/cities.md
      valuesFromDVQuery: ""
    type: Multi
    path: ""
    command:
      id: insert__UW7W2S
      icon: land-plot
      label: Insert city field
  - id: E4RNZ4
    name: url
    options: {}
    type: Input
    path: ""
  - id: Yqgq0I
    name: linkedin_url
    options: {}
    type: Input
    path: ""
  - id: s1pbiT
    name: connection
    options:
      valuesList:
        "1": education
        "2": personal
        "3": professional
        "4": work
        "5": "null"
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Multi
    path: ""
  - id: TGI3Ry
    name: source
    options:
      valuesList:
        "1": family
        "2": shalem_college
        "3": secular_yeshiva
        "4": garin_tsabar
        "5": jolt
        "6": informed_decisions
        "7": hatashtit
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: dv.pages('"50_directory"').file.name
    type: Select
    path: ""
  - id: SCfDJK
    name: picture_path
    options: {}
    type: Input
    path: ""
---

title:: {"type":"Input","options":{}}

aliases:: {"type":"Input","options":{}}

country:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesListNotePath","valuesListNotePath":"00_system/04_metadata_values/countries.md","valuesFromDVQuery":""}}

city:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesListNotePath","valuesListNotePath":"00_system/04_metadata_values/cities.md","valuesFromDVQuery":""}}

url:: {"type":"Input","options":{}}

linkedin_url:: {"type":"Input","options":{}}

connection:: {"type":"Multi","options":{"valuesList":{"1":"education","2":"personal","3":"professional","4":"work","5":"null"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

source:: {"type":"Select","options":{"valuesList":{"1":"family","2":"shalem_college","3":"secular_yeshiva","4":"garin_tsabar","5":"jolt","6":"informed_decisions","7":"hatashtit"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"50_directory\"').file.name"}}

picture_path:: {"type":"Input","options":{}}
