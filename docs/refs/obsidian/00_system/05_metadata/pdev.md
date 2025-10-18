---
limit: 100
mapWithTag: false
icon: life-buoy
tagNames: 
excludes: 
extends: 
version: "2.5"
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - id: 7jC1kz
    name: title
    options: {}
    type: Input
    path: ""
  - id: M9pkgs
    name: aliases
    options: {}
    type: Input
    path: ""
  - id: bNJIOV
    name: date
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: sm1Iwj
    name: subtype
    options:
      valuesList:
        "1": daily
        "2": weekly
        "3": monthly
        "4": quarterly
        "5": yearly
        "6": prompt
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: 9Z4Id5
    name: type
    options:
      valuesList:
        "1": detachment
        "2": general
        "5": gratitude
        "6": limiting_belief
        "7": reflection
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - name: pillar
    type: MultiFile
    options:
      dvQueryString: dv.pages('"20_pillars"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pdev__pillar
      icon: landmark
      label: Insert pillar field
    path: ""
    id: ybXo8m
  - name: goal
    type: MultiFile
    options:
      dvQueryString: dv.pages('"30_goals"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pdev__goal
      icon: trophy
      label: Insert goal field
    path: ""
    id: 1ETc8B
  - name: project
    type: MultiFile
    options:
      dvQueryString: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.filter((x) => String(x.frontmatter.file_class).includes("project"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pdev__project
      icon: clipboard-list
      label: Insert project field
    path: ""
    id: bfa8Dy
  - name: parent_task
    type: MultiFile
    options:
      dvQueryString: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.filter((x) => String(x.frontmatter.file_class).includes("parent"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pdev__parent_task
      icon: list-checks
      label: Insert parent_task field
    path: ""
    id: rk7Gtz
---

title:: {"type":"Input","options":{}}

aliases:: {"type":"Input","options":{}}

date:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

pillar:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"20_pillars\"').file.name"}}

goal:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"30_goals\"').file.name"}}

project:: {"type":"Select","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"40_projects\"').file\n.filter((x) => String(x.frontmatter.type).includes(\"project\"))\n.map((p) => p.name)\n.sort()"}}

parent_task:: {"type":"Select","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"40_projects\"').file\n.filter((x) => String(x.frontmatter.type).includes(\"parent\"))\n.map((p) => p.name)\n.sort()"}}

subtype:: {"type":"Select","options":{"valuesList":{"1":"daily","2":"weekly","3":"monthly","4":"quarterly","5":"yearly","6":"prompt"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

type:: {"type":"Select","options":{"valuesList":{"1":"detachment","2":"general","5":"gratitude","6":"limiting_belief","7":"reflection"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
