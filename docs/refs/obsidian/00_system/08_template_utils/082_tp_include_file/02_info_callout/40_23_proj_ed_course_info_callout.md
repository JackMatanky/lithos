> [!project] Project Details
>
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Goal**: `dv: this.file.frontmatter.goal`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) => !contains(lower(x), "null")), " | ")`
>
> - **Course**: `dv: this.file.frontmatter.library`
>
> - **Dates**: `dv: join([this.file.frontmatter.task_start, this.file.frontmatter.task_end], " - ")`

---
