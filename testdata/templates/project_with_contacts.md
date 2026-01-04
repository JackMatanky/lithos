{{$project := lookup "project_alpha" -}}
# Project Team Report: {{if $project}}{{$project.Title}}{{else}}<no value>{{end}}

**Generated:** {{now "2006-01-02"}}

## Project Overview
{{if and $project ($project.Frontmatter.Fields.description)}}
{{$project.Frontmatter.Fields.description}}
{{else}}
No description provided.
{{end}}

**Status:** {{if $project}}{{$project.Frontmatter.Fields.status}}{{else}}<no value>{{end}}
**Start Date:** {{if $project}}{{$project.Frontmatter.Fields.start_date}}{{else}}<no value>{{end}}
**Target Release:** {{if $project}}{{$project.Frontmatter.Fields.target_release}}{{else}}<no value>{{end}}

## Team Members
{{$contacts := query (dict "file_class" "contact") -}}
{{if $contacts}}
| Name | Email | Organization | Role |
|------|-------|--------------|------|
{{- range $contact := sortByTitle $contacts}}
| {{$contact.Title}} | {{$contact.Frontmatter.Fields.email}} | {{$contact.Frontmatter.Fields.organization}} | {{$contact.Frontmatter.Fields.role}} |
{{- end}}
{{else}}
No team members found.
{{end}}

## Contact Details
{{$johnDoe := lookup "john_doe" -}}
{{if $johnDoe}}
### Primary Contact

**{{$johnDoe.Title}}**
- Email: {{$johnDoe.Frontmatter.Fields.email}}
- Phone: {{$johnDoe.Frontmatter.Fields.phone}}
- Organization: {{$johnDoe.Frontmatter.Fields.organization}}
- FileClass: {{fileClass $johnDoe.Path}}
{{else}}
Primary contact not found.
{{end}}

## File Classes

{{if $project}}
Project Alpha FileClass: {{fileClass $project.Path}}
{{else}}
Project file class unknown.
{{end}}

{{if $contacts}}
{{range $contact := sortByTitle $contacts}}
- {{$contact.Title}} FileClass: {{fileClass $contact.Path}}
{{end}}
{{else}}
No contact file classes available.
{{end}}

## Tags

{{if and $project ($project.Frontmatter.Fields.tags)}}
{{- $sep := "" -}}
{{range $tag := $project.Frontmatter.Fields.tags}}
{{- printf "%s#%v" $sep $tag -}}
{{- $sep = " " -}}
{{end}}
{{else}}
No tags
{{end}}
