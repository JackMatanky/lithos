// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> YAML FRONTMATTER FIELDS <<<<<
//-------------------------------------------------------------------
const metadata = {
  file_name: "file.name",
  tags: "file.etags AS Tags",
  title: "file.frontmatter.title",
  alias: "file.frontmatter.aliases[0]",
  class: "file.frontmatter.file_class",
  type: "file.frontmatter.type",
  subtype: "file.frontmatter.subtype",
  status: "file.frontmatter.status",
  category: "file.frontmatter.category",
  branch: "file.frontmatter.branch",
  field: "file.frontmatter.field",
  subject: "file.frontmatter.subject",
  topic: "file.frontmatter.topic",
  subtopic: "file.frontmatter.subtopic",
  syntax: "file.frontmatter.syntax",
  url: "file.frontmatter.url",
  about: "file.frontmatter.about",
  date_created: "file.frontmatter.date_created",
};

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELD FUNCTIONS <<<<<
//-------------------------------------------------------------------
const data_fields = {
  title_link: `link(file.link, ${metadata.alias})`,
  md_title_link: `"[[" + ${metadata.file_name} + "\|" + ${metadata.alias} "]]"`,
  code_lang: `${metadata.topic}[0] AS Language`,
  code_sublang: `choice(!contains(${metadata.subtopic}[0], "null"), flat(${metadata.subtopic}), "") AS Sublanguage`,
  code_lang_sub: `choice(!contains(${metadata.subtopic}[0], "null"), flat(list(("**Language**: " + ${metadata.topic}[0]), ("**Sublanguage**: " + join(${metadata.subtopic})))), ${metadata.topic}[0]) AS Language`,
  code_type: `choice(${metadata.subtype} = "regex", "RegEx", choice(${metadata.subtype} = "pass_through_rawsql", "Pass-Through RAWSQL", choice(${metadata.subtype} != "null", join(map(split(${metadata.subtype}, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " "), ""))) + " " + join(map(split(${metadata.type}, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " ") AS Type`,
  code_subtype: `choice(${metadata.subtype} = "regex", "RegEx", choice(${metadata.subtype} = "pass_through_rawsql", "Pass-Through RAWSQL", choice(${metadata.subtype} != "null", join(map(split(${metadata.subtype}, "_"), (x) => upper(substring(x, 0, 1)) + substring(x, 1)), " "), ""))) AS Subtype`,
  pkm_status: `choice(${metadata.status} = "schedule", "🤷Unknown", choice(${metadata.status} = "review", "🔜Review", choice(${metadata.status} = "clarify", "🌱Clarify", choice(${metadata.status} = "develop", "🪴Develop", choice(${metadata.status} = "done", "🌳Done", "🗄️Resource"))))) AS Status`,
  code_content: metadata.about,
  code_content_md: `regexreplace(${metadata.about}, "\\n", "<br>")`,
};

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//-------------------------------------------------------------------
const data_source = {
  pkm_dir: "70_pkm",
};

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//-------------------------------------------------------------------
const data_filters = {
  class_filter: `contains(${metadata.class}, "pkm_code")`,
  current_file_filter: `file.name != this.file.name`,
  outlink_filter: `contains(file.outlinks, this.file.link)`,
  inlink_filter: `contains(file.inlinks, this.file.link)`,
};

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
// Placeholder for future data sorting logic.
//-------------------------------------------------------------------

//-------------------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//-------------------------------------------------------------------
const unicode_backtick = String.fromCodePoint(0x60); // Unicode backtick character
const three_backtick = unicode_backtick.repeat(3); // Triple backtick for code blocks
const dataview_block = `${three_backtick}dataview`; // Dataview block code

/**
 * Function to generate Dataview tables for linked PKM code files.
 * @param {Object} options - Options for the Dataview table.
 * @param {string} options.type - Type of code file (e.g., snippet).
 * @param {string} options.relation - Relation type (e.g., link_type_subtype).
 * @param {string} options.md - Markdown formatting option.
 */

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

async function dv_pkm_code_linked({ type: type, relation: relation, md: md }) {
  const type_arg = `${type}`;
  const relation_arg = `${relation}`;
  const md_arg = `${md}`;

  const title_link =
    (md_arg === "true" ? data_fields.md_title_link : data_fields.title_link) +
    " AS Title";
  const code_content =
    (md_arg === "true"
      ? data_fields.md_code_content
      : data_fields.code_content) + " AS Description";

  let data_field_high;
  const data_field_low = `${code_content}, ${metadata.tags}`;

  // Determine high-level data fields based on relation type
  if (relation_arg.startsWith("lang") || relation_arg.startsWith("in_lang")) {
    data_field_high = `${title_link}, ${data_fields.code_sublang}`;
  } else if (
    relation_arg.startsWith("sublang") ||
    relation_arg.startsWith("in_sublang")
  ) {
    data_field_high = title_link;
  } else {
    data_field_high = `${title_link}, ${data_fields.code_lang_sub}, ${data_fields.code_type}`;
  }

  // Determine overall data fields based on type and relation
  let data_field;
  if (
    type_arg.startsWith("snip") ||
    relation_arg.endsWith("subtype") ||
    relation_arg.endsWith("type") ||
    relation_arg.endsWith("lang") ||
    relation_arg.startsWith("link")
  ) {
    data_field = `${data_field_high}, ${data_field_low}`;
  } else {
    data_field = `${data_field_high}, ${data_fields.code_type}, ${data_field_low}`;
  }

  // Set filters based on relation and type
  let relation_filter;

  if (relation_arg.startsWith("in_link")) {
    relation_filter = `${data_filters.outlink_filter} AND !${data_filters.inlink_filter}`;
  } else if (relation_arg.startsWith("link")) {
    relation_filter = `(${data_filters.outlink_filter} OR ${data_filters.inlink_filter})`;
  } else if (
    relation_arg.startsWith("sublang") ||
    relation_arg.startsWith("in_sublang")
  ) {
    relation_filter = `contains(${metadata.subtopic}, this.${metadata.subtopic}[0])`;
  } else if (
    relation_arg.startsWith("type") ||
    relation_arg.startsWith("in_type")
  ) {
    relation_filter = `contains(${metadata.type}, this.${metadata.type})`;
  }

  if (relation_arg.endsWith("lang_ex")) {
    relation_filter = `!contains(${metadata.topic}, this.${metadata.topic}[0]) AND ${relation_filter}`;
  } else if (relation_arg.endsWith("lang")) {
    relation_filter = `contains(${metadata.topic}, this.${metadata.topic}[0]) AND ${relation_filter}`;
  } else if (relation_arg.endsWith("type_ex")) {
    relation_filter = `${relation_filter} AND !contains(${metadata.subtopic}, this.${metadata.subtopic}[0]) AND !contains(${metadata.type}, this.${metadata.type})`;
  }

  const type_filter = type_arg.startsWith("snip")
    ? `contains(${metadata.type}, "snippet")`
    : `!contains(${metadata.type}, "snippet")`;

  const filter = [
    data_filters.class_filter,
    data_filters.current_file_filter,
    relation_filter,
    type_filter,
  ].join(" AND ");

  // Determine table title based on relation argument
  const title =
    relation_arg.startsWith("link") || relation_arg.startsWith("in_link")
      ? "Linked Code Files"
      : "Related Code Files";

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

  if (md_arg === "true") {
    const dataview_block_start_regex = /^dataview\n/g;
    const dataview_block_end_regex = /\n$/g;

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

module.exports = dv_pkm_code_linked;
