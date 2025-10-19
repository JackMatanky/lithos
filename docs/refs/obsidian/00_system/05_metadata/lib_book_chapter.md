---
limit: 100
mapWithTag: true
icon: album
tagNames:
  - book_chapter
  - chapter
excludes:
  - cover_url
  - issue
extends: lib_book
version: "2.8"
filesPaths:
bookmarksGroups:
savedViews: []
favoriteView:
date_created: 2023-06-27T22:18
date_modified: 2023-09-05T19:18
fields:
  - id: 6nizLK
    name: type
    options:
      valuesList:
        "1": book_chapter
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - name: page_start
    type: Number
    options: {}
    command:
      id: insert__lib_book_chapter__page_start
      icon: start
      label: Insert page_start field
    path: ""
    id: lFPRVL
  - name: page_end
    type: Number
    options: {}
    command:
      id: insert__lib_book_chapter__page_end
      icon: end
      label: Insert page_end field
    path: ""
    id: 1i8JqY
  - name: library
    type: MultiFile
    options:
      dvQueryString: dv.pages('"60_library/61_books"').file.filter((x) => x.frontmatter.file_class == "lib_book")
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__lib_book_chapter__library
      icon: book
      label: Insert library field
    path: ""
    id: 1Tg1Xo
---
