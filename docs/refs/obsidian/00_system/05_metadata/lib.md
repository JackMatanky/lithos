---
limit: 100
mapWithTag: false
icon: school
tagNames:
excludes:
extends:
version: "2.9"
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - id: CyUvV1
    name: title
    options: {}
    type: Input
    path: ""
  - id: PtspuA
    name: aliases
    options: {}
    type: Input
    path: ""
  - name: main_title
    type: Input
    options: {}
    command:
      id: insert__lib__main_title
      icon: title
      label: Insert main_title field
    path: ""
    id: m6P1ze
  - id: 4FX3ae
    name: subtitle
    options: {}
    type: Input
    path: ""
  - id: jIvhCM
    name: date_published
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: BqmNeO
    name: series
    options: {}
    type: Input
    path: ""
  - id: yWFRKT
    name: volume
    options:
      step: "1"
    type: Number
    path: ""
  - id: yk3C3C
    name: url
    options: {}
    type: Input
    path: ""
  - id: yh34lo
    name: about
    options: {}
    type: Input
    path: ""
    command:
      id: insert__lib__about
      icon: question
      label: Insert about field
  - id: vgHUrY
    name: status
    options:
      valuesList:
        "1": undetermined
        "2": schedule
        "3": to_do
        "4": in_progress
        "5": done
        "6": resource
        "7": on_hold
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: ECHkwk
    name: type
    options:
      valuesList:
        "1": book
        "2": book_chapter
        "3": journal
        "4": report
        "5": news
        "6": magazine
        "7": webpage
        "8": blog
        "9": video
        "10": youtube
        "11": documentary
        "12": audio
        "13": podcast
        "14": documentation
        "15": course
        "16": course_lecture
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - name: author
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib__author
      icon: pen
      label: Insert author field
    path: ""
    id: gVUImO
  - name: editor
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib__editor
      icon: pen
      label: Insert editor field
    path: ""
    id: iB90qD
  - name: translator
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib__translator
      icon: pen
      label: Insert translator field
    path: ""
    id: XPTMft
  - name: publisher
    type: MultiFile
    options:
      dvQueryString: dv.pages('"52_organizations"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib__publisher
      icon: printer
      label: Insert publisher field
    path: ""
    id: sVjtp7
---
