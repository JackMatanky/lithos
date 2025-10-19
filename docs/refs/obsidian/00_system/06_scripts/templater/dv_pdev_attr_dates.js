// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//-------------------------------------------------------------------
// Title
const title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias}) AS Title`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]" AS Title`;

// Journal frontmatter date
const yaml_date = `file.frontmatter.date`;
const date = `${yaml_date} AS Date`;

// Journal creation date
const creation_date = `file.frontmatter.date_created`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCES <<<<<
//-------------------------------------------------------------------
// Insight directory
const insight_dir = `"80_insight"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//-------------------------------------------------------------------
// File class filter
const class_filter = `contains(file.frontmatter.file_class, "pdev")`;

// Detachment and achievement filters
const ordinal_section_filter = `regextest("(1st)|(2nd)|(3rd)|(4th)|(5th)|(6th)|(7th)", regexreplace(string(L.section), "(.+\\>\\s)|(\\]\\]$)", ""))`;

// Gratitude and self gratitude filter
const gratitude_filter = `regextest("(I)|(For)", regexreplace(string(L.section), "(.+\\>\\s)|(\\]\\]$)", ""))`;

// Limiting belief filter
const limiting_belief_filter = `regextest("Limiting", regexreplace(string(L.section), "(.+\\>\\s)|(\\]\\]$)", ""))`;

const list_item_filter = `filter(file.lists, (x) => regextest(":.\\w", x.text)`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA GROUPING <<<<<
//-------------------------------------------------------------------
// Full date
const date_full = `dateformat(date(regexreplace(${yaml_date}, "^(\\[\\[)|(\\]\\])$", "")), "DDDD")`;

// section group
const section_group = `link(L.section, ${date_full} + " § " + regexreplace(string(L.section), ".+>|\\]\\]$", ""))`;

// Journal date link
const date_link = `link(file.link, ${date_full})`;

//-------------------------------------------------------------------
// DATAVIEW TABLE FOR JOURNALS WRITTEN BETWEEN DATES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR: JOURNAL: "file",
// VAR: ATTR: "recount", "best-experience", "blindspot", "achievement",
// VAR: ATTR: "gratitude", "detachment", "limiting_belief", "lesson"

async function dv_pdev_attr_dates({
  attribute: attribute,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  const date_start_filter = `date(${creation_date}) >= date(${date_start})`;
  const date_end_filter = `date(${creation_date}) <= date(${date_end})`;
  let date_filter = `${date_start_filter}
    AND ${date_end_filter}`;
  if (date_end == "" || !date_end) {
    date_filter = `contains(${creation_date}, "${date_start}")`;
  }
  let dataview_query;

  if (attribute == "detachment") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    ${list_item_filter} AS L
WHERE
    ${ordinal_section_filter}
    AND contains(file.frontmatter.type, "${attribute}")
    AND ${class_filter}
    AND ${date_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else if (attribute == "achievement") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    ${list_item_filter} AS L
WHERE
    ${ordinal_section_filter}
    AND contains(file.frontmatter.type, "reflection")
    AND ${class_filter}
    AND ${date_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else if (attribute == "gratitude") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    ${list_item_filter} AS L
WHERE
    ${gratitude_filter}
    AND contains(file.frontmatter.type, "${attribute}")
    AND ${class_filter}
    AND ${date_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else if (attribute == "limiting_belief") {
    dataview_query = `${dataview_block}
LIST
    rows.L.text
FROM
    ${insight_dir}
FLATTEN
    ${list_item_filter} AS L
WHERE
    ${limiting_belief_filter}
    AND contains(file.frontmatter.type, "${attribute}")
    AND ${class_filter}
    AND ${date_filter}
GROUP BY
    ${section_group}
SORT
    ${yaml_date},
    L.section ASC
${three_backtick}`;
  } else {
    dataview_query = `${dataview_block}
LIST
    rows.D
FROM
    ${insight_dir}
FLATTEN
    ${attribute} AS D
WHERE
    ${class_filter}
    AND ${date_filter}
    AND regextest(".", D)
GROUP BY
    ${date_link}
SORT
    ${yaml_date} ASC
${three_backtick}`;
  }

  if (attribute == "file") {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${title_link},
    ${date}
FROM
    ${insight_dir}
WHERE
    ${class_filter}
    AND ${date_filter}
SORT
    ${yaml_date} ASC
${three_backtick}`;
  }

  let md_query = "null";
  if (md == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;
    md_query = String(
      dataview_query
        .replace(dataview_block_start_regex, "")
        .replace(dataview_block_end_regex, "")
        .replaceAll(/\n\s+/g, " ")
        .replaceAll(/\n/g, " ")
        .replace(title_link, md_title_link)
    );
    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
    // dataview_query = md_query;
  }

  return dataview_query;
}

module.exports = dv_pdev_attr_dates;
