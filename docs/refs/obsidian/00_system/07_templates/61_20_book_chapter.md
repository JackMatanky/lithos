<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const library_dir = "60_library/";
const lib_books_dir = "60_library/61_books/";

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
const head_lvl = (int) => `${hash.repeat(int)}${space}`;
const yaml_li = (value) => `${new_line}${ul_yaml}"${value}"`;
const link_alias = (file, alias) => ["[[" + file, alias + "]]"].join("|");
const link_tbl_alias = (file, alias) => ["[[" + file, alias + "]]"].join("\\|");
const cmnt_ob_start = `${two_percent}${space}`;
const cmnt_ob_end = `${space}${two_percent}`;
const cmnt_html_start = `${less_than}${excl}${two_hyphen}${space}`;
const cmnt_html_end = `${space}${two_hyphen}${great_than}`;
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

//-------------------------------------------------------------------
// FORMATTING FUNCTIONS
//-------------------------------------------------------------------
const snake_case_fmt = (name) =>
  name.replaceAll(/(\-\s\-)|(\s)|(\-)]/g, "_").toLowerCase();

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
const null_arr = ["", "null", "[[null|Null]]", null];
const null_link = "[[null|Null]]";
const null_yaml_li = yaml_li(null_link);

const dv_yaml = "file.frontmatter";
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

//-------------------------------------------------------------------
// BOOK CHAPTER TYPE AND FILE CLASS
//-------------------------------------------------------------------
const type_name = "Book Chapter";
const type_value = type_name.replaceAll(/\s/g, "_").toLowerCase();
const file_class = `lib_${type_value}`;

//-------------------------------------------------------------------
// SET BOOK AND DIRECTORY
//-------------------------------------------------------------------
const book_name_alias = await tp.user.include_template(
  tp,
  "61_book_name_alias"
);
const book_value = book_name_alias.split(";")[0];
const book_name = book_name_alias.split(";")[1];

const book_dir = `${lib_books_dir}${book_value}/`;

//-------------------------------------------------------------------
// BOOK METADATA CACHE
//-------------------------------------------------------------------
const book_tfile = tp.file.find_tfile(`${book_value}.md`);
const book_file_cache = await app.metadataCache.getFileCache(book_tfile);
const book_main_title = book_file_cache?.frontmatter?.main_title;
const book_main_title_value = book_main_title
  .replaceAll(/\s/g, "_")
  .toLowerCase();

const book_link = `[[${book_value}|${book_main_title}]]`;

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith(`Untitled`);
let title;
if (!has_title) {
  title = await tp.system.prompt(`${type_name} Title`, null, true, false);
} else {
  title = tp.file.title;
}
title = title.trim();
title = await tp.user.title_case(title);

//-------------------------------------------------------------------
// SET CHAPTER NUMBER
//-------------------------------------------------------------------
const chapter_number = await tp.user.include_template(
  tp,
  "61_book_chapter_number"
);

//-------------------------------------------------------------------
// AUTHOR CONTACT FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const contact_value = book_file_cache?.frontmatter?.author;
const contact_tfile = tp.file.find_tfile(`${contact_value}.md`);
const contact_file_cache = await app.metadataCache.getFileCache(contact_tfile);
const contact_name = contact_file_cache?.frontmatter?.aliases[0];
const contact_link = `[[${contact_value}|${contact_name}]]`;
const contact_value_link = yaml_li(contact_link);

//-------------------------------------------------------------------
// PUBLISHER ORGANIZATION FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const organization_value = book_file_cache?.frontmatter?.publisher;
const organization_tfile = tp.file.find_tfile(`${organization_value}.md`);
const organization_file_cache = await app.metadataCache.getFileCache(
  organization_tfile
);
const organization_name = organization_file_cache?.frontmatter?.aliases[0];
const organization_link = `[[${organization_value}|${organization_name}]]`;
const organization_value_link = yaml_li(organization_link);

//-------------------------------------------------------------------
// BOOK PUBLISHED DATE
//-------------------------------------------------------------------
const year_published = book_file_cache?.frontmatter?.year_published;

//-------------------------------------------------------------------
// SET PAGES
//-------------------------------------------------------------------
const page_start = await tp.system.prompt(
  "What is the chapter's starting page?"
);
const page_end = await tp.system.prompt("What is the chapter's ending page?");

//-------------------------------------------------------------------
// BOOK URL
//-------------------------------------------------------------------
const url = book_file_cache?.frontmatter?.url;

//-------------------------------------------------------------------
// SET LIBRARY STATUS
//-------------------------------------------------------------------
const lib_status = await tp.user.include_template(tp, "60_library_status");
const status_value = lib_status.split(";")[0];
const status_name = lib_status.split(";")[1];

//-------------------------------------------------------------------
// BOOK CHAPTER TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const title_subtitle_name = lib_content_titles.full_title_name;
const title_subtitle_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sub_title;
const main_title_value = main_title.replaceAll(/\s/g, "_").toLowerCase();

const book_chapter_title_name = `${book_main_title}: ${main_title}`;
const book_chapter_title_value = `${book_main_title_value}_${main_title_value}`;

const file_name = `${chapter_number}_${main_title_value}_${book_main_title_value}`;
const file_section = file_name + hash;

let file_alias =
  new_line +
  [
    title_subtitle_name,
    title_subtitle_value,
    main_title,
    main_title_value,
    book_chapter_title_name,
    book_chapter_title_value,
    file_name,
  ]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: "Related Knowledge",
    toc_key: "PKM",
    file: "100_70_related_pkm_sect",
  },
  {
    head_key: "Related Library Content",
    toc_key: "Library",
    file: "100_60_related_lib_sect",
  },
  {
    head_key: "Related Tasks and Events",
    toc_key: "Related Tasks",
    file: "100_40_related_task_sect_general",
  },
  {
    head_key: "Related Directory",
    toc_key: "Directory",
    file: "100_50_related_dir_sect",
  },
];

// Content, heading, and table of contents link
for (let i = 0; i < section_obj_arr.length; i++) {
  section_obj_arr[i].content = await tp.user.include_template(
    tp,
    section_obj_arr[i].file
  );
}
section_obj_arr.map((x) => (x.head = head_lvl(2) + x.head_key));
section_obj_arr.map(
  (x) => (x.toc = `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
);

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_title = `${call_start}[!toc]${space}${dv_content_link}`;

const toc_body_high =
  call_tbl_start +
  section_obj_arr.map((x) => x.toc).join(tbl_pipe) +
  call_tbl_end;
const toc_body_div =
  call_tbl_start + Array(4).fill(tbl_cent).join(tbl_pipe) + call_tbl_end;

const toc = [toc_title, call_start, toc_body_high, toc_body_div].join(new_line);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info = await tp.user.include_file("61_01_book_chapter_info_callout");

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = book_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
main_title: <%* tR += main_title %>
subtitle: <%* tR += subtitle %>
author: <%* tR += contact_value_link %>
editor:
translator:
year_published: <%* tR += year_published %>
publisher: <%* tR += organization_value_link %>
page_start: <%* tR += page_start %>
page_end: <%* tR += page_end %>
doi:
url: <%* tR += url %>
library:
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
cssclasses:
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_title_name %>

<%* tR += info %>

---

<!-- Insert chapter content here  -->

---

<%* tR += sections_content %>