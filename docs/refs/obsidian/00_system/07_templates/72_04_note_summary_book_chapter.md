<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";
const library_dir = "60_library/";
const lib_books_dir = library_dir + "61_books/";
const pkm_dir = "70_pkm/";
const pkm_lab_dir = pkm_dir + "_knowledge_lab/";

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = "Summary";
const type_value = "summary";
const type_value_short = "sum";
const file_class = "pkm_zettel";

/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const hr_line = hyphen.repeat(3);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const colon = String.fromCodePoint(0x3a);
const two_percent = String.fromCodePoint(0x25).repeat(2);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

//Text Formatting
const tbl_start = `${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end = `${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${hyphen}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${great_than}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_start}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;
const dv_colon = `${colon.repeat(2)}${space}`;

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
// FORMATTING
const head_lvl = (level, heading) => [hash.repeat(level), heading].join(space);
const re_snake_under = /(;\s)|(:\s)|(\-\s\-)|(\s)|(\-)/g;
const re_snake_remove = /(,|'|:|;)/g;
const re_snake_double_under = /_{2,}/g;
const re_snake_trim_under = /^_+|_+$/g;
const snake_case_fmt = (name) =>
  name
    .replace(re_snake_under, '_')
    .replace(re_snake_remove, '')
    .replace(re_snake_double_under, '_')
    .replace(re_snake_trim_under, '');
    .toLowerCase();
const md_ext = (file_name) => file_name + '.md';
const quote_enclose = (content) => `"${content}"`;

const code_inline = (content) => backtick + content + backtick;
const cmnt_obsidian = (content) =>
  [two_percent, content, two_percent].join(space);
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

// LINKS
const regex_link = /.*\[([\w_].+)\|([\w\s].+)\].+/g;
const link_alias = (file, alias) => '[[' + [file, alias].join('|') + ']]';
const link_tbl_alias = (file, alias) => '[[' + [file, alias].join('\\|') + ']]';

// YAML PROPERTIES
const yaml_li = (value) => new_line + ul_yaml + `"${value}"`;
const yaml_li_link = (file, alias) =>
  new_line + ul_yaml + `"${link_alias(file, alias)}"`;

// CALLOUT
const call_title = (call_type, title) =>
  [great_than, `[!${call_type}]`, title].join(space);

// CALLOUT TABLE
const call_tbl_row = (content) =>
  [
    great_than,
    String.fromCodePoint(0x7c),
    content,
    String.fromCodePoint(0x7c),
    space,
  ].join(space);
const call_tbl_div = (int) =>
  call_tbl_row(Array(int).fill(tbl_cent).join(tbl_pipe));

// DATE
const date_time = (date, time = '00:00') => moment(`${date}T${time}`);
const date_fmt = (format, date, time = '00:00') =>
  moment(`${date}T${time}`).format(format);
const date_add_sub_fmt = (format, unit, value, date, time = '00:00') =>
  value > 0
    ? moment(`${date}T${time}`).add(value, unit).format(format)
    : moment(`${date}T${time}`).subtract(Math.abs(value), unit).format(format);

// DATAVIEW - INLINE
const dv_inline = (key, value) =>
  '[' + key + colon.repeat(2) + space + value + ']';
const dv_yaml = (property) => 'file.frontmatter.' + property;
const dv_content_link = code_inline(
  [
    'dv:',
    `link(this.file.name + "#" +`,
    `this.${dv_yaml('aliases[0]')},`,
    `"Contents")`,
  ].join(space)
);

// Utility: Split a semicolon-delimited string into trimmed components
function parse_semicolon_values(input, expected_count) {
  const parts = input.split(";");
  if (expected_count !== undefined && parts.length < expected_count) {
    throw new Error(`Expected ${expected_count} values but got ${parts.length}: "${input}"`);
  }

  return parts.map((s, i) => (i === 0 ? s.trim() : s));
}

// OBSIDIAN API
async function file_path(file_name) {
  const path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${md_ext(file_name)}`))
    .map((file) => file.path)[0];
  return path
}

async function metadata_alias(file_name) {
  const path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${md_ext(file_name)}`))
    .map((file) => file.path)[0];
  const abstract_file = await app.vault.getAbstractFileByPath(path);
  const file_cache = await app.metadataCache.getFileCache(abstract_file);
  return file_cache?.frontmatter?.aliases[0];
}

