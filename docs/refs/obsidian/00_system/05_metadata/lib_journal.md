---
limit: 100
mapWithTag: true
icon: scroll
tagNames:
  - journal
excludes: 
extends: lib
version: "2.8"
filesPaths: 
bookmarksGroups: 
savedViews: []
favoriteView: 
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - name: author
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    path: ""
    id: qFjXw5
    command:
      id: insert__qFjXw5
      icon: author
      label: Insert author field
    display: asList
  - name: translator
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_journal__translator
      icon: translator
      label: Insert translator field
    path: ""
    id: IZ0CpO
  - name: publisher
    type: MultiFile
    options:
      dvQueryString: dv.pages('"52_organizations"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_journal__publisher
      icon: list-plus
      label: Insert publisher field
    path: ""
    id: yTcwjm
  - id: 3X5ybv
    name: year_published
    options:
      dateFormat: YYYY
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: 3E4P3t
    name: volume
    options:
      step: "1"
    type: Number
    path: ""
  - id: VHTtn0
    name: issue
    options:
      step: "1"
    type: Number
    path: ""
  - id: dUHe9o
    name: page_start
    options:
      step: "1"
    type: Number
    path: ""
  - id: Zb16pW
    name: page_end
    options:
      step: "1"
    type: Number
    path: ""
  - id: ubSF0z
    name: doi
    options: {}
    type: Input
    path: ""
---