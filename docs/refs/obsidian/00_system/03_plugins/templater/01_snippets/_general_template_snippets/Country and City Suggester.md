---
title: Country and City Suggester
aliases:
  - Country and City Suggester
  - Suggester for Country and City
  - country_city_suggester
  - suggester_country_city
plugin: templater
language:
  - javascript
module:
  - system
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-08T12:46
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Country and City Suggester Suggester

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Set the country and city using a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET COUNTRY
//---------------------------------------------------------
const country = await tp.user.country(tp);
const country_name = country.key;
const country_value = country.value;

//---------------------------------------------------------  
// SET CITY
//---------------------------------------------------------  
const city = await tp.user.suggester_location(tp, country_value);
const city_name = city.key;
const city_value = city.value;
```

### Templater

<!-- Add the full code as it appears in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------  
// SET COUNTRY
//---------------------------------------------------------
const country = await tp.user.country(tp);
const country_name = country.key;
const country_value = country.value;

//---------------------------------------------------------  
// SET CITY
//---------------------------------------------------------  
const city = await tp.user.suggester_location(tp, country_value);
const city_name = city.key;
const city_value = city.value;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[61_contact]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[country|Country Suggester]]
2. [[city suggester by country|City Suggester Filtered by Country]]

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
