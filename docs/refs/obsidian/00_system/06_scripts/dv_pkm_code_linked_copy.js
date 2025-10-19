// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> YAML FRONTMATTER FIELDS <<<<<
//-------------------------------------------------------------------
// File name
const file_name = "file.name";

// Tags
const tags = "file.etags AS Tags";

// YAML Frontmatter
const yaml_front = (value) => yaml_front("") + value;

// YAML title
const yaml_title = yaml_front("title");

// YAML alias
const yaml_alias = yaml_front("aliases[0]");

// YAML file class
const yaml_class = yaml_front("file_class");

// YAML type
const yaml_type = yaml_front("type");

// File subtype
const yaml_subtype = yaml_front("subtype");

// YAML Status
const yaml_status = yaml_front("status");

// YAML PKM Category
const yaml_category = yaml_front("category");

// YAML PKM Branch
const yaml_branch = yaml_front("branch");

// YAML PKM Field
const yaml_field = yaml_front("field");

// YAML PKM Subject
const yaml_subject = yaml_front("subject");

// YAML PKM Topic
const yaml_topic = yaml_front("topic");

// YAML PKM Topic
const yaml_subtopic = yaml_front("subtopic");

// YAML Syntax
const yaml_syntax = yaml_front("syntax");

// YAML URL
const yaml_url = yaml_front("url");

// YAML About
const yaml_about = yaml_front("about");

