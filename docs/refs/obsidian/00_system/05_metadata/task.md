---
limit: 100
mapWithTag: false
icon: check
tagNames: 
excludes: 
extends: 
version: "2.18"
date_created: 2023-07-02T20:15
date_modified: 2023-11-26T10:52
fields:
  - id: W2DxvQ
    name: date
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "true"
    type: Date
    path: ""
  - id: eeex6e
    name: task_start
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "true"
    type: Date
    path: ""
  - id: 28Igk0
    name: task_end
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "true"
    type: Date
    path: ""
  - id: ZHOwWB
    name: due_do
    options:
      valuesList:
        "1": do
        "2": due
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
    command:
      id: insert__task__pillar
      icon: landmark
      label: Insert pillar field
    path: ""
    id: tBxPUl
  - name: goal
    type: MultiFile
    options:
      dvQueryString: dv.pages('"30_goals"').file
      customRendering: page.file.frontmatter.aliases[0]
    command:
      id: insert__task__goal
      icon: trophy
      label: Insert goal field
    path: ""
    id: utItX9
  - id: 7t8p57
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
  - name: project
    type: MultiFile
    options:
      dvQueryString: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.filter((x) => String(x.frontmatter.file_class).includes("project"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__task__project_
      icon: clipboard-list
      label: Insert project_ field
    path: ""
    id: 1tqZYT
  - name: parent_task
    type: MultiFile
    options:
      dvQueryString: dv.pages('"41_personal" OR "42_education" OR "43_professional" OR "44_work" OR "45_habit_ritual"').file.filter((x) => String(x.frontmatter.file_class).includes("parent"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__task__parent_task
      icon: list-checks
      label: Insert parent_task field
    path: ""
    id: qCObAC
  - id: P6tdCO
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
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: 4LlzkA
    name: type
    options:
      valuesList:
        "1": project
        "2": parent_task
        "3": action_item
        "4": meeting
        "5": ritual
        "6": habit
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - name: organization
    type: MultiFile
    options:
      dvQueryString: dv.pages('"52_organizations"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__task__organization
      icon: building
      label: Insert organization field
    path: ""
    id: fryhoT
  - name: contact
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__task__contact
      icon: contact
      label: Insert contact field
    path: ""
    id: PeTdmq
  - name: library
    type: MultiFile
    options:
      dvQueryString: dv.pages('"60_library"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__task__library
      icon: library
      label: Insert library field
    path: ""
    id: ELIEl1
filesPaths: 
bookmarksGroups: 
savedViews: []
favoriteView: 
---

date:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

task_start:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

task_end:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

due_do:: {"type":"Select","options":{"valuesList":{"1":"do","2":"due"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

pillar:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"20_pillars\"').file.name"},"command":{"id":"insert__task__pillar","icon":"landmark","label":"Insert pillar field"}}

<!-- {"type":"MultiFile","options":{"dvQueryString":"dv.pages('\"20_pillars\"')","customRendering":"page.file.frontmatter.aliases[0]","customSorting":"a.basename < b.basename? -1: 1"}} -->

context:: {"type":"Select","options":{"valuesList":{"1":"personal","2":"habit_ritual","3":"education","4":"professional","5":"work"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

goal:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"30_goals\"').file.name"},"command":{"id":"insert__task__goal","icon":"trophy","label":"Insert goal field"}}

project:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"40_projects\"').file\n.filter((x) => String(x.frontmatter.file_class).includes(\"project\"))\n.map((p) => p.name)\n.sort()"},"command":{"id":"insert__task__project","icon":"clipboard-list","label":"Insert project field"}}

parent_task:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"40_projects\"').file\n.filter((x) => String(x.frontmatter.file_class).includes(\"parent\"))\n.map((p) => p.name)\n.sort()"},"command":{"id":"insert__task__parent_task","icon":"list-checks","label":"Insert parent_task field"}}

organization:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"52_organizations\"').file.name.sort()"},"command":{"id":"insert__task__organization","icon":"building","label":"Insert organization field"}}

contact:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"51_contacts\"').file.name.sort()"},"command":{"id":"insert__task__contact","icon":"contact","label":"Insert contact field"}}

library:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"60_library\"').file.name.sort()"},"command":{"id":"insert__task__library","icon":"school","label":"Insert library field"}}

status:: {"type":"Select","options":{"valuesList":{"1":"to_do","2":"in_progress","3":"done","4":"discarded","5":"schedule","6":"on_hold"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""},"command":{"id":"insert__task__status","icon":"square-stack","label":"Insert status field"}}

type:: {"type":"Select","options":{"valuesList":{"1":"project","2":"parent_task","3":"action_item","4":"meeting","5":"ritual","6":"habit"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}
