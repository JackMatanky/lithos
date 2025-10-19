// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> YAML FRONTMATTER FIELDS <<<<<
//-------------------------------------------------------------------
// File name
const file_name = "file.name";

// Tags
const tags = "file.etags AS Tags";

// YAML title
const yaml_title = "file.frontmatter.title";

// YAML alias
const yaml_alias = "file.frontmatter.aliases[0]";

// YAML file class
const yaml_class = "file.frontmatter.file_class";

// YAML type
const yaml_type = "file.frontmatter.type";

// YAML subtype
const yaml_subtype = "file.frontmatter.subtype";

// YAML Status
const yaml_status = "file.frontmatter.status";

// YAML PKM Category
const yaml_category = "file.frontmatter.category";

// YAML PKM Branch
const yaml_branch = "file.frontmatter.branch";

// YAML PKM Field
const yaml_field = "file.frontmatter.field";

// YAML PKM Subject
const yaml_subject = "file.frontmatter.subject";

// YAML PKM Topic
const yaml_topic = "file.frontmatter.topic";

// YAML PKM Topic
const yaml_subtopic = "file.frontmatter.subtopic";

// YAML URL
const yaml_url = "file.frontmatter.url";

// YAML About
const yaml_about = "file.frontmatter.about";

// File creation date
const date_created = "file.frontmatter.date_created";

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELD FUNCTIONS <<<<<
//-------------------------------------------------------------------
// Title link
const title_link = `link(file.link, ${yaml_alias}) AS Title`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${yaml_alias} + "]]" AS Title`;

// Status
const pkm_status = `default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
    }[x])(${yaml_status}), "🗄️Resource")
    AS Status`;

// File subtype
const pkm_type = `default(((x) => {
      "category": "🏘️Category",
      "branch": "🪑Branch",
      "field": "🚪Field",
      "subject": "🗝️Subject",
      "topic": "🧱Topic",
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
    }[x])(${yaml_type}), "🪟Definition")
    AS Type`;

const tree_type = `default(((x) => {
      "category": "🏘️Category",
      "branch": "🪑Branch",
      "field": "🚪Field",
      "subject": "🗝️Subject",
      "topic": "🧱Topic"
    }[x])(${yaml_type}), "🔩Subtopic")
    AS Type`;

const note_type = `default(((x) => {
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
    }[x])(${yaml_type}), "🪟Definition")
    AS Type`;

const pkm_content = `choice(!contains(
    ["evidence", "step", "conclusion", "summary", "proof"],
    ${yaml_type}),
      filter(split(${yaml_about}, "\\n"), (x) => regextest("\\w", x)),
      ${yaml_about}
    ) AS Content`;
const pkm_content_md = `regexreplace(regexreplace(choice(!contains(
    ["evidence", "step", "conclusion", "summary", "proof"],
    ${yaml_type}),
      filter(split(${yaml_about}, "\\n"), (x) => regextest("\\w", x)),
      ${yaml_about}
    ), "\\n", "<br>"), "<br>$", ""
    ) AS Content`;

const tree_content = `${yaml_about} AS Description`;
const tree_content_md = `regexreplace(
    regexreplace(${yaml_about}, "\\n", "<br>"),
    "<br>$", "")
    AS Description`;

const tree_context = `default(((x) => {
      "branch": ${yaml_category},
      "field": flat(list(${yaml_category}, ${yaml_branch})),
      "subject": flat(list(${yaml_category}, ${yaml_branch}, ${yaml_field})),
      "topic": flat(list(${yaml_category}, ${yaml_branch}, ${yaml_field}, ${yaml_subject})),
      "subtopic": flat(list(${yaml_category}, ${yaml_branch}, ${yaml_field}, ${yaml_subject}, ${yaml_topic}))
    }[x])(${yaml_type}), "")
    AS Context`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//-------------------------------------------------------------------
// PKM directory
const pkm_dir = `"70_pkm"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTER <<<<<
//-------------------------------------------------------------------
// File class filter
const class_filter = `contains(${yaml_class}, "pkm")`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//-------------------------------------------------------------------
const pkm_sort = `default(((x) => {
      "category": 1,
      "branch": 2,
      "field": 3,
      "subject": 4,
      "topic": 5,
      "subtopic": 6,
      "question": 7,
      "evidence": 8,
      "step": 9,
      "conclusion": 10,
      "theorem": 11,
      "proof": 12,
      "quote": 13,
      "idea": 14,
      "summary": 15,
      "concept": 16
    }[x])(${yaml_type}), 17)`;

const tree_type_sort = `default(((x) => {
      "category": 1,
      "branch": 2,
      "field": 3,
      "subject": 4,
      "topic": 5
    }[x])(${yaml_type}), 6)`;

const note_type_sort = `default(((x) => {
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
    }[x])(${yaml_type}), 11)`;

