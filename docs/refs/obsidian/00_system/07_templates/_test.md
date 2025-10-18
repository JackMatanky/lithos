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
// SET BIRTHDAY
//-------------------------------------------------------------------
let date = await tp.user.suggester_date(tp);


//-------------------------------------------------------------------
// AUTHOR CONTACT FILE NAME AND RETRIEVE ALIAS
//-------------------------------------------------------------------
const author_yaml = book_file_cache?.frontmatter?.author;

let author_value_link = null_yaml_li;
if (!null_arr.includes(author_yaml) && typeof author_yaml != "undefined") {
  author_arr = author_yaml.toString().split(",");
  author_value_link = author_arr
    .map((author) => yaml_li(author))
    .join("");
}

const author_file_name_arr = author_yaml.map((file) =>
  file.toString().replaceAll(regex_link_to_value, "")
);
if (author_file_name_arr.length >= 3 || author_file_name_arr.length == 1) {
  author_file_name = author_file_name_arr[0];
  author_file_path = `${contacts_dir}${author_file_name}.md`;
  author_tfile = await app.vault.getAbstractFileByPath(author_file_path);
  author_file_cache = await app.metadataCache.getFileCache(author_tfile);
  if (author_file_name_arr.length == 1) {
    author_name_last = author_file_cache?.frontmatter?.name_last;
  } else {
    author_name_last = `${author_file_cache?.frontmatter?.name_last} et al`;
  }
} else if (author_file_name_arr.length >= 2) {
  author1_file_name = author_file_name_arr[0];
  author1_file_path = `${contacts_dir}${author1_file_name}.md`;
  author1_tfile = await app.vault.getAbstractFileByPath(author1_file_path);
  author1_file_cache = await app.metadataCache.getFileCache(author1_tfile);
  author1_last_name = author1_file_cache?.frontmatter?.name_last;
  author2_file_name = author_file_name_arr[1];
  author2_file_path = `${contacts_dir}${author2_file_name}.md`;
  author2_tfile = await app.vault.getAbstractFileByPath(author2_file_path);
  author2_file_cache = await app.metadataCache.getFileCache(author2_tfile);
  author2_last_name = author2_file_cache?.frontmatter?.name_last;
  author_name_last = `${author1_last_name}'s and ${author2_last_name}`;
}

const author_last_name = author_name_last;
const author_last_name_value = author_last_name
  .replace(/'s\sand/g, "")
  .replaceAll(/\s/g, "_");

const pillars_dir = "20_pillars/";
const { value, link } = await tp.user.multi_suggester({
  tp,
  items: await tp.user.md_file_name_alias(pillars_dir),
  prompt: "Select Pillar(s)",
  type: "pillar",
});

tR += `Pillar Value(s): ${value}\n`;
tR += `Pillar YAML:\n${link}`;


tR += date;
tR += "";
tR += yaml_proj;
%>
# <%* tR += full_title_name %>

<%* tR += proj_info %>
<%* tR += proj_sections_content %>