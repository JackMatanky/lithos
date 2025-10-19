> [!contact] Contact Details
>
> **PERSONAL**
>
> - **Connection**: `dv: map(this.file.frontmatter.connection, (x) => (upper(substring(x, 0, 1)) + substring(x, 1)))`
> - **Source**: `dv: map(this.file.frontmatter.source, (x) => choice(regexmatch("^\w", x), join(map(split(x, "_"), (y) => (upper(substring(y, 0, 1)) + substring(y, 1))), " "), x))`
> - **Birthday**: `dv: choice(regextest("0001", this.file.frontmatter.date_birth), dateformat(date(this.file.frontmatter.date_birth), "MMMM dd"), dateformat(date(this.file.frontmatter.date_birth), "MMMM dd, yyyy"))`
>
> **CONTACT**
>
> - **Mobile**: `dv: this.file.frontmatter.phone_mobile`
> - **Personal Email**: `dv: this.file.frontmatter.email_personal`
> - **Work Email**: `dv: this.file.frontmatter.email_work`
>
> **PROFESSIONAL**
>
> - **Website**: `dv: elink(this.file.frontmatter.url, this.file.frontmatter.aliases[0] + " Website")`
> - **LinkedIn**: `dv: elink(this.file.frontmatter.linkedin_url, this.file.frontmatter.aliases[0] + " LinkedIn")`
> - **Job Title**: `dv: this.file.frontmatter.job_title`
> - **Organization**: `dv: this.file.frontmatter.organization`

---
