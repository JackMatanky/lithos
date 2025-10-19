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
    ["evidence", "step", "conclusion", "summary"],
    ${yaml_type}),
      filter(split(${yaml_about}, "\\n"), (x) => regextest("\\w", x)),
      ${yaml_about}
    ) AS Content`;
const pkm_content_md = `regexreplace(regexreplace(choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
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

// Current file filter
const current_file_filter = `file.name != this.file.name`;

// File outlink filter
const outlink_filter = `contains(file.outlinks, this.file.link)`;

// File inlink filter
const inlink_filter = `contains(file.inlinks, this.file.link)`;

// Tree child filter
const tree_child_filter = `(choice(contains(this.${yaml_type}, "category"), contains(${yaml_category}, this.file.name),
		choice(contains(this.${yaml_type}, "branch"), contains(${yaml_branch}, this.file.name),
		choice(contains(this.${yaml_type}, "field"), contains(${yaml_field}, this.file.name),
		choice(contains(this.${yaml_type}, "subject"),
			contains(${yaml_subject}, this.file.name),
			contains(${yaml_topic}, this.file.name))))))`;

const tree_sibling_filter = `(choice(contains(this.${yaml_type}, "subtopic"), contains(this.${yaml_topic}, ${yaml_topic}),
		choice(contains(this.${yaml_type}, "topic"), contains(this.${yaml_subject}, ${yaml_subject}),
		choice(contains(this.${yaml_type}, "subject"), contains(this.${yaml_field}, ${yaml_field}),
		choice(contains(this.${yaml_type}, "field"),
			contains(this.${yaml_branch}, ${yaml_branch}),
			contains(this.${yaml_category}, ${yaml_category}))))))`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//-------------------------------------------------------------------
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
// RELATED FILES DATAVIEW TABLES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE: "tree", "not_tree", "permanent", "literature", "fleeting", "info"
// VAR SUBTYPE: "category", "branch", "field", "subject", "topic", "subtopic", "qec_question", "qec_evidence", "qec_conclusion", "psa_problem", "psa_step", "psa_answer", "quote", "idea", "concept", "definition"
// VAR RELATION: "linked", "unrelated", "parent", "child", "child_b", "child_f", "child_subj", "child_t", "child_subt", "sibling"
// EXP: "linked" for linked all pkm external to their hierarchy;
// EXP: "in_link" for pkm with links to the file;
// EXP: "parent" for hierarchical anscestors
// EXP: "child" for direct descendents in hierarchy;
// EXP: "sibling" for pkm tree notes under the same tree object or qec and psa notes connected to each other in the relations callout.
// EXP: "unrelated" for linked notes without direct hierarchical relationships

async function dv_pkm_linked({
  type: type,
  subtype: subtype,
  relation: relation,
  md: md,
}) {
  let data_field;
  if (type.startsWith("tree")) {
    if (
      relation.startsWith("link") ||
      relation.startsWith("in_link") ||
      relation.startsWith("unrel") ||
      relation.startsWith("in_unrel")
    ) {
      if (subtype == "") {
        data_field = `${title_link},
    ${tree_type},
    ${tree_content},
    ${tree_context}`;
      } else {
        data_field = `${title_link},
    ${tree_content},
    ${tree_context}`;
      }
    } else if (
      relation.startsWith("par") ||
      relation.startsWith("in_par") ||
      relation.startsWith("child") ||
      relation.startsWith("in_child")
    ) {
      data_field = `${title_link},
    ${tree_content}`;
    } else if (relation.startsWith("sib") || relation.startsWith("in_sib")) {
      data_field = `${title_link},
    ${tree_type},
    ${tree_content},
    ${tree_context}`;
    }
  } else if (
    type.startsWith("perm") ||
    type.startsWith("lit") ||
    type.startsWith("fleet") ||
    type.startsWith("info") ||
    type.startsWith("not_tree") ||
    type.startsWith("zettel")
  ) {
    if (
      relation.startsWith("link") ||
      relation.startsWith("in_link") ||
      relation == ""
    ) {
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
  } else if (type.startsWith("perm")) {
    type_filter = `contains(${yaml_status}, "perm")`;
  }

  let relation_filter;
  if (relation.startsWith("link") || relation.startsWith("in_link")) {
    if (relation.startsWith("in_")) {
      relation_filter = `${outlink_filter}
    AND !${inlink_filter}`;
    } else {
      relation_filter = `(${outlink_filter}
    OR ${inlink_filter})`;
    }
    if (subtype != "") {
      type_filter = `${type_filter}
    AND (${yaml_type} = "${subtype}")`;
    }
  } else if (type.startsWith("tree")) {
    if (relation.startsWith("unrel") || relation.startsWith("in_unrel")) {
      relation_filter = `!filter(flat(list(
      this.${yaml_category},
      this.${yaml_branch},
      this.${yaml_field},
      this.${yaml_subject},
      this.${yaml_topic},
      this.${yaml_subtopic})),
        (x) => contains(x, file.name))`;
      if (relation.startsWith("in_")) {
        relation_filter = `${relation_filter}
    AND ${outlink_filter}
    AND !${inlink_filter}`;
      } else {
        relation_filter = `${relation_filter}
    AND (${outlink_filter}
    OR ${inlink_filter})`;
      }
    } else if (relation.startsWith("par") || relation.startsWith("in_par")) {
      if (subtype.startsWith("branch")) {
        // Table for a BRANCHES's parent CATEGORY
        relation_filter = `filter(this.${yaml_category},
      (x) => contains(x, file.name))`;
        type_filter = `${type_filter}
    AND (${yaml_type} = "category")`;
      } else if (subtype.startsWith("field")) {
        // Table for a FIELD's parent BRANCH and CATEGORY
        relation_filter = `filter(flat(list(
      this.${yaml_category},
      this.${yaml_branch})),
        (x) => contains(x, file.name))`;
        type_filter = `${type_filter}
    AND filter(list("category", "branch"),
      (x) => ${yaml_type} = x)`;
      } else if (subtype.startsWith("subject")) {
        // Table for a SUBJECT's parent FIELD, BRANCH, and CATEGORY
        relation_filter = `filter(flat(list(
      this.${yaml_category},
      this.${yaml_branch},
      this.${yaml_field})),
        (x) => contains(x, file.name))`;
        type_filter = `${type_filter}
    AND filter(list("category", "branch", "field"),
      (x) => ${yaml_type} = x)`;
      } else if (subtype.startsWith("topic")) {
        // Table for a TOPIC's parent SUBJECT, FIELD, BRANCH, and CATEGORY
        relation_filter = `filter(flat(list(
      this.${yaml_category},
      this.${yaml_branch},
      this.${yaml_field},
      this.${yaml_subject})),
        (x) => contains(x, file.name))`;
        type_filter = `${type_filter}
    AND filter(list("category", "branch", "field", "subject"),
      (x) => ${yaml_type} = x)`;
      } else if (subtype.startsWith("subtopic")) {
        // Table for a SUBTOPIC's parent TOPIC, SUBJECT, FIELD, BRANCH, and CATEGORY
        relation_filter = `filter(flat(list(
      this.${yaml_category},
      this.${yaml_branch},
      this.${yaml_field},
      this.${yaml_subject},
      this.${yaml_topic})),
        (x) => contains(x, file.name))`;
        type_filter = `${type_filter}
    AND filter(list("category", "branch", "field", "subject", "topic"),
      (x) => ${yaml_type} = x)`;
      }
      if (relation.startsWith("in_")) {
        relation_filter = `${relation_filter}
    AND ${outlink_filter}
    AND !${inlink_filter}`;
      }
    } else if (
      relation.startsWith("child") ||
      relation.startsWith("in_child")
    ) {
      if (subtype.startsWith("category")) {
        relation_filter = `contains(${yaml_category}, this.file.name)`;
      } else if (subtype.startsWith("branch")) {
        relation_filter = `contains(${yaml_branch}, this.file.name)`;
      } else if (subtype.startsWith("field")) {
        relation_filter = `contains(${yaml_field}, this.file.name)`;
      } else if (subtype.startsWith("subject")) {
        relation_filter = `contains(${yaml_subject}, this.file.name)`;
      } else if (subtype.startsWith("topic")) {
        relation_filter = `contains(${yaml_topic}, this.file.name)`;
      }
      if (relation.startsWith("child_b")) {
        // Table for a tree object's direct BRANCHES
        type_filter = `${type_filter}
    AND (${yaml_type} = "branch")`;
      } else if (relation.startsWith("child_f")) {
        // Table for a tree object's direct FIELDS
        type_filter = `${type_filter}
    AND (${yaml_type} = "field")`;
      } else if (relation.startsWith("child_subj")) {
        // Table for a tree object's direct SUBJECTs
        type_filter = `${type_filter}
    AND (${yaml_type} = "subject")`;
      } else if (relation.startsWith("child_t")) {
        // Table for a tree object's direct TOPICS
        type_filter = `${type_filter}
    AND (${yaml_type} = "topic")`;
      } else if (relation.startsWith("child_subt")) {
        // Table for a tree object's direct SUBTOPICS
        type_filter = `${type_filter}
    AND (${yaml_type} = "subtopic")`;
      }
      if (relation.startsWith("in_")) {
        relation_filter = `${relation_filter}
    AND ${outlink_filter}
    AND !${inlink_filter}`;
      }
    } else if (relation.startsWith("sibl") || relation.startsWith("in_sib")) {
      type_filter = `${type_filter}
    AND contains(${yaml_type}, "${subtype}")`;
      if (subtype.startsWith("branch")) {
        relation_filter = `!contains(this.${yaml_category}, "null")
    AND filter(this.${yaml_category},
      (x) => contains(${yaml_category}, x))`;
      } else if (subtype.startsWith("field")) {
        relation_filter = `!contains(this.${yaml_branch}, "null")
    AND filter(this.${yaml_branch},
      (x) => contains(${yaml_branch}, x))`;
      } else if (subtype.startsWith("subject")) {
        relation_filter = `!contains(this.${yaml_field}, "null")
    AND filter(this.${yaml_field},
      (x) => contains(${yaml_field}, x))`;
      } else if (subtype.startsWith("topic")) {
        relation_filter = `!contains(this.${yaml_subject}, "null")
    AND filter(this.${yaml_subject},
      (x) => contains(${yaml_subject}, x))`;
      } else if (subtype.startsWith("subtopic")) {
        relation_filter = `!contains(this.${yaml_topic}, "null")
    AND filter(this.${yaml_topic},
      (x) => contains(${yaml_topic}, x))`;
      }
      if (relation.startsWith("in_")) {
        relation_filter = `${relation_filter}
    AND ${outlink_filter}
    AND !${inlink_filter}`;
      }
    }
  }
  filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;

  let sort;
  if (type.startsWith("tree")) {
    if (
      subtype == "" ||
      relation.startsWith("par") ||
      relation.startsWith("in_par")
    ) {
      sort = `${tree_type_sort},
    ${yaml_title} ASC`;
    } else {
      sort = `${yaml_title} ASC`;
    }
  } else {
    sort = `${note_type_sort},
	${yaml_title} ASC`;
  }

  const dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${data_field}
FROM
    ${pkm_dir}
WHERE
    ${filter}
SORT
    ${sort}
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

module.exports = dv_pkm_linked;
