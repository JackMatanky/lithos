---
mapWithTag: true
tags: contact, person
limit: 100
icon: contact
tagNames:
  - contact
excludes: 
extends: dir
version: "2.2"
date_created: 2023-09-03T19:26
date_modified: 2023-09-05T19:18
fields:
  - id: rZGDnh
    name: name_last
    options: {}
    type: Input
    path: ""
  - id: bJg1Of
    name: name_first
    options: {}
    type: Input
    path: ""
  - id: tUfw6R
    name: name_last_maiden
    options: {}
    type: Input
    path: ""
  - id: 1uk7Du
    name: gender
    options:
      valuesList:
        "1": female
        "2": male
        "3": other
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: RHS5vo
    name: date_birth
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: TPPq6b
    name: date_death
    options:
      dateFormat: YYYY-MM-DD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: tVeoRx
    name: address
    options: {}
    type: Input
    path: ""
  - id: 5RzUs5
    name: utc
    options:
      valuesList:
        "1": -12:00
        "2": -11:00
        "3": -10:00
        "4": -09:30
        "5": -09:00
        "6": -08:00
        "7": -07:00
        "8": -06:00
        "9": -05:00
        "10": -04:00
        "11": -03:30
        "12": -03:00
        "13": -02:00
        "14": -01:00
        "15": ±00:00
        "16": +01:00
        "17": +02:00
        "18": +03:00
        "19": +03:30
        "20": +04:00
        "21": +04:30
        "22": +05:00
        "23": +05:30
        "24": +05:45
        "25": +06:00
        "26": +06:30
        "27": +07:00
        "28": +08:00
        "29": +08:45
        "30": +09:00
        "31": +09:30
        "32": +10:00
        "33": +10:30
        "34": +11:00
        "35": +12:00
        "36": +12:45
        "37": +13:00
        "38": +14:00
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: BjuklH
    name: dst
    options:
      valuesList:
        "1": -12:00
        "2": -11:00
        "3": -10:00
        "4": -09:30
        "5": -09:00
        "6": -08:00
        "7": -07:00
        "8": -06:00
        "9": -05:00
        "10": -04:00
        "11": -03:30
        "12": -03:00
        "13": -02:00
        "14": -01:00
        "15": ±00:00
        "16": +01:00
        "17": +02:00
        "18": +03:00
        "19": +03:30
        "20": +04:00
        "21": +04:30
        "22": +05:00
        "23": +05:30
        "24": +05:45
        "25": +06:00
        "26": +06:30
        "27": +07:00
        "28": +08:00
        "29": +08:45
        "30": +09:00
        "31": +09:30
        "32": +10:00
        "33": +10:30
        "34": +11:00
        "35": +12:00
        "36": +12:45
        "37": +13:00
        "38": +14:00
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: Wr88iH
    name: phone_mobile
    options: {}
    type: Input
    path: ""
  - id: oFzRDb
    name: phone_home
    options: {}
    type: Input
    path: ""
  - id: JfWZEI
    name: phone_work
    options: {}
    type: Input
    path: ""
  - id: tYkomx
    name: email_personal
    options: {}
    type: Input
    path: ""
  - id: s2tOuo
    name: email_work
    options: {}
    type: Input
    path: ""
  - name: organization
    type: MultiFile
    options:
      dvQueryString: dv.pages('"52_organizations"').file
      customRendering: page.file.frontmatter.aliases[0]
      customSorting: "a.basename < b.basename ? 1 : -1"
    command:
      id: insert__dir_contact__organization
      icon: building
      label: Insert organization field
    path: ""
    id: UKhjy6
  - name: job_title
    type: Multi
    options:
      valuesList: {}
      sourceType: ValuesListNotePath
      valuesListNotePath: 00_system/03_metadata/_metadata_values/job_titles.md
      valuesFromDVQuery: ""
    path: ""
    id: SV2wbp
---

name_last:: {"type":"Input","options":{}}

name_first:: {"type":"Input","options":{}}

name_last_maiden:: {"type":"Input","options":{}}

gender:: {"type":"Select","options":{"valuesList":{"1":"female","2":"male","3":"other"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

date_birth:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

date_death:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

address:: {"type":"Input","options":{}}

utc:: {"type":"Select","options":{"valuesList":{"1":"-12:00","2":"-11:00","3":"-10:00","4":"-09:30","5":"-09:00","6":"-08:00","7":"-07:00","8":"-06:00","9":"-05:00","10":"-04:00","11":"-03:30","12":"-03:00","13":"-02:00","14":"-01:00","15":"±00:00","16":"+01:00","17":"+02:00","18":"+03:00","19":"+03:30","20":"+04:00","21":"+04:30","22":"+05:00","23":"+05:30","24":"+05:45","25":"+06:00","26":"+06:30","27":"+07:00","28":"+08:00","29":"+08:45","30":"+09:00","31":"+09:30","32":"+10:00","33":"+10:30","34":"+11:00","35":"+12:00","36":"+12:45","37":"+13:00","38":"+14:00"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

dst:: {"type":"Select","options":{"valuesList":{"1":"-12:00","2":"-11:00","3":"-10:00","4":"-09:30","5":"-09:00","6":"-08:00","7":"-07:00","8":"-06:00","9":"-05:00","10":"-04:00","11":"-03:30","12":"-03:00","13":"-02:00","14":"-01:00","15":"±00:00","16":"+01:00","17":"+02:00","18":"+03:00","19":"+03:30","20":"+04:00","21":"+04:30","22":"+05:00","23":"+05:30","24":"+05:45","25":"+06:00","26":"+06:30","27":"+07:00","28":"+08:00","29":"+08:45","30":"+09:00","31":"+09:30","32":"+10:00","33":"+10:30","34":"+11:00","35":"+12:00","36":"+12:45","37":"+13:00","38":"+14:00"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

phone_mobile:: {"type":"Input","options":{}}

phone_home:: {"type":"Input","options":{}}

phone_work:: {"type":"Input","options":{}}

email_personal:: {"type":"Input","options":{}}

email_work:: {"type":"Input","options":{}}

organization:: {"type":"Select","options":{"valuesList":{},"sourceType":"ValuesFromDVQuery","valuesListNotePath":"","valuesFromDVQuery":"dv.pages('\"52_organizations\"').file.name"}}

job_title:: {"type":"Multi","options":{"valuesList":{},"sourceType":"ValuesListNotePath","valuesListNotePath":"00_system/04_metadata_values/job_titles.md","valuesFromDVQuery":""}}

