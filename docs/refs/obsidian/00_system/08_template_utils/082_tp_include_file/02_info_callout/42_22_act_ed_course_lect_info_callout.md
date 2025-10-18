> [!action_item] Action Item Details
> 
> - **Life Context**: `dv: join(filter(nonnull(flat([join(map(split(this.file.frontmatter.context, "_"), (x) => upper(x[0]) + substring(x, 1)), " and "), this.file.frontmatter.pillar])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.goal, this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) => !contains(lower(x), "null")), " | ")`
> - **Directory**: `dv: join(filter(nonnull(flat([this.file.frontmatter.organization, this.file.frontmatter.contact])), (x) => !contains(lower(x), "null")), " | ")`
> 
> - **Course**: `dvjs: dv.page((dv.current().file.frontmatter.library[0]).replaceAll(/^(\[\[)|(\|.+)$/g, "")).file.frontmatter.course`
> - **Lecture**: `dv: this.file.frontmatter.library`
>
> - **Date**: `dv: this.file.frontmatter.date`

---
