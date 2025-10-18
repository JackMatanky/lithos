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
const title_link = `link(file.link, ${alias})`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]"`;

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

//-------------------------------------------------------------------
// DATAVIEW TABLE FOR JOURNALS ON A SPECIFIC DATE
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

async function dv_pdev_date(date, md) {
	const md_arg = `${md}`;

	let dataview_query;
	dataview_query = `${dataview_block}
LIST WITHOUT ID
	${title_link}
FROM 
	${insight_dir}
WHERE 
	${class_filter}
	AND contains(${creation_date}, "${date}")
SORT
	${creation_date} ASC
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

module.exports = dv_pdev_date;
