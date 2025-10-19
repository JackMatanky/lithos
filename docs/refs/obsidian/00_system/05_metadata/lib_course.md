---
limit: 100
mapWithTag: false
icon: presentation
tagNames:
excludes:
  - date_published
  - city
  - edition
  - volume
extends: lib
version: "2.8"
filesPaths:
bookmarksGroups:
savedViews: []
favoriteView:
date_created: 2023-06-12T21:03
date_modified: 2023-09-05T19:18
fields:
  - name: lecturer
    type: MultiFile
    options:
      dvQueryString: dv.pages('"51_contacts"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_course__lecturer
      icon: list-plus
      label: Insert lecturer field
    path: ""
    id: AwPMIL
  - name: institution
    type: MultiFile
    options:
      dvQueryString: dv.pages('"52_organizations"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_course__institution
      icon: institution
      label: Insert institution field
    path: ""
    id: eEH5jt
---
