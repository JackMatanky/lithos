---
limit: 100
mapWithTag: true
icon: book-open
tagNames:
  - book
excludes: 
extends: lib
version: "2.15"
filesPaths: 
bookmarksGroups: 
savedViews: []
favoriteView: 
date_created: 2023-06-12T21:03
date_modified: 2023-09-05T19:18
fields:
  - id: hvENcU
    name: type
    options:
      valuesList:
        "1": book
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
      id: insert__lib_book__author
      icon: pen
      label: Insert author field
    path: ""
    id: 2X4hyA
  - name: editor
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_book__editor
      icon: pen
      label: Insert editor field
    path: ""
    id: jR9GIS
  - name: translator
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_book__translator
      icon: pen
      label: Insert translator field
    path: ""
    id: ORuQEL
  - name: publisher
    type: MultiFile
    options:
      dvQueryString: dv.pages('"52_organizations"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_book__publisher
      icon: publisher
      label: Insert publisher field
    path: ""
    id: wEY3Ed
  - name: doi
    type: Input
    options: {}
    path: ""
    id: vdx57Z
  - name: isbn10
    type: Input
    options: {}
    path: ""
    id: SyEPfb
  - name: isbn13
    type: Input
    options: {}
    path: ""
    id: JXRdeB
  - name: year_published
    type: Date
    options:
      dateFormat: YYYY
      defaultInsertAsLink: "false"
    path: ""
    id: T4J80A
  - name: city
    type: Multi
    options:
      valuesList: {}
      sourceType: ValuesListNotePath
      valuesListNotePath: 00_system/03_metadata/_metadata_values/cities.md
      valuesFromDVQuery: ""
    command:
      id: insert__lib_book__city
      icon: city
      label: Insert city field
    path: ""
    id: GCDM67
  - name: edition
    type: Number
    options: {}
    command:
      id: insert__lib_book__edition
      icon: edition
      label: Insert edition field
    path: ""
    id: jZLe0B
  - name: cover_url
    type: Input
    options: {}
    command:
      id: insert__lib_book__cover_url
      icon: cover_url
      label: Insert cover_url field
    path: ""
    id: PlVNdg
  - name: cover_path
    type: File
    options: {}
    command:
      id: insert__lib_book__cover_path
      icon: cover_path
      label: Insert cover_path field
    path: ""
    id: 4Z9rIv
---