/* ---------------------------------------------------------- */
/*                      GENERAL VARIABLES                     */
/* ---------------------------------------------------------- */

/* --------------------- NULL VARIABLES --------------------- */
const null_link = link_alias('null', 'Null');
const null_yaml_li = yaml_li(null_link);
const null_arr = ['', 'null', null_link, null];

/* -------- FILE CREATION AND MODIFIED DATE VARIABLES ------- */
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// SET SUMMARIZED BOOK AND CHAPTER CONTENT
//-------------------------------------------------------------------
const book_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: lib_books_dir,
  file_class: "lib",
  type: "book",
});
const book_obj = await tp.system.suggester(
  (item) => item.key,
  book_obj_arr,
  false,
  "Library Book Resource?",
);
const book_value = book_obj.value;
const book_name = book_obj.key;
const book_yaml = yaml_li_link(book_value, book_name);
const book_path = await file_path(book_value);

const chapter_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: book_path,
  file_class: "lib",
  type: "book_chapter",
});
const chapter_obj = await tp.system.suggester(
  (item) => item.key,
  chapter_obj_arr,
  false,
  "Library Book Chapter Resource?",
);
const chapter_value = chapter_obj.value;
const chapter_name = chapter_obj.key;
const chapter_yaml = yaml_li_link(chapter_value, chapter_name);

const lib_resource_yaml = [chapter_obj, book_obj]
  .map((x) => yaml_li_link(x.value, x.key))
  .join("");


/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
let title;
let title_value = "";

const has_title = !tp.file.title.startsWith("Untitled");
if (!has_title) {
  if (book_value == "null" && chapter_value == "null") {
    title = await tp.system.prompt(`Title of Summarized Content?`, null, true, false);
    title = title.trim();
    
    const lib_content_titles = await tp.user.lib_content_titles(title);
    
    const full_title_name = lib_content_titles.full_title_name;
    const full_title_value = lib_content_titles.full_title_value;
    title = `${type_name} of ${full_title_name}`;
    title_value = `${type_value}_${full_title_value}`;
  } else {
    title = `${type_name} of ${lib_resource_name}`;
    title_value = `${type_value_short}_${lib_resource_value}`;
  }
} else {
  title = tp.file.title;
  title = title.trim();
  title = await tp.user.title_case(title);
}

/* ---------------------------------------------------------- */
/*          FRONTMATTER TITLE, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
const full_title_name = title;
const short_title_name = title.toLowerCase();
const full_title_value = snake_case_fmt(short_title_name);
const short_title_value = snake_case_fmt(title_value ? title_value : title);

const file_alias = [
  full_title_name,
  short_title_name,
  full_title_value,
  short_title_value,
]
  .map((x) => yaml_li(x))
  .join('');

const file_name = short_title_value;
const file_section = file_name + hash;

const file_dir = pkm_lab_dir + book_value + '/' + file_name;

const full_summary_title_name = ['Full', full_title_name].join(space);
const short_summary_title_name = full_summary_title_name.toLowerCase();
const full_summary_title_value = snake_case_fmt(short_summary_title_name);
const short_summary_title_value = [short_title_value, 'full'].join('_');

/* ---------------------------------------------------------- */
/*                       SET DESCRIPTION                      */
/* ---------------------------------------------------------- */
const about = await tp.system.prompt(
  `Write a quick ${type_value} of the content?`,
  null,
  false,
  true
);
const about_value = await tp.user.yaml_multiline(about);

/* ---------------------------------------------------------- */
/*                      SET REFERENCE URL                     */
/* ---------------------------------------------------------- */
const url = await tp.system.prompt(`${type_name} Reference URL?`);

/* ---------------------------------------------------------- */
/*   SET PILLAR FILE NAME AND TITLE; PRESET KNOW. EXPANSION   */
/* ---------------------------------------------------------- */
const { value: pillar_value, link: pillar_yaml } =
  await tp.user.multi_suggester({
    tp,
    items: await tp.user.file_by_status({
      dir: pillars_dir,
      status: "active",
    }),
    type: "pillar",
    context: "education",
  });

