---
limit: 100
mapWithTag: false
icon: presentation
tagNames: 
excludes: 
extends: lib_course
version: "2.2"
filesPaths: 
bookmarksGroups: 
savedViews: []
favoriteView: 
date_created: 2023-06-12T21:03
date_modified: 2023-09-05T19:18
fields:
  - name: libraby
    type: MultiFile
    options:
      dvQueryString: dv.pages('"60_library/68_courses"').file.filter((x) => x.frontmatter.file_class == "lib_course")
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_course_lecture__libraby
      icon: course
      label: Insert libraby field
    path: ""
    id: pQhikA
---
