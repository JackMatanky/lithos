// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//-------------------------------------------------------------------
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
const lib_status = `default(((x) => {
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
// SECT: >>>>> DATA SOURCES <<<<<
//-------------------------------------------------------------------
// Library directory
const lib_dir = `"60_library"`;

// Inbox directory
const inbox_dir = `"inbox"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTERS <<<<<
//-------------------------------------------------------------------
// File class filters
const class_filter = `contains(file.frontmatter.file_class, "lib")`;

// Resource content filter
const resource_filter = `contains(${yaml_status}, "resource")`;

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
// DATAVIEW TABLE FOR JOURNALS WRITTEN BETWEEN DATES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// COMPLETED OPTIONS: "done", "completed",
// ACTIVE OPTIONS: "active", "to_do", "in_progress"
// SCHEDULE OPTIONS: "schedule", "on_hold"
// CREATED OPTIONS: "new", "created"
// DETERMINE OPTIONS: "undetermined", "determine"

async function dv_lib_status_dates({
  status: status,
  start_date: date_start,
  end_date: date_end,
  md: md,
}) {
  let sort_field = yaml_type;
  let date_field = "null";
  let filter = "null";

  // DATE fields DONE, NEW, and MODIFIED statuses
  if (status.startsWith("don") || status.startsWith("comp")) {
    date_field = "Completed";
    sort_field = "Completed";
  } else if (status.startsWith("new") || status.startsWith("created")) {
    date_field = "file.frontmatter.date_created";
  } else if (status.startsWith("mod")) {
    date_field = "file.frontmatter.date_modified";
  }

  // DATE FILTER
  if (date_end == "") {
    // SPECIFIC date
    filter = `date(${date_field}) = date(${date_start})`;
  } else {
    // BETWEEN two dates
    filter = `date(${date_field}) >= date(${date_start})
    AND date(${date_field}) <= date(${date_end})`;
  }

  // FILTERs for ACTIVE, SCHEDULE, and UNDETERMINED statuses
  if (status.startsWith("act")) {
    // Active content filter
    filter = `(contains(${yaml_status}, "to_do")
      OR contains(${yaml_status}, "in_progress"))`;
  } else if (status.startsWith("sch")) {
    filter = `(contains(${yaml_status}, "schedule")
      OR contains(${yaml_status}, "on_hold"))`;
  } else if (status.startsWith("und") || status.startsWith("det")) {
    filter = `contains(${yaml_status}, "undetermined")`;
  }

  let dataview_query;
  if (md != "true") {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${title_link},
    ${lib_status},
    ${author},
    ${publish_date},
    ${file_type},
    ${tags}
FROM
    ${lib_dir}
    OR ${inbox_dir}
WHERE
    ${class_filter}
    AND ${filter}
SORT
    ${lib_status_sort},
    ${sort_field},
    ${alias} ASC
LIMIT 25
${three_backtick}`;
  } else {
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${title_link},
    ${lib_status},
    ${author},
    ${publish_date},
    ${file_type},
    ${tags}
FROM
    ${lib_dir}
    OR ${inbox_dir}
WHERE
    ${class_filter}
    AND ${filter}
SORT
    ${lib_status_sort},
    ${sort_field},
    ${alias} ASC
${three_backtick}`;
  }

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

module.exports = dv_lib_status_dates;