//-------------------------------------------------------------------
// DATAVIEW TABLE FOR NOTES WRITTEN BETWEEN DATES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR: TYPES: "tree", "zettel", "info", "permanent", "literature", "fleeting"
// VAR: STATUSES: "eve_review", "review", "clarify", "develop", "permanent", "resource"

async function dv_pkm_type_status_dates({
  type: type,
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  let data_field;
  if (type == "") {
    data_field = `${title_link},
    ${pkm_type},
    ${pkm_status},
    ${pkm_content}`;
  } else if (type.startsWith("tree")) {
    if (md == "true") {
      data_field = `${title_link},
    ${tree_type},
    ${tree_content_md},
    ${tree_context},
    ${tags}`;
    } else {
      data_field = `${title_link},
    ${tree_type},
    ${tree_content},
    ${tree_context},
    ${tags}`;
    }
  } else if (
    type.startsWith("perm") ||
    type.startsWith("lit") ||
    type.startsWith("fle") ||
    type.startsWith("info") ||
    type.startsWith("not_tree")
  ) {
    if (md == "true") {
      data_field = `${title_link},
    ${note_type},
    ${pkm_status},
    ${pkm_content_md},
    ${tags}`;
    } else {
      data_field = `${title_link},
    ${note_type},
    ${pkm_status},
    ${pkm_content},
    ${tags}`;
    }
  }

  let filter;
  let type_filter;
  // TYPE FILTER
  if (type == "tree") {
    type_filter = `contains(${yaml_class}, "tree")`;
  } else if (type == "not_tree") {
    type_filter = `!contains(${yaml_class}, "tree")`;
  } else if (type.startsWith("lit")) {
    type_filter = `contains(${yaml_class}, "zettel")
    AND filter(list("question", "evidence", "step", "conclusion", "theor", "proof"),
      (x) => contains(${yaml_type}, x))
    AND !contains(${yaml_status}, "perm")`;
  } else if (type.startsWith("fle")) {
    type_filter = `contains(${yaml_class}, "zettel")
    AND filter(list("quote", "idea", "summary"),
      (x) => contains(${yaml_type}, x))
    AND !contains(${yaml_status}, "perm")`;
  } else if (type.startsWith("info")) {
    type_filter = `contains(${yaml_class}, "info")
    AND filter(list("def", "conc", "gen"),
      (x) => contains(${yaml_type}, x))`;
  }
  let date_filter;
  // DATE FILTER
  if (date_end == "") {
    // SPECIFIC date
    date_filter = `contains(${date_created}, "${date_start}")`;
  } else {
    // Completed BETWEEN dates
    date_filter = `date(${date_created}) >= date(${date_start})
    AND date(${date_created}) <= date(${date_end})`;
  }

  if (date_start != "") {
    if (type != "") {
      filter = `${type_filter}`;
      if (status != "") {
        filter = `${type_filter}
    AND contains(${yaml_status}, "${status}")`;
      }
      filter = `${filter}
    AND ${date_filter}`;
    } else if (status == "eve_review") {
      filter = `${date_filter}`;
    } else if (status != "") {
      filter = `contains(${yaml_status}, "${status}")
    AND ${date_filter}`;
    }
  } else {
    if (type != "") {
      filter = `${type_filter}`;
      if (status != "") {
        filter = `${type_filter}
    AND contains(${yaml_status}, "${status}")`;
      }
    } else if (status != "") {
      filter = `contains(${yaml_status}, "${status}")`;
    }
  }

  let sort_field;
  if (date_end == "") {
    if (type == "") {
      sort_field = `${pkm_sort},
    ${yaml_title}`;
    } else if (type.startsWith("tree")) {
      sort_field = `${tree_type_sort},
    ${yaml_title}`;
    } else {
      sort_field = `${note_type_sort},
    ${yaml_title}`;
    }
  } else if (status != "") {
    if (type == "") {
      sort_field = `${date_created},
    ${pkm_sort},
    ${yaml_title}`;
    } else if (type.startsWith("tree")) {
      sort_field = `${date_created},
    ${tree_type_sort},
    ${yaml_title}`;
    } else {
      sort_field = `${date_created},
    ${note_type_sort},
    ${yaml_title}`;
    }
  }

  let dataview_query;
  dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${pkm_dir}
WHERE
    ${class_filter}
    AND ${filter}
SORT
    ${sort_field} ASC
${three_backtick}`;

  if (md == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    const md_query = String(
      dataview_query
        .replace(dataview_block_start_regex, "")
        .replace(dataview_block_end_regex, "")
        .replaceAll(/\n\s+/g, " ")
        .replaceAll(/\n/g, " ")
        .replace(title_link, md_title_link)
    );

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
  }

  return dataview_query;
}

module.exports = dv_pkm_type_status_dates;
