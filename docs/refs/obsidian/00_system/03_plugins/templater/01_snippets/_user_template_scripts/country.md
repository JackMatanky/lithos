---
title: country
aliases:
  - Country Suggester
  - country_suggester
plugin: templater
language:
  - javascript
module:
description:
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-05-01T09:36
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester
---
# Country Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input::
> Output::
> Description:: Choose a country from a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
const country_obj_arr = [
  { key: "Null", value: "null" },
  { key: `Afghanistan`, value: `afghanistan` },
  { key: `Albania`, value: `albania` },
  { key: `Algeria`, value: `algeria` },
  { key: `Andorra`, value: `andorra` },
  { key: `Angola`, value: `angola` },
  { key: `Antigua and Barbuda`, value: `antigua_barbuda` },
  { key: `Argentina`, value: `argentina` },
  { key: `Armenia`, value: `armenia` },
  { key: `Australia`, value: `australia` },
  { key: `Austria`, value: `austria` },
  { key: `Azerbaijan`, value: `azerbaijan` },
  { key: `Bahamas`, value: `bahamas` },
  { key: `Bahrain`, value: `bahrain` },
  { key: `Bangladesh`, value: `bangladesh` },
  { key: `Barbados`, value: `barbados` },
  { key: `Belarus`, value: `belarus` },
  { key: `Belgium`, value: `belgium` },
  { key: `Belize`, value: `belize` },
  { key: `Benin`, value: `benin` },
  { key: `Bhutan`, value: `bhutan` },
  { key: `Bolivia`, value: `bolivia` },
  { key: `Bosnia and Herzegovina`, value: `bosnia_herzegovina` },
  { key: `Botswana`, value: `botswana` },
  { key: `Brazil`, value: `brazil` },
  { key: `Brunei`, value: `brunei` },
  { key: `Bulgaria`, value: `bulgaria` },
  { key: `Burkina Faso`, value: `burkina_faso` },
  { key: `Burundi`, value: `burundi` },
  { key: `Cabo Verde`, value: `cabo_verde` },
  { key: `Cambodia`, value: `cambodia` },
  { key: `Cameroon`, value: `cameroon` },
  { key: `Canada`, value: `canada` },
  { key: `Central African Republic`, value: `central_african_republic` },
  { key: `Chad`, value: `chad` },
  { key: `Chile`, value: `chile` },
  { key: `China`, value: `china` },
  { key: `Colombia`, value: `colombia` },
  { key: `Comoros`, value: `comoros` },
  {
    key: `Democratic Republic of the Congo`,
    value: `democratic_republic_of_the_congo`,
  },
  { key: `Republic of the Congo`, value: `republic_of_the_congo` },
  { key: `Costa Rica`, value: `costa_rica` },
  { key: `Cote d'Ivoire`, value: `cote_d'ivoire` },
  { key: `Croatia`, value: `croatia` },
  { key: `Cuba`, value: `cuba` },
  { key: `Cyprus`, value: `cyprus` },
  { key: `Czechia`, value: `czechia` },
  { key: `Denmark`, value: `denmark` },
  { key: `Djibouti`, value: `djibouti` },
  { key: `Dominica`, value: `dominica` },
  { key: `Dominican Republic`, value: `dominican_republic` },
  { key: `Ecuador`, value: `ecuador` },
  { key: `Egypt`, value: `egypt` },
  { key: `El Salvador`, value: `el_salvador` },
  { key: `England`, value: `england` },
  { key: `Equatorial Guinea`, value: `equatorial_guinea` },
  { key: `Eritrea`, value: `eritrea` },
  { key: `Estonia`, value: `estonia` },
  { key: `Eswatini`, value: `eswatini` },
  { key: `Ethiopia`, value: `ethiopia` },
  { key: `Fiji`, value: `fiji` },
  { key: `Finland`, value: `finland` },
  { key: `France`, value: `france` },
  { key: `Gabon`, value: `gabon` },
  { key: `Gambia`, value: `gambia` },
  { key: `Georgia`, value: `georgia` },
  { key: `Germany`, value: `germany` },
  { key: `Ghana`, value: `ghana` },
  { key: `Greece`, value: `greece` },
  { key: `Grenada`, value: `grenada` },
  { key: `Guatemala`, value: `guatemala` },
  { key: `Guinea`, value: `guinea` },
  { key: `Guinea-Bissau`, value: `guinea-bissau` },
  { key: `Guyana`, value: `guyana` },
  { key: `Haiti`, value: `haiti` },
  { key: `Honduras`, value: `honduras` },
  { key: `Hungary`, value: `hungary` },
  { key: `Iceland`, value: `iceland` },
  { key: `India`, value: `india` },
  { key: `Indonesia`, value: `indonesia` },
  { key: `Iran`, value: `iran` },
  { key: `Iraq`, value: `iraq` },
  { key: `Ireland`, value: `ireland` },
  { key: `Israel`, value: `israel` },
  { key: `Italy`, value: `italy` },
  { key: `Jamaica`, value: `jamaica` },
  { key: `Japan`, value: `japan` },
  { key: `Jordan`, value: `jordan` },
  { key: `Kazakhstan`, value: `kazakhstan` },
  { key: `Kenya`, value: `kenya` },
  { key: `Kiribati`, value: `kiribati` },
  { key: `Kosovo`, value: `kosovo` },
  { key: `Kuwait`, value: `kuwait` },
  { key: `Kyrgyzstan`, value: `kyrgyzstan` },
  { key: `Laos`, value: `laos` },
  { key: `Latvia`, value: `latvia` },
  { key: `Lebanon`, value: `lebanon` },
  { key: `Lesotho`, value: `lesotho` },
  { key: `Liberia`, value: `liberia` },
  { key: `Libya`, value: `libya` },
  { key: `Liechtenstein`, value: `liechtenstein` },
  { key: `Lithuania`, value: `lithuania` },
  { key: `Luxembourg`, value: `luxembourg` },
  { key: `Madagascar`, value: `madagascar` },
  { key: `Malawi`, value: `malawi` },
  { key: `Malaysia`, value: `malaysia` },
  { key: `Maldives`, value: `maldives` },
  { key: `Mali`, value: `mali` },
  { key: `Malta`, value: `malta` },
  { key: `Marshall Islands`, value: `marshall_islands` },
  { key: `Mauritania`, value: `mauritania` },
  { key: `Mauritius`, value: `mauritius` },
  { key: `Mexico`, value: `mexico` },
  { key: `Micronesia`, value: `micronesia` },
  { key: `Moldova`, value: `moldova` },
  { key: `Monaco`, value: `monaco` },
  { key: `Mongolia`, value: `mongolia` },
  { key: `Montenegro`, value: `montenegro` },
  { key: `Morocco`, value: `morocco` },
  { key: `Mozambique`, value: `mozambique` },
  { key: `Myanmar`, value: `myanmar` },
  { key: `Namibia`, value: `namibia` },
  { key: `Nauru`, value: `nauru` },
  { key: `Nepal`, value: `nepal` },
  { key: `Netherlands`, value: `netherlands` },
  { key: `New Zealand`, value: `new_zealand` },
  { key: `Nicaragua`, value: `nicaragua` },
  { key: `Niger`, value: `niger` },
  { key: `Nigeria`, value: `nigeria` },
  { key: `North Korea`, value: `north_korea` },
  { key: `North Macedonia`, value: `north_macedonia` },
  { key: `Norway`, value: `norway` },
  { key: `Oman`, value: `oman` },
  { key: `Pakistan`, value: `pakistan` },
  { key: `Palau`, value: `palau` },
  { key: `Palestine`, value: `palestine` },
  { key: `Panama`, value: `panama` },
  { key: `Papua New Guinea`, value: `papua_new_guinea` },
  { key: `Paraguay`, value: `paraguay` },
  { key: `Peru`, value: `peru` },
  { key: `Philippines`, value: `philippines` },
  { key: `Poland`, value: `poland` },
  { key: `Portugal`, value: `portugal` },
  { key: `Qatar`, value: `qatar` },
  { key: `Romania`, value: `romania` },
  { key: `Russia`, value: `russia` },
  { key: `Rwanda`, value: `rwanda` },
  { key: `Saint Kitts and Nevis`, value: `saint_kitts_nevis` },
  { key: `Saint Lucia`, value: `saint_lucia` },
  {
    key: `Saint Vincent and the Grenadines`,
    value: `saint_vincent_the_grenadines`,
  },
  { key: `Samoa`, value: `samoa` },
  { key: `San Marino`, value: `san_marino` },
  { key: `Sao Tome and Principe`, value: `sao_tome_principe` },
  { key: `Saudi Arabia`, value: `saudi_arabia` },
  { key: `Senegal`, value: `senegal` },
  { key: `Serbia`, value: `serbia` },
  { key: `Seychelles`, value: `seychelles` },
  { key: `Sierra Leone`, value: `sierra_leone` },
  { key: `Singapore`, value: `singapore` },
  { key: `Slovakia`, value: `slovakia` },
  { key: `Slovenia`, value: `slovenia` },
  { key: `Solomon Islands`, value: `solomon_islands` },
  { key: `Somalia`, value: `somalia` },
  { key: `South Africa`, value: `south_africa` },
  { key: `South Korea`, value: `south_korea` },
  { key: `South Sudan`, value: `south_sudan` },
  { key: `Spain`, value: `spain` },
  { key: `Sri Lanka`, value: `sri_lanka` },
  { key: `Sudan`, value: `sudan` },
  { key: `Suriname`, value: `suriname` },
  { key: `Sweden`, value: `sweden` },
  { key: `Switzerland`, value: `switzerland` },
  { key: `Syria`, value: `syria` },
  { key: `Taiwan`, value: `taiwan` },
  { key: `Tajikistan`, value: `tajikistan` },
  { key: `Tanzania`, value: `tanzania` },
  { key: `Thailand`, value: `thailand` },
  { key: `Timor-Leste`, value: `timor-leste` },
  { key: `Togo`, value: `togo` },
  { key: `Tonga`, value: `tonga` },
  { key: `Trinidad and Tobago`, value: `trinidad_tobago` },
  { key: `Tunisia`, value: `tunisia` },
  { key: `Turkey`, value: `turkey` },
  { key: `Turkmenistan`, value: `turkmenistan` },
  { key: `Tuvalu`, value: `tuvalu` },
  { key: `Uganda`, value: `uganda` },
  { key: `Ukraine`, value: `ukraine` },
  { key: `United Arab Emirates`, value: `uae` },
  { key: `United States of America`, value: `usa` },
  { key: `Uruguay`, value: `uruguay` },
  { key: `Uzbekistan`, value: `uzbekistan` },
  { key: `Vanuatu`, value: `vanuatu` },
  { key: `Vatican City`, value: `vatican_city` },
  { key: `Venezuela`, value: `venezuela` },
  { key: `Vietnam`, value: `vietnam` },
  { key: `Yemen`, value: `yemen` },
  { key: `Zambia`, value: `zambia` },
  { key: `Zimbabwe`, value: `zimbabwe` },
];

async function country(tp) {
  const country_obj = await tp.system.suggester(
    (item) => item.key,
    country_obj_arr,
    false,
    "Country?"
  );

  return country_obj;
}

module.exports = country;
```

### Templater

<!-- Add the full code as it should appear in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET COUNTRY
//---------------------------------------------------------
const country = await tp.user.country(tp);
const country_name = country.key;
const country_value = country.value;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

#### In Conjunction

<!-- Snippets used together with this snippet  -->

---

## Related

### Script Link

<!-- Link the user template script here -->

1. [[country.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[city suggester by country|City Suggester Filtered by Country]]
2. [[Country and City Suggester]]

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
