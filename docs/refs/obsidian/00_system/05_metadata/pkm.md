---
limit: 100
mapWithTag: false
icon: brain
tagNames:
excludes:
extends:
version: "2.13"
date_created: 2023-09-03T19:26
date_modified: 2023-09-12T18:48
fields:
  - id: 1zaVFx
    name: title
    options: {}
    type: Input
    path: ""
  - id: pZyVia
    name: aliases
    options: {}
    type: Input
    path: ""
  - id: TlwD8g
    name: type
    options:
      valuesList:
        "1": quote
        "2": idea
        "3": summary
        "4": question
        "5": evidence
        "6": step
        "7": conclusion
        "8": definition
        "9": general
        "10": category
        "11": branch
        "12": field
        "13": subject
        "14": topic
        "15": subtopic
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
    command:
      id: insert__pkm__type
      icon: pointer
      label: Insert type field
  - id: kJWlup
    name: status
    options:
      valuesList:
        "1": review
        "2": clarify
        "3": develop
        "4": permanent
        "5": resource
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
      id: insert__pkm__pillar
      icon: landmark
      label: Insert pillar field
    path: ""
    id: 2rF02z
  - name: category
    type: MultiFile
    options:
      dvQueryString: dv.pages('"70_pkm"').file.filter((x) => String(x.frontmatter.subtype).includes("category"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pkm__category
      icon: tree
      label: Insert category field
    path: ""
    id: dtgRCf
  - name: branch
    type: MultiFile
    options:
      dvQueryString: dv.pages('"70_pkm"').file.filter((x) => String(x.frontmatter.subtype).includes("branch"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pkm__branch
      icon: branch
      label: Insert branch field
    path: ""
    id: 2MBROh
  - name: field
    type: MultiFile
    options:
      dvQueryString: dv.pages('"70_pkm"').file.filter((x) => String(x.frontmatter.subtype).includes("field"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pkm__field
      icon: field
      label: Insert field field
    path: ""
    id: cNuTA6
  - name: subject
    type: MultiFile
    options:
      dvQueryString: dv.pages('"70_pkm"').file.filter((x) => String(x.frontmatter.subtype).includes("subject"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pkm__subject
      icon: subject
      label: Insert subject field
    path: ""
    id: MtPlEY
  - name: topic
    type: MultiFile
    options:
      dvQueryString: dv.pages('"70_pkm"').file.filter((x) => String(x.frontmatter.subtype).includes("topic"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pkm__topic
      icon: topic
      label: Insert topic field
    path: ""
    id: vr38Q5
  - name: subtopic
    type: MultiFile
    options:
      dvQueryString: dv.pages('"70_pkm"').file.filter((x) => String(x.frontmatter.subtype).includes("subtopic"))
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__pkm__subtopic
      icon: subtopic
      label: Insert subtopic field
    path: ""
    id: 9qnXnB
  - name: library
    type: MultiFile
    options:
      dvQueryString: dv.pages('"60_library"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    path: ""
    id: 3EqZaC
---
