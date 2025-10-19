---
title: attachment_file_suggester
aliases:
  - Attachment File Suggester
  - Suggester for Attachment File
  - attachment file suggester
plugin: templater
language:
  - javascript
module:
  - system
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-29T11:24
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Attachment File Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Return an attachment file, or any file depending on the directory, using a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Attachment folder variables
const sys_attachments_dir = `00_system/02_attachments/`;
const sys_atch_contacts_dir = `00_system/02_attachments/_51_contacts/`;
const sys_atch_organizations_dir = `00_system/02_attachments/_52_organizations/`;
const sys_atch_books_dir = `00_system/02_attachments/_61_books/`;

//---------------------------------------------------------
// SET IMAGE PATH
//---------------------------------------------------------
// Retrieve the directory files
const vault_files = await tp.user.vault_file(<directory>);

// Choose the cover image
const file_path = await tp.system.suggester(
  vault_files,
  vault_files,
  false,
  `File?`
);
```

### Templater

<!-- Add the full code excluding explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET IMAGE PATH
//---------------------------------------------------------
const vault_files = await tp.user.vault_file(<directory>);
const file_path = await tp.system.suggester(
  vault_files,
  vault_files,
  false,
  `Image?`
);
```

#### Examples

```javascript
//---------------------------------------------------------
// SET BOOK COVER IMAGE PATH
//---------------------------------------------------------
const sys_atch_books_dir = `00_system/02_attachments/_61_books/`;
const book_image_files = await tp.user.vault_file(sys_attachments_books_dir);
const file_path = await tp.system.suggester(
  book_image_files,
  book_image_files,
  false,
  "Book Cover?"
);
```

```javascript
//---------------------------------------------------------
// SET CONTACT PROFILE PICTURE
//---------------------------------------------------------
const sys_atch_contacts_dir = `00_system/02_attachments/_51_contacts/`;
const contact_picture_files = await tp.user.vault_file(
  sys_atch_contacts_dir
);
let picture_path = await tp.system.suggester(
  contact_picture_files,
  contact_picture_files,
  false,
  `Profile picture?`
);

if (picture_path == "_user_input") {
  picture_path = `${file_name}_pic.jpg`;
}
```

```javascript
//---------------------------------------------------------
// SET ORGANIZATION LOGO PICTURE
//---------------------------------------------------------
const sys_atch_organizations_dir = `00_system/02_attachments/_52_organizations/`;
const organization_picture_files = await tp.user.vault_file(
  sys_atch_organizations_dir
);
let picture_path = await tp.system.suggester(
  organization_picture_files,
  organization_picture_files,
  false,
  `Organization logo?`
);

if (picture_path == "_user_input") {
  picture_path = `${file_name}_pic.jpg`;
}
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[61_contact|Contact Template]]
2. [[62_organization|Organization Template]]
3. [[71_00_book|Book Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[get_file|Get Vault Files]]

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

1. [[tp.system.suggester Templater Function|The Templater tp.system.suggester() Function]]

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
