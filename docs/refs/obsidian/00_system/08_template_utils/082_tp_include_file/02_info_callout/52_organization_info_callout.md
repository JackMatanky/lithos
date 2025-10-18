> [!organization] Organization Details
>
> **PERSONAL**
>
> - **Connection**: `dv: map(this.file.frontmatter.connection, (x) => (upper(substring(x, 0, 1)) + substring(x, 1)))`
> - **Source**: `dv: map(this.file.frontmatter.source, (x) => choice(x = "job_application", "Job Application", choice(regexmatch("^\w", x), (upper(substring(x, 0, 1)) + substring(x, 1)), x)))`
>
> **CONTACT**
>
> - **Phone**: `dv: this.file.frontmatter.phone`
> - **Email**: `dv: this.file.frontmatter.email`
>
> **PROFESSIONAL**
>
> - **Website**: `dv: elink(this.file.frontmatter.url, this.file.frontmatter.aliases[0] + " Website")`
> - **LinkedIn**: `dv: elink(this.file.frontmatter.linkedin_url, this.file.frontmatter.aliases[0] + " LinkedIn")`
> - **Industry**: `dv: this.file.frontmatter.industry`
> - **Specialties**: `dv: this.file.frontmatter.specialties`
> - **About**: `dv: this.file.frontmatter.about`

---
