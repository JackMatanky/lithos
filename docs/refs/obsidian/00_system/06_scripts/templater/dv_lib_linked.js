// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//-------------------------------------------------------------------
// Title
const yaml_title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias}) AS Title`;

// Title link for DV markdown query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]" AS Title`;

// File type
const yaml_type = `file.frontmatter.type`;
const file_type = `default(((x) => {
      "book": "📚Book",
      "book_chapter": "📑Book Chapter",
      "course": "🧑‍🏫Course",
      "course_lecture": "🧑‍🎓Course Lecture",
      "journal": "📜️Journal",
      "report": "📈Report",
      "news": "🗞️News",
      "magazine": "📰️Magazine",
      "webpage": "🌐Webpage",
      "blog": "💻Blog",
      "video": "🎥️Video",
      "youtube": "▶YouTube",
      "documentary": "🖼️Documentary",
      "audio": "🔉Audio",
      "podcast": "🎧️Podcast"
    }[x])(${yaml_type}), "📃Documentation")
    AS Type`;

// Status
const yaml_status = `file.frontmatter.status`;
const status = `default(((x) => {
      "undetermined": "❓Undetermined",
      "to_do": "🔜To do",
      "in_progress": "👟In progress",
      "done": "✔️Done",
      "resource": "🗃️Resource",
      "schedule": "📅Schedule"
    }[x])(${yaml_status}), "🤌On hold")
    AS Status`;

// Author
const yaml_author = "file.frontmatter.author";
const yaml_lecturer = "file.frontmatter.lecturer";
const author = `choice(length(choice(contains(${yaml_type}, "course"), ${yaml_lecturer}, ${yaml_author})) < 2,
      choice(contains(${yaml_type}, "course"), ${yaml_lecturer}[0], ${yaml_author}[0]),
      flat(choice(contains(${yaml_type}, "course"), ${yaml_lecturer}, ${yaml_author})))
    AS Creator`;

// Date published
const publish_date = `choice(contains(${yaml_type}, "book"),
      file.frontmatter.year_published,
      file.frontmatter.date_published)
    AS "Date Published"`;

// Tags
const tags = `file.etags AS Tags`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//-------------------------------------------------------------------
// Library directory
const lib_dir = `"60_library"`;

// Inbox directory
const inbox_dir = `"inbox"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTER <<<<<
//-------------------------------------------------------------------
// Current file filter
const current_file_filter = `file.name != this.file.name`;

// File inlinks and outlinks
const in_out_link_filter = `(contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))`;

// File class filter
const class_filter = `contains(file.frontmatter.file_class, "lib")`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//-------------------------------------------------------------------
const lib_status_sort = `default(((x) => {
      "done": 1,
      "in_progress": 2,
      "to_do": 3,
      "schedule": 4,
      "resource": 5,
      "on_hold": 6
    }[x])(${yaml_status}), 7)`;

//-------------------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE:
// VAR SUBTYPE:

async function dv_lib_linked(type, subtype, md) {
  const type_arg = `${type}`;
  const subtype_arg = `${subtype}`;
  const md_arg = `${md}`;

  const type_filter = `contains(file.frontmatter.file_class, "${type_arg}")`;
  let subtype_filter = `contains(${yaml_type}, "${subtype_arg}")`;

  let dataview_query;

  if (type_arg != "") {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${author},
	${publish_date},
	${file_type},
	${status},
	${tags}
FROM
	${lib_dir}
	OR ${inbox_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
SORT
	${yaml_type},
	${yaml_title} ASC
${three_backtick}`;
  } else {
    // Table for linked LIBRARY content
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${author},
	${publish_date},
	${file_type},
	${status},
	${tags}
FROM
	${lib_dir}
	OR ${inbox_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT
	${yaml_type},
	${yaml_title} ASC
${three_backtick}`;
  }

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

module.exports = dv_lib_linked;
