> - **Type**: `dv: choice(contains(this.file.frontmatter.file_class, "journal"), choice(!regextest("\w", this.file.frontmatter.subtype), "Limiting Belief", join(map([this.file.frontmatter.subtype, this.file.frontmatter.type], (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ")), join(map(split(this.file.frontmatter.type, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ") + " Vision")`
> - **Pillar**: `dv: this.file.frontmatter.pillar`
> - **Task Hierarchy**: `dv: join(filter(nonnull(flat([this.file.frontmatter.project, this.file.frontmatter.parent_task])), (x) =>!contains(lower(x), "null")), " | ")`
> - **Date**: `dv: this.file.frontmatter.date`
