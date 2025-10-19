---
title: organization_specialties
aliases:
  - Organization Specialties
  - organization specialties
plugin: templater
language:
  - javascript
module:
  - user
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-27T18:18
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/prompt
---
# Organization Specialties

## Description

> [!snippet] Snippet Details
>
> Plugin:: [[Templater]]
> Language:: [[JavaScript]]
> Input::
> Output::
> Description:: Return an organization's specialties in title case, an of lowercase values, and as a list of tags

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET LINKEDIN SPECIALTIES
//---------------------------------------------------------
// Copy the specialties list from LinkedIn
const org_specialties = await tp.system.prompt(
  "Organization's specialties?"
);

// Return the organization's specialties in title case
const specialties_name = await tp.user.title_case(org_specialties);

// Remove the final "and" in the specialties list
const specialties_value = `[${specialties_name
  .replace(/\s(and)\s(\w+)$/g, " $2")
  .toLowerCase()}]`;

// Split the specialties string into an array
// Concatenate a "#" to each item without any spaces
// Join the array elements delimited by a space
const specialties_tag = specialties_name
  .replace(/\s(and)\s(\w+)$/g, " $2")
  .split(", ")
  .map((s) => "#" + s.replaceAll(/\s/g, "_"))
  .join(" ")
  .toLowerCase();
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET LINKEDIN SPECIALTIES
//---------------------------------------------------------
const org_specialties = await tp.system.prompt(
  "Organization's specialties?"
);
const specialties_name = await tp.user.title_case(org_specialties);
const specialties_value = `[${specialties_name
  .replace(/\s(and)\s(\w+)$/g, " $2")
  .toLowerCase()}]`;
const specialties_tag = specialties_name
  .replace(/\s(and)\s(\w+)$/g, " $2")
  .split(", ")
  .map((s) => "#" + s.replaceAll(/\s/g, "_"))
  .join(" ")
  .toLowerCase();
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript

```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[62_organization|Organization Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[title_case|Title Case]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
