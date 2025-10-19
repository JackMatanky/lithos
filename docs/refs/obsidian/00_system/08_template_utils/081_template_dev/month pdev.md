```dataview
LIST
    rows.L.text
FROM
    "80_insight"
FLATTEN
    filter(file.lists, (x) => regextest(":.\w", x.text)) AS L
WHERE
    contains(file.frontmatter.type, "gratitude")
    AND contains(file.frontmatter.file_class, "pdev")
    AND contains(file.frontmatter.date_created, "2024-04-09")
GROUP BY
    link(
	    L.section,
	    dateformat(
		    date(
			    regexreplace(file.frontmatter.date, "^(\[\[)|(\]\])$", "")), "DDDD")
			    + " § "
			    + regexreplace(string(L.section), ".+>|]]$", ""))
SORT
    file.frontmatter.date,
    L.section ASC
```

```dataview
LIST
    rows.L.text
FROM
    "80_insight"
FLATTEN
    filter(file.lists, (x) => regextest(":.\w", x.text)) AS L
WHERE
    regextest("(1st)|(2nd)|(3rd)|(4th)|(5th)|(6th)|(7th)", regexreplace(string(L.section), "(.+\>\s)|(\]\]$)", ""))
    AND contains(file.frontmatter.type, "detachment")
    AND contains(file.frontmatter.file_class, "pdev")
    AND contains(file.frontmatter.date_created, "2024-04-09")
GROUP BY
    link(L.section, dateformat(date(regexreplace(file.frontmatter.date, "^(\[\[)|(\]\])$", "")), "DDDD") + " § " + regexreplace(string(L.section), ".+>|\]\]$", ""))
SORT
    file.frontmatter.date,
    L.section ASC
```

```dataview
LIST
    rows.week
FROM
    "80_insight"
FLATTEN
    list(sunday-best-experience, monday-best-experience, tuesday-best-experience, wednesday-best-experience, thursday-best-experience, friday-best-experience, saturday-best-experience, flat(list("> [!insight] Trends and Insight", weekly-best-experience))) AS week
WHERE
    contains(file.frontmatter.file_class, "pdev")
    AND contains(file.frontmatter.type, "reflection")
    AND contains(file.frontmatter.subtype, "weekly")
    AND date(eom) >= date(file.frontmatter.date_created)
    AND date(som) <= date(file.frontmatter.date_created)
    AND regextest(".", week)
GROUP BY
    link(file.link, regexreplace(file.name, "^(\d{4}).+(\d{2}).+", "Week $2, $1"))
SORT
    file.frontmatter.date ASC
```
