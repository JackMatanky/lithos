// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//-------------------------------------------------------------------
// SECT: >>>>> GENERAL FIELDS <<<<<
// File title
const yaml_title = `file.frontmatter.title`;

// Title link
const title_link = `link(file.link, file.frontmatter.aliases[0]) AS Title`;

// File type
const yaml_type = `file.frontmatter.type`;

// File subtype
const yaml_subtype = `file.frontmatter.subtype`;

// Status
const yaml_status = `file.frontmatter.status`;

// Tags
const tags = `file.etags AS Tags`;

// SECT: >>>>> LIBRARY <<<<<
// Library file type
const lib_file_type = `choice(${yaml_type} = "book", "📚Book", 
	choice(${yaml_type} = "book_chapter", "📑Book Chapter", 
	choice(${yaml_type} = "journal", "📜️Journal", 
	choice(${yaml_type} = "report", "📈Report", 
	choice(${yaml_type} = "news", "🗞️News", 
	choice(${yaml_type} = "magazine", "📰️Magazine", 
	choice(${yaml_type} = "webpage", "🌐Webpage", 
	choice(${yaml_type} = "blog", "💻Blog", 
	choice(${yaml_type} = "video", "🎥️Video", 
	choice(${yaml_type} = "youtube", "▶YouTube", 
	choice(${yaml_type} = "documentary", "🖼️Documentary", 
	choice(${yaml_type} = "audio", "🔉Audio", 
	choice(${yaml_type} = "podcast", "🎧️Podcast", "📃Documentation")))))))))))))
	AS Type`;

// Library status
const lib_status = `choice(${yaml_status} = "undetermined", "❓Undetermined",
	choice(${yaml_status} = "to_do", "🔜To do",
	choice(${yaml_status} = "in_progress", "👟In progress",
	choice(${yaml_status} = "done", "✔️Done",
	choice(${yaml_status} = "resource", "🗃️Resource",
	choice(${yaml_status} = "schedule", "📅Schedule", "🤌On hold"))))))
	AS Status`;

// SECT: >>>>> PKM <<<<<
// PKM Subtype
const pkm_subtype = `choice(contains(${yaml_subtype}, "category"), "🏘️Category",
	choice(contains(${yaml_subtype}, "branch"), "🪑Branch",
	choice(contains(${yaml_subtype}, "field"), "🚪Field",
	choice(contains(${yaml_subtype}, "subject"), "🗝️Subject",
	choice(contains(${yaml_subtype}, "topic"), "🧱Topic", 
	choice(contains(${yaml_subtype}, "subtopic"), "🔩Subtopic"
	choice(contains(${yaml_subtype}, "question"), "❔Question",
	choice(contains(${yaml_subtype}, "evidence"), "⚖️Evidence",
	choice(contains(${yaml_subtype}, "conclusion"), "💡Conclusion",
	choice(contains(${yaml_subtype}, "problem"), "🪨Problem",
	choice(contains(${yaml_subtype}, "step"), "🪜Step",
	choice(contains(${yaml_subtype}, "answer"), "🎱Answer",
	choice(contains(${yaml_subtype}, "quote"), "🎤Quote",
	choice(contains(${yaml_subtype}, "idea"), "☁️Idea",
	choice(contains(${yaml_subtype}, "concept"), "🎞️Concept", "🪟Definition")))))))))))))))
	AS Subtype`;

// PKM status
const pkm_status = `choice(${yaml_status} = "review", "🌱️Review",
	choice(${yaml_status} = "clarify", "🌿️Clarify",
	choice(${yaml_status} = "develop", "🪴Develop",
	choice(${yaml_status} = "evergreen", "🌳Evergreen", "🗃️Resource"))))
	AS Status`;

const pkm_content = `choice(${yaml_subtype} = "qec_question", list(Context, Question),
	choice(${yaml_subtype} = "qec_evidence", Evidence,
	choice(${yaml_subtype} = "qec_conclusion", Conclusion,
	choice(${yaml_subtype} = "psa_problem", list(Context, Problem),
	choice(${yaml_subtype} = "psa_step", Step,
	choice(${yaml_subtype} = "psa_answer", Answer,
	choice(${yaml_subtype} = "quote", Quote,
	choice(${yaml_subtype} = "idea", Idea,
	choice(${yaml_subtype} = "definition", Definition, Description)))))))))
	AS Content`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//-------------------------------------------------------------------
// Template directory
const template_dir = `-"00_system/05_templates"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//-------------------------------------------------------------------
const pkm_sort = `choice(${yaml_subtype} = "category", 1,
	choice(${yaml_subtype} = "branch", 2,
	choice(${yaml_subtype} = "field", 3,
	choice(${yaml_subtype} = "subject", 4,
	choice(${yaml_subtype} = "topic", 5, 
	choice(${yaml_subtype} = "subtopic", 6,
	choice(${yaml_subtype} = "qec_question", 7,
	choice(${yaml_subtype} = "qec_evidence", 8,
	choice(${yaml_subtype} = "qec_conclusion", 9,
	choice(${yaml_subtype} = "psa_problem", 10,
	choice(${yaml_subtype} = "psa_step", 11,
	choice(${yaml_subtype} = "psa_answer", 12,
	choice(${yaml_subtype} = "quote", 13,
	choice(${yaml_subtype} = "idea", 14,
	choice(${yaml_subtype} = "concept", 15, 16)))))))))))))))`;

//-------------------------------------------------------------------
// DAILY NOTE DATAVIEW TABLES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"

async function dv_day_class_type_file({
  file_class: file_class,
  type: type,
  date: date,
  md: md,
}) {
  const class_arg = `${file_class}`;
  const type_arg = `${type}`;
  const md_arg = `${md}`;

  const date_filter = `contains(file.frontmatter.date_created, "${date}")`;
  const class_filter = `contains(file.frontmatter.file_class, "${class_arg}")`;
  const type_filter = `contains(${yaml_type}, "${type_arg}")`;

  let dataview_query;
  if (class_arg.startsWith("pkm")) {
    if (type_arg == "") {
      dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${pkm_subtype},
	${pkm_status},
	${pkm_content},
	${tags}
FROM
	${template_dir}
WHERE
	${class_filter}
	AND ${date_filter}
SORT
	${pkm_sort}
	file.frontmatter.date_created ASC
${three_backtick}`;
    } else if (type_arg.startsWith("info")) {
      dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${pkm_subtype},
	${pkm_status},
	${tags}
FROM
	${template_dir}
WHERE
	${class_filter}
	AND ${type_filter}
	AND ${date_filter}
SORT
	${yaml_subtype},
	${yaml_title},
	file.frontmatter.date_created ASC
${three_backtick}`;
    } else {
      dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${pkm_subtype},
	${pkm_status},
	${tags}
FROM
	${template_dir}
WHERE
	${class_filter}
	AND ${type_filter}
	AND ${date_filter}
SORT
	${yaml_title},
	file.frontmatter.date_created ASC
${three_backtick}`;
    }
  } else if (class_arg.startsWith("lib")) {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${lib_file_type},
	${lib_status},
	${tags}
FROM
	${template_dir}
WHERE
	contains(file.frontmatter.file_class, "${class_arg}")
	AND contains(file.frontmatter.file_class, "${type_arg}")
	AND contains(file.frontmatter.date_created, "${date}")
SORT
	file.frontmatter.date_created ASC
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
    );

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
  }
  return dataview_query;
}

module.exports = dv_day_class_type_file;
