---
limit: 100
mapWithTag: false
icon: calendar
tagNames: 
excludes: 
extends: 
version: "2.0"
date_created: 2023-06-12T08:14
date_modified: 2023-09-05T19:18
fields:
  - id: sj4qSz
    name: date
    options: []
    type: Input
    path: ""
  - id: TjpFU7
    name: date_start
    options:
      dateFormat: YYYY-MM-DDTHH:mm
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: uzTjMa
    name: date_end
    options:
      dateFormat: YYYY-MM-DDTHH:mm
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: 7Mkn1C
    name: year
    options:
      dateFormat: YYYY
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: OpyaxS
    name: year_day
    options:
      dateFormat: DDDD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: UqH1TD
    name: quarter
    options:
      dateFormat: Q
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: f1SYLL
    name: month_name
    options:
      valuesList:
        "1": January
        "2": February
        "3": March
        "4": April
        "5": May
        "6": June
        "7": July
        "8": August
        "9": September
        "10": October
        "11": November
        "12": December
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: 6J86j4
    name: month_number
    options:
      valuesList:
        "1": "01"
        "2": "02"
        "3": "03"
        "4": "04"
        "5": "05"
        "6": "06"
        "7": "07"
        "8": "08"
        "9": "09"
        "10": "10"
        "11": "11"
        "12": "12"
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: DFIXbL
    name: month_day
    options:
      dateFormat: DD
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: fmvQAe
    name: week_number
    options:
      dateFormat: ww
      defaultInsertAsLink: "false"
    type: Date
    path: ""
  - id: IYcM7V
    name: weekday_name
    options:
      valuesList:
        "1": Sunday
        "2": Monday
        "3": Tuesday
        "4": Wednesday
        "5": Thursday
        "6": Friday
        "7": Saturday
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: z6IEKM
    name: weekday_number
    options:
      valuesList:
        "1": "00"
        "2": "01"
        "3": "02"
        "4": "03"
        "5": "04"
        "6": "05"
        "7": "06"
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
  - id: w3QzF2
    name: type
    options:
      valuesList:
        "1": day
        "2": week
        "3": month
        "4": quarter
        "5": year
      sourceType: ValuesList
      valuesListNotePath: ""
      valuesFromDVQuery: ""
    type: Select
    path: ""
---

date:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}

date_start:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DDTHH:mm","defaultInsertAsLink":"false"}}

date_end:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DDTHH:mm","defaultInsertAsLink":"false"}}

year:: {"type":"Date","options":{"dateFormat":"YYYY","defaultInsertAsLink":"false"}}

year_day:: {"type":"Date","options":{"dateFormat":"DDDD","defaultInsertAsLink":"false"}}

quarter:: {"type":"Date","options":{"dateFormat":"Q","defaultInsertAsLink":"false"}}

month_name:: {"type":"Select","options":{"valuesList":{"1":"January","2":"February","3":"March","4":"April","5":"May","6":"June","7":"July","8":"August","9":"September","10":"October","11":"November","12":"December"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

month_number:: {"type":"Select","options":{"valuesList":{"1":"01","2":"02","3":"03","4":"04","5":"05","6":"06","7":"07","8":"08","9":"09","10":"10","11":"11","12":"12"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

month_day:: {"type":"Date","options":{"dateFormat":"DD","defaultInsertAsLink":"false"}}

week_number:: {"type":"Date","options":{"dateFormat":"ww","defaultInsertAsLink":"false"}}

weekday_name:: {"type":"Select","options":{"valuesList":{"1":"Sunday","2":"Monday","3":"Tuesday","4":"Wednesday","5":"Thursday","6":"Friday","7":"Saturday"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

weekday_number:: {"type":"Select","options":{"valuesList":{"1":"00","2":"01","3":"02","4":"03","5":"04","6":"05","7":"06"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

type:: {"type":"Select","options":{"valuesList":{"1":"day","2":"week","3":"month","4":"quarter","5":"year"},"sourceType":"ValuesList","valuesListNotePath":"","valuesFromDVQuery":""}}

date:: {"type":"Date","options":{"dateFormat":"YYYY-MM-DD","defaultInsertAsLink":"false"}}
