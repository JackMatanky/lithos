<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const library_dir = "60_library/";
const lib_course_dir = "60_library/68_courses/";

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

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = "Course Lecture";
const type_value = type_name.replace(/\s/g, "_").toLowerCase();
const file_class = `lib_${type_value}`;

//-------------------------------------------------------------------
// SET COURSE AND DIRECTORY
//-------------------------------------------------------------------
const course_name_alias = await tp.user.include_template(
  tp,
  "68_course_name_alias"
);
const course_value = course_name_alias.split(";")[0];
const course_name = course_name_alias.split(";")[1];

const course_dir = `${lib_course_dir}${course_value}/`;

//-------------------------------------------------------------------
// COURSE METADATA CACHE
//-------------------------------------------------------------------
const course_file_path = `${course_dir}${course_value}.md`;
const course_tfile = await app.vault.getAbstractFileByPath(course_file_path);
const course_file_cache = await app.metadataCache.getFileCache(course_tfile);
const course_main_title = course_file_cache?.frontmatter?.main_title;
const course_main_title_value = course_main_title
  .replaceAll(/\s/g, "_")
  .toLowerCase();

const course_link = `[[${course_value}|${course_main_title}]]`;
const course_value_link = yaml_li(course_link);

//-------------------------------------------------------------------
// AUTHOR CONTACT FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const lecturer_fmatter = course_file_cache?.frontmatter?.lecturer;

let contact_value_link = null_yaml_li;
if (
  !null_arr.includes(lecturer_fmatter) &&
  typeof lecturer_fmatter != "undefined"
) {
  lecturer_arr = lecturer_fmatter.toString().split(",");
  contact_value_link = lecturer_arr
    .map((lecturer) => yaml_li(lecturer))
    .join("");
}

//-------------------------------------------------------------------
// PUBLISHER ORGANIZATION FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const publisher_fmatter = course_file_cache?.frontmatter?.publisher;

let organization_value_link = null_yaml_li;
if (
  !null_arr.includes(publisher_fmatter) &&
  typeof publisher_fmatter != "undefined"
) {
  publisher_arr = publisher_fmatter.toString().split(",");
  organization_value_link = publisher_arr
    .map((publisher) => yaml_li(publisher))
    .join("");
}

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt(`${type_name} Title`, null, true, false);
} else {
  title = tp.file.title;
}
title = title.trim();
title = await tp.user.title_case(title);

//-------------------------------------------------------------------
// SET LECTURE NUMBER
//-------------------------------------------------------------------
const lecture_number_value = await tp.user.include_template(
  tp,
  "61_book_chapter_number"
);
let lecture_number = Number(lecture_number_value.replace(/^0(\d)/g, "$1"));

if (lecture_number_value.length > 2) {
  part_number = lecture_number_value[0];
  section_number = lecture_number_value.slice(1);
  if (section_number[0] == "0") {
    section_number = section_number.slice(1);
  }
  //section_number.replace(/^0(\d)/g, "$1");
  lecture_number = `${part_number}.${section_number}`;
}

//-------------------------------------------------------------------
// SET BOOK URL
//-------------------------------------------------------------------
const url = await tp.system.prompt("Course Lecture URL?", null, false, false);

/* ---------------------------------------------------------- */
/*                       SET DESCRIPTION                      */
/* ---------------------------------------------------------- */
const about = await tp.system.prompt("Lecture Description?", null, false, true);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, "$2")
  .replaceAll(/(\s*)\n/g, "\n")
  .replaceAll(/([^\s])(\s*)$/g, "$1")
  .replaceAll(/\n{1,6}/g, "<new_line>")
  .replaceAll(/(<new_line>)(\w)/g, "\n \n $2")
  .replaceAll(/(<new_line>)(-\s|\d\.\s)/g, "\n $2");

//-------------------------------------------------------------------
// SET LIBRARY STATUS
//-------------------------------------------------------------------
const lib_status = await tp.user.include_template(tp, "60_library_status");
const status_value = lib_status.split(";")[0];
const status_name = lib_status.split(";")[1];

//-------------------------------------------------------------------
// BOOK TITLES, ALIAS, FILE NAME, AND DIRECTORY
//-------------------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const title_subtitle_name = lib_content_titles.full_title_name;
const title_subtitle_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const subtitle = lib_content_titles.sub_title;
const main_title_value = main_title.replaceAll(/\s/g, "_").toLowerCase();

const lecture_title_num_name = `${lecture_number}.${space}${title_subtitle_name}`;
const lecture_title_num_value = `${lecture_number_value}_${main_title_value}_${course_main_title_value}`;

const course_lect_title_name = `${course_main_title}:${space}${main_title}`;
const course_lect_title_value = `${course_main_title_value}_${main_title_value}`;

const file_name = lecture_title_num_value;
const file_section = file_name + hash;

let file_alias =
  new_line +
  [
    course_lect_title_name,
    title_subtitle_name,
    lecture_title_num_name,
    main_title,
    main_title_value,
    title_subtitle_value,
    course_lect_title_value,
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
const info = await tp.user.include_file(
  "68_02_course_lect_online_info_callout"
);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = course_dir;
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
lecturer: <%* tR += contact_value_link %>
publisher: <%* tR += organization_value_link %>
about: |-
 <%* tR += about_value %>
url: <%* tR += url %>
library: <%* tR += course_value_link %>
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
cssclasses: null
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += lecture_title_num_name %>

<%* tR += info %>
## Lecture

<!-- Insert lecture content here  -->

```\timestamp-url
timestamp-url
```

---

<%* tR += sections_content %>
