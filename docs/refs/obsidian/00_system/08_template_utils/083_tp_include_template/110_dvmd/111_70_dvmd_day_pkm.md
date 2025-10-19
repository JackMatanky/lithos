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
const snake_case_fmt = (name) =>
  name.replaceAll(/(\-\s\-)|(\s)|(\-)]/g, "_").toLowerCase();

const call_tbl_div = (int) =>
  call_tbl_start + Array(int).fill(tbl_cent).join(tbl_pipe) + call_tbl_end;

//-------------------------------------------------------------------
// RETRIEVE CURRENTLY ACTIVE FILE METADATA CACHE
//-------------------------------------------------------------------
const current_file = this.app.workspace.getActiveFile();
const current_file_name = current_file.name;
const tfile = tp.file.find_tfile(current_file_name);
const file_cache = await app.metadataCache.getFileCache(tfile);

/* ---------------------------------------------------------- */
/*                       DATE VARIABLES                       */
/* ---------------------------------------------------------- */
const yaml_date = file_cache?.frontmatter?.date;
const date = moment(yaml_date).format("YYYY-MM-DD");

//-------------------------------------------------------------------
// WEEK PKM SUBFILE DETAILS
//-------------------------------------------------------------------
const full_name = "Knowledge Management";
const name = "PKM";
const value = "pkm";

const file_name = `${date}_${value}`;
const file_section = file_name + hash;

//-------------------------------------------------------------------
// DAILY PKM OBJECT ARRAY
//-------------------------------------------------------------------
const file_obj_arr = [
  {
    head_key: "Knowledge Tree",
    toc_key: "Tree",
    type: "tree",
  },
  {
    head_key: "Permanent",
    toc_key: "Permanent",
    type: "permanent",
  },
  {
    head_key: "Literature",
    toc_key: "Literature",
    type: "literature",
  },
  {
    head_key: "Fleeting",
    toc_key: "Fleeting",
    type: "fleeting",
  },
  {
    head_key: "General",
    toc_key: "General",
    type: "info",
  },
];

file_obj_arr.map((x) => (x.head = head_lvl(4) + x.head_key));

// WEEK PKM SUBFILE TABLE OF CONTENTS
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.file.frontmatter.aliases[0],${space}"Contents")${backtick}`;
const toc_title = `${call_start}[!toc]${space}Week${space}${name}${space}${dv_content_link}`;

const toc_high =
  call_tbl_start +
  file_obj_arr
    .map((x) => `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
    .join(tbl_pipe) +
  call_tbl_end;

const toc = [toc_title, call_start, toc_high, call_tbl_div(5)].join(new_line);

// WEEK PKM SUBFILE DATAVIEW QUERIES
for (let i = 0; i < file_obj_arr.length; i++) {
  file_obj_arr[i].query = await tp.user.dv_pkm_type_status_dates({
    type: file_obj_arr[i].type,
    status: "",
    start_date: date,
    end_date: "",
    md: "true",
  });
}

file_obj_arr.map(
  (x) => (x.content = [x.head, toc, x.query].join(two_new_line))
);

const day_file = file_obj_arr.map((x) => x.content).join(two_new_line);

tR += day_file;
%>
