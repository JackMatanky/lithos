---
title: side_panel_pkm
aliases:
  - Side Panel PKM
  - side panel pkm
  - Dataview PKM
  - dv pkm table
  - side_panel_dv_pkm
  - side_panel_pkm
cssclasses:
  - inline_title_hide
  - \\side_panel_style
  - side_panel_narrow
  - read_narrow_margin
  - table_narrow_margin
  - read_hide_properties
  - metadata_icon_remove
file_class: pkm
date_created: 2023-09-03T19:26
date_modified: 2023-09-27050T3419:17
---
```dataviewjs
/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
const new_line = String.fromCodePoint(0xa);
const space = String.fromCodePoint(0x20);

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const duration = dv.luxon.Duration;
const datetime = dv.luxon.DateTime;
const today = moment().format("YYYY-MM-DD");
const yesterday = moment().subtract(1, "days").format("YYYY-MM-DD");
const shilshom = moment().subtract(2, "days").format("YYYY-MM-DD");
const shilshom_name = moment().subtract(2, "days").format("dddd");
const minus_seven_days = moment().subtract(7, "days").format("YYYY-MM-DD");

//-------------------------------------------------------------------
// OBJECT ARRAYS
//-------------------------------------------------------------------
const type_obj_arr = [
  { key: "category", value: "🏘️Category" },
  { key: "branch", value: "🪑Branch" },
  { key: "field", value: "🚪Field" },
  { key: "subject", value: "🗝️Subject" },
  { key: "topic", value: "🧱Topic" },
  { key: "question", value: "❔Question" },
  { key: "evidence", value: "⚖️Evidence" },
  { key: "step", value: "🪜Step" },
  { key: "conclusion", value: "🎱Conclusion" },
  { key: "quote", value: "⏺️Quote" },
  { key: "idea", value: "💭Idea" },
  { key: "summary", value: "📝Summary" },
  { key: "concept", value: "🎞️Concept" },
  { key: "definition", value: "🪟Definition" },
];

const status_obj_arr = [
  { key: "review", value: "📥Review" },
  { key: "clarify", value: "🌱Clarify" },
  { key: "develop", value: "🪴Develop" },
  { key: "permanent", value: "🌳Permanent" },
  { key: "resource", value: "🗄️Resource" },
];

//-------------------------------------------------------------------
// CONSTANT VARIABLES
//-------------------------------------------------------------------
const pkm_pages = dv.pages('"70_pkm"');
const type_content_filter_arr = ["evidence", "step", "conclusion", "summary"];

const link_today = dv.fileLink(`${today}_pkm`, false, "Today");
const link_yesterday = dv.fileLink(`${yesterday}_pkm`, false, "Yesterday");
const link_shilshom = dv.fileLink(`${shilshom}_pkm`, false, shilshom_name);

//-------------------------------------------------------------------
// FUNCTIONS FOR LIST QUERIES
//-------------------------------------------------------------------
const file_class_filter = (pg, yaml_class) =>
  `${pg.file.frontmatter.file_class}`.startsWith(yaml_class);

const date_created_filter = (pg, date_arg) =>
  dv.equal(
    datetime.fromISO(pg.file.frontmatter.date_created).toFormat("yyyy-MM-dd"),
    date_arg
  );

const file_link = (pg) => {
  const file_path = pg.file.path;
  const file_alias_arr = pg.file.frontmatter.aliases;
  const file_class = pg.file.frontmatter.file_class;

  const file_alias = file_class.includes("code")
    ? file_alias_arr[2]
    : file_alias_arr[0];
  return dv.fileLink(file_path, false, file_alias);
};
const file_content = (pg) => {
  const file_about = pg.file.frontmatter.about;
  const file_type = pg.file.frontmatter.type;
  const content = type_content_filter_arr.includes(file_type)
    ? file_about
    : `${file_about}`.split(/\n/g).filter((x) => x.match(/^\w/g));
  return content;
};

const page_filter = (pg_array, yaml_class, date_arg) =>
  pg_array.filter(
    (page) =>
      file_class_filter(page, yaml_class) && date_created_filter(page, date_arg)
  );

function list_nested(pg_array, yaml_class, date_arg) {
  const list_root = dv.el("ul", "");
  const page_arr = page_filter(pg_array, yaml_class, date_arg);

  const page_list = page_arr.forEach((page) => {
    dv.el("li", file_link(page), { container: list_root });
    const page_root = dv.el("ul", "", { container: list_root });
    dv.el("li", file_content(page), { container: page_root });
  });
  return page_list;
}

//-------------------------------------------------------------------
// LISTS OF PKM LISTS OUTPUT
//-------------------------------------------------------------------
dv.header(1, link_today);
list_nested(pkm_pages, "pkm", today);

dv.header(1, link_yesterday);
list_nested(pkm_pages, "pkm", yesterday);

dv.header(1, link_shilshom);
list_nested(pkm_pages, "pkm", shilshom);
```