// File creation date
const date_created = yaml_front("date_created");

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELD FUNCTIONS <<<<<
//-------------------------------------------------------------------
// Title link
const title_link = `link(file.link, ${yaml_alias}) AS Title`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${yaml_alias} + "]]" AS Title`;

// Code language (topic[0])
const code_lang = `${yaml_topic}[0] AS Language`;

// Code sublanguage, library, or module (subtopic[0])
const code_sublang = `choice(!contains(${yaml_subtopic}[0], "null"),
    flat(${yaml_subtopic}), "") AS Sublanguage`;

// Code language and sublanguage
const code_lang_sub = `choice(!contains(${yaml_subtopic}[0], "null"),
      flat(list(("**Language**: " + ${yaml_topic}[0]), ("**Sublanguage**: " + join(${yaml_subtopic})))),
      ${yaml_topic}[0]
    ) AS Language`;

// File type and subtype
const code_type = `choice(${yaml_subtype} = "regex", "RegEx",
    choice(${yaml_subtype} = "pass_through_rawsql", "Pass-Through RAWSQL",
    choice(${yaml_subtype} != "null",
    join(map(split(${yaml_subtype}, "_"),
      (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " "), "")))
    + " " +
    join(map(split(${yaml_type}, "_"),
      (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ")
    AS Type`;

// File subtype
const code_subtype = `choice(${yaml_subtype} = "regex", "RegEx",
    choice(${yaml_subtype} = "pass_through_rawsql", "Pass-Through RAWSQL",
    choice(${yaml_subtype} != "null",
    join(map(split(${yaml_subtype}, "_"),
      (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " "), "")))
    AS Subtype`;

// Status
const pkm_status = `choice(${yaml_status} = "schedule", "🤷Unknown",
    choice(${yaml_status} = "review", "🔜Review",
    choice(${yaml_status} = "clarify", "🌱Clarify",
    choice(${yaml_status} = "develop", "🪴Develop",
    choice(${yaml_status} = "done", "🌳Done", "🗄️Resource")))))
	  AS Status`;

const code_content = `${yaml_about} AS Description`;
const code_content_md = `regexreplace(${yaml_about}, "\\n", "<br>") AS Description`;

const tree_context = `choice(${yaml_subtype} = "subtopic", flat(list(${yaml_category}, ${yaml_branch}, ${yaml_field}, ${yaml_subject}, ${yaml_topic})),
	  choice(${yaml_subtype} = "topic", flat(list(${yaml_category}, ${yaml_branch}, ${yaml_field}, ${yaml_subject})),
	  choice(${yaml_subtype} = "subject", flat(list(${yaml_category}, ${yaml_branch}, ${yaml_field})),
	  choice(${yaml_subtype} = "field", flat(list(${yaml_category}, ${yaml_branch})),
	  choice(${yaml_subtype} = "branch", ${yaml_category}, "")))))
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
const class_filter = `contains(${yaml_class}, "pkm_code")`;

const yaml_index_filter = (pkm) => `contains(${pkm}, this.${pkm}[0])`;
const yaml_filter = (pkm) => `contains(${pkm}, this.${pkm})`;

// Current file filter
const current_file_filter = `file.name != this.file.name`;

// File outlink filter
const outlink_filter = `contains(file.outlinks, this.file.link)`;

// File inlink filter
const inlink_filter = `contains(file.inlinks, this.file.link)`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//-------------------------------------------------------------------

//-------------------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE: "snippet", "data_type", "error", "function", "keyword", "method", "operator", "statement"
// VAR RELATION: "linked", "lang", "sublang", "cousin", "sibling"
// EXP: "linked" for all linked pkm code files;
// EXP: "lang" for same language (topic[0])
// EXP: "lang_ex" for same language (topic[0]), different sublanguage, type, and subtype
// EXP: "sub" for same language and sublanguage (subtopic[0])
// EXP: "sub_ex" for same language and sublanguage (subtopic[0]), different type and subtype
// EXP: "type" for same type
// EXP: "type_ex" for same type, different subtype
// EXP: "subtype" for same subtype
// EXP: add "lang_" prefix for same language
// EXP: add "sublang_" prefix for same sublanguage
// EXP: add "type_" prefix for same language, sublanguage, and type
// EXP: add "_ex" suffix for excluding lower relations (1:lang, 2:type, 3:sub)
// EXP: add "in_" prefix to exclude outlinks

async function dv_pkm_code_linked_copy({ type: type, relation: relation, md: md }) {
  const type_arg = `${type}`;
  const relation_arg = `${relation}`;
  const md_arg = `${md}`;

  let data_field;
  let data_field_high;
  const data_field_low = `${code_content},
	  ${tags}`;
  if (relation_arg.startsWith("lang") || relation_arg.startsWith("in_lang")) {
    data_field_high = `${title_link},
    ${code_sublang}`;
  } else if (
    relation_arg.startsWith("sublang") ||
    relation_arg.startsWith("in_sublang")
  ) {
    data_field_high = `${title_link}`;
  } else if (
    relation_arg.startsWith("type") ||
    relation_arg.startsWith("in_type")
  ) {
    data_field_high = `${title_link},
    ${code_lang_sub}`;
  } else if (
    relation_arg.startsWith("link") ||
    relation_arg.startsWith("in_link")
  ) {
    data_field_high = `${title_link},
    ${code_lang_sub},
    ${code_type}`;
  }

  if (type_arg.startsWith("snip")) {
    data_field = `${data_field_high},
	  ${data_field_low}`;
  } else if (!type_arg.startsWith("snip")) {
    // Data fields for data_type, error, function, keyword, method, operator, statement
    if (relation_arg.endsWith("subtype")) {
      // Data fields for same subtype
      data_field = `${data_field_high},
	  ${data_field_low}`;
    } else if (
      relation_arg.endsWith("type") ||
      relation_arg.endsWith("type_ex")
    ) {
      // Data fields for same type
      data_field = `${data_field_high},
    ${code_subtype},
	  ${data_field_low}`;
    } else if (
      relation_arg.endsWith("lang") ||
      relation_arg.endsWith("lang_ex")
    ) {
      // Data fields for same language and/or sublanguage
      if (
        relation_arg.startsWith("link") ||
        relation_arg.startsWith("in_link")
      ) {
        data_field = `${data_field_high},
	  ${data_field_low}`;
      } else {
        data_field = `${data_field_high},
    ${code_type},
	  ${data_field_low}`;
      }
    } else if (
      relation_arg.startsWith("link") ||
      relation_arg.startsWith("in_link")
    ) {
      // Data fields for linked code files
      data_field = `${data_field_high},
	  ${data_field_low}`;
    }
  }

  let filter;
  let type_filter;
  let relation_filter;
  if (relation_arg.startsWith("in_link")) {
    relation_filter = `${outlink_filter}
    AND !${inlink_filter}`;
    if (relation_arg.endsWith("lang_ex")) {
      relation_filter = `!${yaml_index_filter(yaml_topic)}
    AND ${relation_filter}`;
    } else if (relation_arg.endsWith("lang")) {
      relation_filter = `${yaml_index_filter(yaml_topic)}
    AND ${relation_filter}`;
    }
  } else if (relation_arg.startsWith("link")) {
    relation_filter = `(${outlink_filter}
    OR ${inlink_filter})`;
    if (relation_arg.endsWith("lang_ex")) {
      relation_filter = `!${yaml_index_filter(yaml_topic)}
    AND ${relation_filter}`;
    } else if (relation_arg.endsWith("lang")) {
      relation_filter = `${yaml_index_filter(yaml_topic)}
    AND ${relation_filter}`;
    }
  } else if (
    relation_arg.startsWith("lang") ||
    relation_arg.startsWith("in_lang")
  ) {
    relation_filter = `${yaml_index_filter(yaml_topic)}`;
    if (relation_arg.endsWith("subtype") && !type_arg.startsWith("snip")) {
      relation_filter = `${relation_filter}
    AND ${yaml_index_filter(yaml_subtopic)}
    AND ${yaml_filter(yaml_type)}
    AND ${yaml_filter(yaml_subtype)}`;
    } else if (relation_arg.endsWith("type")) {
      relation_filter = `${relation_filter}
    AND ${yaml_index_filter(yaml_subtopic)}
    AND ${yaml_filter(yaml_type)}`;
    } else if (relation_arg.endsWith("type_ex")) {
      relation_filter = `${relation_filter}
    AND ${yaml_index_filter(yaml_subtopic)}
    AND ${yaml_filter(yaml_type)}
    AND !${yaml_filter(yaml_subtype)}
    AND regextest("\\w", ${yaml_subtype})`;
    } else if (relation_arg.endsWith("sublang")) {
      relation_filter = `${relation_filter}
    AND ${yaml_index_filter(yaml_subtopic)}`;
    } else if (relation_arg.endsWith("sublang_ex")) {
      relation_filter = `${relation_filter}
    AND ${yaml_index_filter(yaml_subtopic)}
    AND !${yaml_filter(yaml_type)}
    AND regextest("\\w", ${yaml_type})
    AND !${yaml_filter(yaml_subtype)}
    AND regextest("\\w", ${yaml_subtype})`;
    } else if (relation_arg.endsWith("_ex")) {
      relation_filter = `${relation_filter}
    AND !${yaml_index_filter(yaml_subtopic)}
    AND !contains(${yaml_subtopic}[0], "null")
    AND !${yaml_filter(yaml_type)}
    AND regextest("\\w", ${yaml_type})
    AND !${yaml_filter(yaml_subtype)}
    AND regextest("\\w", ${yaml_subtype})`;
    }
  } else if (
    relation_arg.startsWith("sublang") ||
    relation_arg.startsWith("in_sublang")
  ) {
    relation_filter = `${yaml_index_filter(yaml_subtopic)}`;
    if (relation_arg.endsWith("subtype") && !type_arg.startsWith("snip")) {
      relation_filter = `${relation_filter}
    AND ${yaml_filter(yaml_type)}
    AND ${yaml_filter(yaml_subtype)}`;
    } else if (relation_arg.endsWith("type")) {
      relation_filter = `${relation_filter}
    AND ${yaml_filter(yaml_type)}`;
    } else if (relation_arg.endsWith("type_ex")) {
      relation_filter = `${relation_filter}
    AND ${yaml_filter(yaml_type)}
    AND !${yaml_filter(yaml_subtype)}
    AND regextest("\\w", ${yaml_subtype})`;
    } else if (relation_arg.endsWith("_ex")) {
      relation_filter = `${relation_filter}
    AND !${yaml_filter(yaml_type)}
    AND regextest("\\w", ${yaml_type})
    AND !${yaml_filter(yaml_subtype)}
    AND regextest("\\w", ${yaml_subtype})`;
    }
  } else if (
    relation_arg.startsWith("type") ||
    relation_arg.startsWith("in_type")
  ) {
    relation_filter = `${yaml_filter(yaml_type)}`;
    if (relation_arg.endsWith("subtype") && !type_arg.startsWith("snip")) {
      relation_filter = `${relation_filter}
    AND ${yaml_filter(yaml_subtype)}`;
    } else if (relation_arg.endsWith("ex")) {
      relation_filter = `${relation_filter}
    AND !${yaml_filter(yaml_subtype)}`;
    } else if (relation_arg.endsWith("type")) {
      relation_filter = `${relation_filter}
    AND !${yaml_index_filter(yaml_topic)}
    AND !contains(${yaml_topic}[0], "null")
    AND !${yaml_index_filter(yaml_subtopic)}
    AND !contains(${yaml_subtopic}[0], "null")
    AND !${yaml_filter(yaml_subtype)}
    AND regextest("\\w", ${yaml_subtype})`;
    }
  } else if (
    relation_arg.startsWith("subtype") ||
    relation_arg.startsWith("in_subtype")
  ) {
    relation_filter = `${yaml_filter(yaml_subtype)}`;
    if (relation_arg.endsWith("subtype") && !type_arg.startsWith("snip")) {
      relation_filter = `${relation_filter}
    AND !${yaml_index_filter(yaml_topic)}
    AND !contains(${yaml_topic}[0], "null")
    AND !${yaml_index_filter(yaml_subtopic)}
    AND !contains(${yaml_subtopic}[0], "null")
    AND !${yaml_filter(yaml_type)}
    AND regextest("\\w", ${yaml_type})`;
    }
  }

  if (relation_arg.startsWith("in_")) {
    relation_filter = `${relation_filter}
    AND !${inlink_filter}`;
  }

  if (type_arg.startsWith("snip")) {
    type_filter = `contains(${yaml_type}, "${type_arg}")`;
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  } else if (type_arg.startsWith("not_snip")) {
    type_filter = `!contains(${yaml_type}, "snip")`;
    filter = `${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${relation_filter}`;
  }

  let sort;
  if (type_arg.startsWith("snip")) {
    if (relation_arg.startsWith("link") || relation_arg.startsWith("in_link")) {
      sort = `${yaml_topic}[0],
    ${yaml_subtopic}[0],
    ${yaml_title} ASC`;
    } else if (
      relation_arg.startsWith("lang") ||
      relation_arg.startsWith("in_lang")
    ) {
      sort = `${yaml_subtopic}[0],
    ${yaml_title} ASC`;
    } else {
      sort = `${yaml_title} ASC`;
    }
  } else {
    if (relation_arg.startsWith("link") || relation_arg.startsWith("in_link")) {
      sort = `${yaml_topic}[0],
    ${yaml_subtopic}[0],
    ${yaml_type},
    ${yaml_subtype},
    ${yaml_title} ASC`;
    } else if (
      relation_arg.startsWith("lang") ||
      relation_arg.startsWith("in_lang")
    ) {
      sort = `${yaml_subtopic}[0],
    ${yaml_type},
    ${yaml_subtype},
    ${yaml_title} ASC`;
    } else if (
      relation_arg.startsWith("sublang") ||
      relation_arg.startsWith("in_sublang")
    ) {
      sort = `${yaml_type},
    ${yaml_subtype},
    ${yaml_title} ASC`;
    } else if (
      relation_arg.startsWith("type") ||
      relation_arg.startsWith("in_type")
    ) {
      sort = `${yaml_subtype},
    ${yaml_title} ASC`;
    } else {
      sort = `${yaml_title} ASC`;
    }
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

  if (md_arg == "true") {
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

module.exports = dv_pkm_code_linked_copy;
