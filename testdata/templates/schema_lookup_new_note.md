{{$project := lookup "project_alpha" -}}
{{$primary := lookup "john_doe" -}}
---
file_class: project
title: {{if $project}}{{$project.Title}}{{else}}Schema Lookup Project{{end}}
description: {{if $project}}{{$project.Frontmatter.Fields.description}}{{else}}Schema-driven lookup integration{{end}}
status: {{if $project}}{{$project.Frontmatter.Fields.status}}{{else}}planning{{end}}
team_members:
  - [[john_doe]]
  - [[jane_smith]]
primary_contact: [[john_doe]]
tags:
  - schema
  - lookup
---
# Schema Lookup Integration Note

Primary Contact: {{if $primary}}{{$primary.Title}}{{else}}<unknown>{{end}}

Generated: {{now "2006-01-02"}}
