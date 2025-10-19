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
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const related_task_sect = "100_40_related_task_sect_general";
const related_dir_sect = "100_50_related_dir_sect";
const related_lib_sect = "100_60_related_lib_sect";
const related_pkm_sect = "100_70_related_pkm_sect";

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

const book_dir = `${lib_books_dir}${book_value}`;

//-------------------------------------------------------------------
// BOOK METADATA CACHE
//-------------------------------------------------------------------
const book_file_path = `${book_dir}/${book_value}.md`;
const book_tfile = await app.vault.getAbstractFileByPath(book_file_path);
const book_file_cache = await app.metadataCache.getFileCache(book_tfile);
const book_main_title = book_file_cache?.frontmatter?.main_title;
const book_main_title_value = book_main_title
  .replaceAll(/\s/g, "_")
  .toLowerCase();

const book_link = `[[${book_value}|${book_main_title}]]`;
const book_value_link = yaml_li(book_link);

//-------------------------------------------------------------------
// AUTHOR CONTACT FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const author_fmatter = book_file_cache?.frontmatter?.author;

let contact_value_link = null_yaml_li;
if (
  !null_arr.includes(author_fmatter) &&
  typeof author_fmatter != "undefined"
) {
  author_arr = author_fmatter.toString().split(",");
  contact_value_link = author_arr
    .map((author) => yaml_li(author))
    .join("");
}

//-------------------------------------------------------------------
// PUBLISHER ORGANIZATION FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const publisher_fmatter = book_file_cache?.frontmatter?.publisher;

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

//-------------------------------------------------------------------
// BOOK PUBLISHED DATE
//-------------------------------------------------------------------
const year_published = book_file_cache?.frontmatter?.year_published;

//-------------------------------------------------------------------
// BOOK URL
//-------------------------------------------------------------------
const url = book_file_cache?.frontmatter?.url;

//-------------------------------------------------------------------
// RELATED PKM SECTION
//-------------------------------------------------------------------
heading = "Related Knowledge";
const head_pkm_sect = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_pkm_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_pkm_section = include_arr;

//-------------------------------------------------------------------
// RELATED LIBRARY SECTION
//-------------------------------------------------------------------
heading = "Related Library Content";
const head_lib_sect = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_lib_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_lib_section = include_arr;

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION
//-------------------------------------------------------------------
heading = "Related Tasks and Events";
const head_task_sect = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_task_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_task_section = include_arr;

//-------------------------------------------------------------------
// RELATED DIRECTORY SECTION
//-------------------------------------------------------------------
heading = "Related Directory";
const head_dir_sect = `${head_lvl(2)}${heading}${two_new_line}`;

