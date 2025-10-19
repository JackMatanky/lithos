> [!book] Book Details
>
> - **Author**: `dv: this.file.frontmatter.author`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Date Published**: `dv: this.file.frontmatter.year_published`
> - **Series**: `dv: choice((regextest(".", this.file.frontmatter.series) AND regextest(".", this.file.frontmatter.series_url)), this.file.frontmatter.series + ", " + elink(this.file.frontmatter.series_url, "link"), choice(regextest(".", this.file.frontmatter.series), this.file.frontmatter.series, ""))`
> - **Link**: `dv: elink(this.file.frontmatter.url, this.file.frontmatter.title)`
> - **About**: `dv: this.file.frontmatter.about`
>
> **Completed**::
