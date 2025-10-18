> - **Name**: `dv: choice(regextest("\w", this.file.frontmatter.url), elink(this.file.frontmatter.url, this.file.frontmatter.aliases[0]), this.file.frontmatter.aliases[0])`
> - **Language**: `dv: this.file.frontmatter.topic[0]`
> - **Type**: `dv: upper(substring(this.file.frontmatter.type, 0, 1)) + substring(this.file.frontmatter.type, 1)`
> - **Subtype**: `dv: upper(substring(this.file.frontmatter.subtype, 0, 1)) + substring(this.file.frontmatter.subtype, 1)`
>
> - **Description**: `dv: this.file.frontmatter.about`

---