temp_file_path = `${sys_temp_include_dir}${related_dir_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_dir_section = include_arr;

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_dv_contents = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;
const toc_title = `${call_start}[!toc]${space}${toc_dv_contents}${two_space}${new_line}${call_start}${new_line}`;
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;

//-------------------------------------------------------------------
// BOOK CHAPTER DETAILS CALLOUT
//-------------------------------------------------------------------
const chapter_info = await tp.user.include_file("61_01_book_chapter_info_callout");

//-------------------------------------------------------------------
// BOOK CHPATER CONTENT COMMENT
//-------------------------------------------------------------------
comment = "Insert chapter content here";
const comment_chapter_file = `${cmnt_html_start}${comment}${cmnt_html_end}`;

//-------------------------------------------------------------------
// BOOK CHAPTER DETAILS
//-------------------------------------------------------------------
const chapter_details_input = (
  await tp.system.prompt("Chapter Details Object Array", null, false, true)
)
  .replaceAll(/\n/g, "")
  .replace(/;$/g, "")
  .split(";");

// BOOK CHAPTER DETAILS REGEX
const number_regex = /(.+_number:\s")(\d{1,4})(",\s.+)/g;
const title_regex = /(.+,\stitle:\s")(.+?)(",\s.+)/g;
const page_start_regex = /(.+,\spage_start:\s")(\d{1,4}|[xvi].+?)(",\s.+)/g;
const page_end_regex = /(.+,\spage_end:\s")(\d{1,4}|[xvi].+?)("\})/g;

// BOOK CHAPTER DETAILS ISOLATED
const chapter_number_arr = chapter_details_input.map((chapter) =>
  chapter.replace(number_regex, "$2")
);
const chapter_title_arr = chapter_details_input.map((chapter) =>
  chapter.replace(title_regex, "$2")
);
const chapter_page_start_arr = chapter_details_input.map((chapter) =>
  chapter.replace(page_start_regex, "$2")
);
const chapter_page_end_arr = chapter_details_input.map((chapter) =>
  chapter.replace(page_end_regex, "$2")
);

// BOOK CHAPTER DETAILS OBJECT ARRAY
let chapter_details_obj_arr = [];
for (let i = 0; i < chapter_details_input.length; i++) {
  chapter_obj = {
    number: chapter_number_arr[i],
    title: chapter_title_arr[i],
    page_start: chapter_page_start_arr[i],
    page_end: chapter_page_end_arr[i],
  };
  chapter_details_obj_arr.push(chapter_obj);
}

//-------------------------------------------------------------------
// BOOK CHAPTER FRONTMATTER
//-------------------------------------------------------------------
let yaml_alias;
let yaml_title;
let yaml_subtitle;

const yaml_author_book_info = [
  `author:${space}${contact_value_link}`,
  "editor:",
  "translator:",
  `year_published:${space}${year_published}`,
  `publisher:${space}${organization_value_link}`,
].join(new_line);

let yaml_page_start;
let yaml_page_end;

const yaml_bottom = [
  "doi:",
  `url:${space}${url}`,
  `library:${space}${book_value_link}`,
  "cssclasses:",
  "status: undetermined",
  `type:${space}${type_value}`,
  `file_class:${space}${file_class}`,
  `date_created:${space}${date_created}`,
  `date_modified:${space}${date_modified}`,
  "tags:",
  hr_line,
].join(new_line);

let file_name;
let file_content;

//-------------------------------------------------------------------
// CREATE NEW BOOK CHAPTER FILES
//-------------------------------------------------------------------
for (let i = 0; i < chapter_details_obj_arr.length; i++) {
  const chapter_number_value = chapter_details_obj_arr[i].number;
  chapter_number = "";
  if (
    chapter_number_value.length <= 2 ||
    chapter_number_value.match(/^(00\d|0\d\d)/g)
  ) {
    chapter_number = part_section_number.replace(/^0{1,2}/g, "");
  } else if (chapter_number_value.length >= 3) {
    part_number = chapter_number_value[0];
    part_section_number = chapter_number_value.slice(1);
    if (chapter_number_value.match(/^\d00/g)) {
      chapter_number = part_number;
    } else {
      chapter_number = `${part_number}.${part_section_number.replace(
        /^(0){1,2}/g,
        ""
      )}`;
    }
  }
  title = chapter_details_obj_arr[i].title;
  title = await tp.user.title_case(title);
  const lib_content_titles = await tp.user.lib_content_titles(title);
  const full_title_name = lib_content_titles.full_title_name;
  const full_title_value = lib_content_titles.full_title_value;
  const main_title = lib_content_titles.main_title;
  const main_title_value = main_title.replaceAll(/[\s-]/g, "_").toLowerCase();
  const subtitle = lib_content_titles.sub_title;
  const title_number_name = `${chapter_number}.${space}${full_title_name}`;
  const book_chapter_title_name = `${book_main_title}:${space}${main_title}`;
  const book_chapter_title_value = `${book_main_title_value}_${main_title_value}`;

  const file_name = `${chapter_number_value}_${main_title_value}_${book_main_title_value}`;
  const file_section = `${file_name}${hash}`;

  const file_alias =
    new_line +
    [
      book_chapter_title_name,
      full_title_name,
      title_number_name,
      main_title,
      main_title_value,
      full_title_value,
      book_chapter_title_value,
      file_name,
    ]
      .map((x) => `${ul_yaml}"${x}"`)
      .join(new_line);

  const page_start = chapter_details_obj_arr[i].page_start;
  const page_end = chapter_details_obj_arr[i].page_end;

  heading = "Related Knowledge";
  toc_pkm_sect = `[[${file_section}${heading}\\|PKM]]`;
  heading = "Related Library Content";
  toc_lib_sect = `[[${file_section}${heading}\\|Library]]`;
  heading = "Related Tasks and Events";
  toc_task_sect = `[[${file_section}${heading}\\|Tasks]]`;
  heading = "Related Directory";
  toc_dir_sect = `[[${file_section}${heading}\\|Directory]]`;

  // TABLE OF CONTENTS CALLOUT
  toc_body_high = `${call_tbl_start}${toc_pkm_sect}${tbl_pipe}${toc_lib_sect}${tbl_pipe}${toc_task_sect}${tbl_pipe}${toc_dir_sect}${call_tbl_end}${new_line}`;
  toc_body = `${toc_body_high}${toc_body_div}`;
  toc = `${toc_title}${toc_body}${two_new_line}`;

  // FILE SECTIONS
  const related_pkm = `${head_pkm_sect}${toc}${related_pkm_section}`;
  const related_lib = `${head_lib_sect}${toc}${related_lib_section}`;
  const related_task = `${head_task_sect}${toc}${related_task_section}`;
  const related_dir = `${head_dir_sect}${toc}${related_dir_section}`;

  const frontmatter = [
    hr_line,
    `title:${space}${file_name}`,
    `uuid:${space}${await tp.user.uuid()}`,
    `aliases:${space}${file_alias}`,
    `main_title:${space}${main_title}`,
    `subtitle:${space}${subtitle}`,
    yaml_author_book_info,
    `page_start:${space}${page_start}`,
    `page_end:${space}${page_end}`,
    yaml_bottom
  ].join(new_line);

  file_content = `${frontmatter}
${head_lvl(1)}${title_number_name}${new_line}
${chapter_info}${new_line}
${hr_line}${new_line}
${comment_chapter_file}${new_line}
${hr_line}${new_line}
${related_pkm}
${related_lib}
${related_task}
${related_dir}`;

  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(book_dir)
  );
}
%>
