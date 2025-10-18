<%*
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
// BOOK CHAPTER TYPE AND FILE CLASS
const chapter_type_name = "Book Chapter";
const chapter_type_value = chapter_type_name
  .replaceAll(/\s/g, "_")
  .toLowerCase();
const chapter_file_class = `lib_${chapter_type_value}`;

// BOOK CHPATER INFO CALLOUT
const info_chapter = await tp.user.include_file(
  "61_01_book_chapter_info_callout"
);

// BOOK CHPATER CONTENT COMMENT
const comment_chapter_file = cmnt_html("Insert chapter content here") + new_line;

// BOOK CHAPTER DETAILS
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
};

const chapter_details_input = (
  await tp.system.prompt("Chapter Details Object Array", null, false, true)
)
  .split(';\n').map(x => x.trim()).filter(x => x.length);

// BOOK CHAPTER DETAILS REGEX
const number_regex = /(.+_number:\s")(\d{1,4})(",\s.+)/g;
const title_regex = /(.+,\stitle:\s")(.+?)(",\s.+)/g;
const page_start_regex = /(.+,\spage_start:\s")(\d{1,4}|[xvi].+?)(",\s.+)/g;
const page_end_regex = /(.+,\spage_end:\s")(\d{1,4}|[xvi].+?)("\})/g;

const patterns = {
  number: /.+_number:\s"(\d{1,4})",\s*title:/,
  title: /,\s+title:\s"(.+?)",\s*page_start:/,
  page_start: /,\s+page_start:\s"(\d{1,4}|[xvi].+?)",\s*page_end:/,
  page_end: /,\s+page_end:\s"(\d{1,4}|[xvi].+?)"\}/,
};

const regex_patterns = {
  number: /chapter_number:\s*"(\d{1,4})"/i,
  title: /title:\s*"([^"]+)"/i,
  page_start: /page_start:\s*"([^"]+)"/i,
  page_end: /page_end:\s*"([^"]+)"/i,
};

// BOOK CHAPTER DETAILS ISOLATED
const extract_chapter_detail = (input, regex) => {
  const match = input.match(regex);
  return match ? match[1] : "";
};

// BOOK CHAPTER DETAILS OBJECT ARRAY
// Parse input array into an array of chapter detail objects
const chapter_details_obj_arr = chapter_details_input.map((line) => ({
  number: extract_chapter_detail(line, regex_patterns.number),
  title: extract_chapter_detail(line, regex_patterns.title),
  page_start: extract_chapter_detail(line, regex_patterns.page_start),
  page_end: extract_chapter_detail(line, regex_patterns.page_end),
}));

const ch_str = chapter_details_obj_arr
  .map((x) => `${x.number}: ${x.title}, pages:  ${x.page_start}-${x.page_end}`)
  .join(new_line);

tR += ch_str
tR += "";
%>