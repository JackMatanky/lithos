---
limit: 100
mapWithTag: false
icon: code-2
tagNames:
excludes:
extends: pkm
version: "2.0"
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - id: C7Qm9Y
    name: topic
    options:
      valuesList:
        "1": autohotkey
        "2": git
        "3": google_apps_script
        "4": google_sheets
        "5": javascript
        "6": microsoft_excel
        "7": python
        "8": r
        "9": sql
        "10": tableau
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Multi
    path: ""
  - id: NQaD6n
    name: syntax
    options: {}
    type: Input
    path: ""
  - id: QIpdxX
    name: url
    options: {}
    type: Input
    path: ""
  - id: pyyeuw
    name: subtype
    options:
      valuesList:
        "1": array
        "2": boolean
        "3": dataframe
        "4": dictionary
        "5": file
        "6": list
        "7": math
        "8": numeric
        "10": object
        "11": series
        "12": set
        "13": string
        "14": tuple
        "15": regex
        "16": general
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Multi
    path: ""
  - id: zRrAYL
    name: type
    options:
      valuesList:
        "1": clause
        "2": data_type
        "3": error
        "4": function
        "5": keyword
        "6": method
        "7": operator
        "8": snippet
        "9": statement
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

topic:: {"type":"Multi","options":{"valuesList":{"1":"autohotkey","2":"git","3":"google_apps_script","4":"google_sheets","5":"javascript","6":"microsoft_excel","7":"python","8":"r","9":"sql","10":"tableau"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

subtopic:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"70_pkm_tree\"').file\n.filter((x) => String(x.frontmatter.subtype).includes(\"subtopic\"))\n.map((p) => p.name)\n.sort()"}}

syntax:: {"type":"Input","options":{}}

url:: {"type":"Input","options":{}}

subtype:: {"type":"Multi","options":{"valuesList":{"1":"array","2":"boolean","3":"dataframe","4":"dictionary","5":"file","6":"list","7":"math","8":"numeric","10":"object","11":"series","12":"set","13":"string","14":"tuple","15":"regex","16":"general"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

type:: {"type":"Select","options":{"valuesList":{"1":"clause","2":"data_type","3":"error","4":"function","5":"keyword","6":"method","7":"operator","8":"snippet","9":"statement"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