/* ---------------------------------------------------------- */
/*                      SET RELATED GOAL                      */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${type_name}?`,
);
const goal_link = `${ul}[[${goal}]]`;

/* ---------------------------------------------------------- */
/*              SET PKM TREE FILE NAMES AND LINKS             */
/* ---------------------------------------------------------- */
const tree_name_link = await tp.user.include_template(
  tp,
  "70_pkm_tree_name_link",
);
const [
  pkm_file_dir,
  category_yaml,
  branch_yaml,
  field_yaml,
  subject_yaml,
  topic_yaml,
  subtopic_yaml,
] = parse_semicolon_values(tree_name_link);

/* ---------------------------------------------------------- */
/*                       SET NOTE STATUS                      */
/* ---------------------------------------------------------- */
const note_status = await tp.user.include_template(tp, "80_note_status");
const [status_name, status_value] = parse_semicolon_values(note_status);

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: "Idea Compass",
    toc_level: 1,
    toc_key: "Idea Compass",
    file: "80_note_idea_compass",
  },
  {
    head_key: "Related Knowledge",
    toc_level: 1,
    toc_key: "PKM",
    file: "100_70_related_pkm_sect",
  },
  {
    head_key: "Related Library Content",
    toc_level: 1,
    toc_key: "Library",
    file: "100_62_related_lib_sect_pkm_file",
  },
  {
    head_key: "Related Goals",
    toc_level: 2,
    toc_key: "Goals",
    file: null,
  },
  {
    head_key: "Related Tasks and Events",
    toc_level: 2,
    toc_key: "Tasks",
    file: "100_41_related_task_sect_related_proj",
  },
  {
    head_key: "Related Directory",
    toc_level: 2,
    toc_key: "Directory",
    file: "100_50_related_dir_sect",
  },
];

// Content, heading, and table of contents link
for (let i = 0; i < section_obj_arr.length; i++) {
  if (!section_obj_arr[i].file) {
    continue;
  }
  section_obj_arr[i].content = await tp.user.include_template(
    tp,
    section_obj_arr[i].file
  );
  section_obj_arr[i].head = head_lvl(2, section_obj_arr[i].head_key);
  section_obj_arr[i].toc = link_tbl_alias(
    file_section + section_obj_arr[i].head_key,
    section_obj_arr[i].toc_key
  );
}

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_lvl = (int) =>
  call_tbl_row(
    section_obj_arr
      .filter((x) => x.toc_level == int)
      .map((x) => x.toc)
      .join(tbl_pipe)
  );

const toc = [
  call_title('toc', dv_content_link),
  call_start,
  toc_lvl(1),
  call_tbl_div(3),
  toc_lvl(2),
].join(new_line);

/* ---------------------------------------------------------- */
/*                        GOAL SECTION                        */
/* ---------------------------------------------------------- */
const goal_section = [
  head_lvl(3, "Outgoing Goals Links"),
  cmnt_html("Link related goals here"),
  goal_link,
  head_lvl(3, "Value Goals"),
  head_lvl(3, "Outcome Goals"),
  hr_line
].join(two_new_line);

section_obj_arr[3].content = goal_section + new_line;

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = pkm_lab_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
pillar: <%* tR += pillar_yaml %>
category: <%* tR += category_yaml %>
branch: <%* tR += branch_yaml %>
field: <%* tR += field_yaml %>
subject: <%* tR += subject_yaml %>
topic: <%* tR += topic_yaml %>
subtopic: <%* tR += subtopic_yaml %>
library: <%* tR += lib_resource_yaml %>
about: |-
 <%* tR += about_value %>
url: <%* tR += url %>
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_title_name %>

> [!Summary]
>
> - **Resource**: `dv: this.file.frontmatter.library[0]`
>
> - **Source**:: [[Cummings_2019_Real Analysis_05_The Topology of R.pdf|Real Analysis: The Topology of R, by Jay Cummings]]
>
> - **Full Summary**:: [[sum_05_the_topology_of_r_real_analysis_full|Full Summary of Real Analysis: The Topology of R]]
>
> - **Summary**: `dv: this.file.frontmatter.about`

---

<%* tR += sections_content %>