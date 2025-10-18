### Knowledge Tree

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    file.frontmatter.about AS Description,
    default(((x) => {
        "branch": file.frontmatter.category,
        "field": flat(list(file.frontmatter.category, file.frontmatter.branch)),
        "subject": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field)),
        "topic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject)),
        "subtopic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject, file.frontmatter.topic))
    }[x])(file.frontmatter.type), "")
    AS Context
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "pkm")
    AND contains(file.frontmatter.file_class, "tree")
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    file.frontmatter.title ASC
```

### Permanent

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
        "question": "❔Question",
        "evidence": "⚖️Evidence",
        "step": "🪜Step",
        "conclusion": "🎱Conclusion",
        "theorem": "🧮Theorem",
        "proof": "📃Proof",
        "quote": "⏺️Quote",
        "idea": "💭Idea",
        "summary": "📝Summary",
        "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
        "review": "📥Review",
        "clarify": "🌱Clarify",
        "develop": "🪴Develop",
        "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
    choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
        filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
        file.frontmatter.about
    ) AS Content,
    file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "pkm")
    AND contains(file.frontmatter.status, "perm")
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
        "question": 1,
        "evidence": 2,
        "step": 3,
        "conclusion": 4,
        "theorem": 5,
        "proof": 6,
        "quote": 7,
        "idea": 8,
        "summary": 9,
        "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Literature

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
        "question": "❔Question",
        "evidence": "⚖️Evidence",
        "step": "🪜Step",
        "conclusion": "🎱Conclusion",
        "theorem": "🧮Theorem",
        "proof": "📃Proof",
        "quote": "⏺️Quote",
        "idea": "💭Idea",
        "summary": "📝Summary",
        "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
        "review": "📥Review",
        "clarify": "🌱Clarify",
        "develop": "🪴Develop",
        "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
    choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
        filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
        file.frontmatter.about
    ) AS Content,
    file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "pkm")
    AND contains(file.frontmatter.file_class, "zettel")
    AND filter(list("question", "evidence", "step", "conclusion", "theor", "proof"),
        (x) => contains(file.frontmatter.type, x))
    AND !contains(file.frontmatter.status, "perm")
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
        "question": 1,
        "evidence": 2,
        "step": 3,
        "conclusion": 4,
        "quote": 5,
        "idea": 6,
        "summary": 7,
        "concept": 8
    }[x])(file.frontmatter.type), 9),
    file.frontmatter.title ASC
```

### Fleeting

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
        "question": "❔Question",
        "evidence": "⚖️Evidence",
        "step": "🪜Step",
        "conclusion": "🎱Conclusion",
        "theorem": "🧮Theorem",
        "proof": "📃Proof",
        "quote": "⏺️Quote",
        "idea": "💭Idea",
        "summary": "📝Summary",
        "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
        "review": "📥Review",
        "clarify": "🌱Clarify",
        "develop": "🪴Develop",
        "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
    choice(!contains(["evidence", "step", "conclusion", "summary"], file.frontmatter.type),
        filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
        file.frontmatter.about
    ) AS Content,
    file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "pkm")
    AND contains(file.frontmatter.file_class, "zettel")
    AND filter(list("quote", "idea", "summary"),
        (x) => contains(file.frontmatter.type, x))
    AND !contains(file.frontmatter.status, "perm")
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
        "question": 1,
        "evidence": 2,
        "step": 3,
        "conclusion": 4,
        "quote": 5,
        "idea": 6,
        "summary": 7,
        "concept": 8
    }[x])(file.frontmatter.type), 9),
    file.frontmatter.title ASC
```

### Info

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
        "question": "❔Question",
        "evidence": "⚖️Evidence",
        "step": "🪜Step",
        "conclusion": "🎱Conclusion",
        "theorem": "🧮Theorem",
        "proof": "📃Proof",
        "quote": "⏺️Quote",
        "idea": "💭Idea",
        "summary": "📝Summary",
        "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
    default(((x) => {
        "review": "📥Review",
        "clarify": "🌱Clarify",
        "develop": "🪴Develop",
        "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
    choice(!contains(
    ["evidence", "step", "conclusion", "summary"], file.frontmatter.type),
        filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
        file.frontmatter.about
    ) AS Content,
    file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "pkm")
    AND contains(file.frontmatter.file_class, "info")
    AND filter(list("def", "conc", "gen"),
        (x) => contains(file.frontmatter.type, x))
    AND (file.frontmatter.type = "undefined")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
        "question": 1,
        "evidence": 2,
        "step": 3,
        "conclusion": 4,
        "quote": 5,
        "idea": 6,
        "summary": 7,
        "concept": 8
    }[x])(file.frontmatter.type), 9),
	file.frontmatter.title ASC
```

---
