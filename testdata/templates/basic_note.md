# {{if .title}}{{.title}}{{else}}Untitled Note{{end}}

Created: {{now "2006-01-02"}}
Author: {{if .author}}{{.author | toLower}}{{else}}{{"Unknown" | toLower}}{{end}}
Tags: {{if .tags}}{{.tags}}{{else}}general{{end}}

## Content

{{if .content}}{{.content}}{{else}}No content provided{{end}}

## Notes

This note was created using the basic_note template.
Project: {{if .project}}{{.project | toUpper}}{{else}}PERSONAL{{end}}
