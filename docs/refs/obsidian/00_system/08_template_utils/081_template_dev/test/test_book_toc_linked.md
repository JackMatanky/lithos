<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const lib_books_dir = "60_library/61_books/";

/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
const new_line = String.fromCodePoint(0xa);
const space = String.fromCodePoint(0x20);
const backtick = String.fromCodePoint(0x60);

//-------------------------------------------------------------------
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const book_name_alias = "61_book_name_alias";

//-------------------------------------------------------------------
// SET BOOK FILE NAME, ALIAS, AND DIRECTORY
//-------------------------------------------------------------------
const book_name_alias = await tp.user.include_template(
  tp,
  "61_book_name_alias"
);
const book_value = book_name_alias.split(";")[0];
const book_name = book_name_alias.split(";")[1];
const book_dir = `${lib_books_dir}${book_value}/`;

//-------------------------------------------------------------------
// BOOK CHAPTERS OBJECT ARRAY
//-------------------------------------------------------------------
const chapter_obj_arr = (
  await tp.user.file_name_alias_by_class_type({
    dir: book_dir,
    file_class: "lib",
    type: "book_chapter",
  })
).filter((file) => file.value != "null" && file.value != "_user_input");

//-------------------------------------------------------------------
// BOOK CONTENTS AND PDF/EPUB SECTION
//-------------------------------------------------------------------
let ol_book_toc_obj_arr = [];
let ol_book_toc_count = 0;

for (let i = 0; i < chapter_obj_arr.length; i++) {
  // CHAPTER METADATA CACHE
  chapter_file_path = `${book_dir}${chapter_obj_arr[i].value}.md`;
  chapter_tfile = await app.vault.getAbstractFileByPath(chapter_file_path);
  chapter_file_cache = await app.metadataCache.getFileCache(chapter_tfile);
  chapter_main_title = chapter_file_cache?.frontmatter?.title;
  chapter_main_title_value = chapter_main_title
    .replaceAll(/\s/g, "_")
    .toLowerCase();

  chapter_link = `[[${chapter_obj_arr[i].value}|${chapter_obj_arr[i].key}]]`;
  ol_book_toc_count = ol_book_toc_count + 1;
  ol_book_toc_obj = { key: ol_book_toc_count, value: chapter_link };
  ol_book_toc_obj_arr.push(ol_book_toc_obj);
}
const ol_book_toc = ol_book_toc_obj_arr
  .map((obj) => `${obj.key}.${space}${obj.value}`)
  .join(new_line);

tR += ol_book_toc;
%>