<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const organizations_dir = "52_organizations/";

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
const type_name = "Organization";
const type_value = type_name.toLowerCase();
const file_class = `dir_${type_value}`;

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt("Organization Name?", null, true, false);
} else {
  title = tp.file.title;
}
const organization_title = title.trim();
title = await tp.user.title_case(organization_title);

//-------------------------------------------------------------------
// ORGANIZATION TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = title;
const short_title_name = full_title_name.toLowerCase();
const short_title_value = short_title_name
  .replaceAll(/[,']|\.$/g, "")
  .replaceAll(/[\s\.\/\+-]|(:\s)/g, "_")
  .replaceAll(/&/g, "and");

let alias_arr = [
  organization_title,
  full_title_name,
  short_title_name,
  short_title_value
];
if (organization_title == full_title_name) {
  alias_arr = [full_title_name, short_title_name, short_title_value];
};
const file_alias =
  new_line + alias_arr.map((x) => `${ul_yaml}"${x}"`).join(new_line);

const file_name = short_title_value;
const file_section = file_name + hash;

//-------------------------------------------------------------------
// SET LINKEDIN URL
//-------------------------------------------------------------------
let linkedin_url = await tp.system.prompt(
  "LinkedIn URL? (CTRL + L: Search Bar Shortcut)",
  null,
  false,
  false
);

if (linkedin_url.endsWith("about/")) {
  linkedin_url = linkedin_url.substring(0, linkedin_url.indexOf("about/"));
}

//-------------------------------------------------------------------
// SET WEBSITE URL
//-------------------------------------------------------------------
const website_url = await tp.system.prompt(
  "Website URL?",
  null,
  false,
  false
);

//-------------------------------------------------------------------
// SET THE ORGANIZATION'S LINKEDIN ABOUT
//-------------------------------------------------------------------
const about = await tp.system.prompt(
  "Organization's LinkedIn 'about'?",
  null,
  false,
  true
);
const about_value = about
  .replaceAll(/^(\s*)([^\s])/g, "$2")
  .replaceAll(/(\s*)\n/g, "\n")
  .replaceAll(/([^\s])(\s*)$/g, "$1")
  .replaceAll(/\n{1,6}/g, "<new_line>")
  .replaceAll(/(<new_line>)(\w)/g, "\n \n $2")
  .replaceAll(/(<new_line>)(\d\.\s)/g, "\n $2")
  .replaceAll(/<new_line>((•|·|\*|-)\s*)/g, "\n - ");

//-------------------------------------------------------------------
// SET LINKEDIN INDUSTRY
//-------------------------------------------------------------------
const linkedin_industry_obj = await tp.user.suggester_industry(tp);
const industry_name = linkedin_industry_obj.key;
const industry_value = linkedin_industry_obj.value;
const industry_link = `[[${industry_value}|${industry_name}]]`;
const industry_value_link = yaml_li(industry_link);

//-------------------------------------------------------------------
// SET LINKEDIN SPECIALTIES
//-------------------------------------------------------------------
const org_specialties = await tp.system.prompt(
  "Organization's specialties?",
  null,
  false,
  false
);

let specialties_value = "";
let specialties_tag = "";
if (org_specialties.match(/\w/g)) {
  specialties = (await tp.user.title_case(org_specialties))
    .replaceAll(/&/g, "and")
    .replace(/(\s|,\s)(and)\s([\w\s]+)$/g, ", $3");
  specialties_value = specialties
    .split(", ")
    .map((s) => yaml_li(s))
    .join("");
  specialties_tag = specialties
    .split(", ")
    .map((s) =>
      s.trim().replaceAll(/["\.]/g, "").replaceAll(/[\s-]/g, "_").toLowerCase()
    )
    .join(" ");
}

//-------------------------------------------------------------------
// SET COUNTRY AND CITY
//-------------------------------------------------------------------
const location = await tp.user.suggester_location({
  tp,
  country: true,
  city: true,
  utc_dst: false,
});
const country_name = location.country_key;
const country_value = location.country_value;
const city_name = location.city_key;
const city_value = location.city_value;

//-------------------------------------------------------------------
// SET EMAIL
//-------------------------------------------------------------------
const email = (
  await tp.system.prompt("Email?", null, false, false)
).toLowerCase();

//-------------------------------------------------------------------
// SET EMAIL
//-------------------------------------------------------------------
let phone = await tp.system.prompt(
  "Phone Number?",
  null,
  false,
  false
);

const number_regex = /\d/g;
if (phone.match(number_regex)) {
  clean_number = phone;
  phone_number = clean_number.match(number_regex).join("");
  phone_length = phone_number.length;
  if (phone_length == 10) {
    phone_slice_0_3 = phone_number.slice(0, 3);
    phone_slice_3_6 = phone_number.slice(3, 6);
    phone_slice_6 = phone_number.slice(6);
    phone = `${phone_slice_0_3}-${phone_slice_3_6}-${phone_slice_6}`;
  } else if (phone_length == 9) {
    phone_slice_0_2 = phone_number.slice(0, 2);
    phone_slice_2_5 = phone_number.slice(2, 5);
    phone_slice_5 = phone_number.slice(5);
    phone = `${phone_slice_0_2}-${phone_slice_2_5}-${phone_slice_5}`;
  }
}

//-------------------------------------------------------------------
// SET CONNECTION TYPE
//-------------------------------------------------------------------
const connection_obj_arr = [
  { key: "Null", value: "null" },
  { key: "Education", value: "education" },
  { key: "Personal", value: "personal" },
  { key: "Professional", value: "professional" },
  { key: "Work", value: "work" },
];
const connection_obj = await tp.system.suggester(
  (item) => item.key,
  connection_obj_arr,
  false,
  `Connection to the ${type_value}?`
);
const connection_name = connection_obj.key;
const connection_value = connection_obj.value;
const connection_value_link = yaml_li(connection_value);

//-------------------------------------------------------------------
// SET CONNECTION SOURCE
//-------------------------------------------------------------------
const source_obj_arr = [
  { key: "Null", value: "null" },
  { key: "Book", value: "book" },
  { key: "Contact (Include Link)", value: "contact" },
  { key: "Course", value: "course" },
  { key: "Family", value: "family" },
  { key: "Garin Tsabar", value: "garin_tsabar" },
  { key: "Internet", value: "internet" },
  { key: "Job Application", value: "job_application" },
  { key: "Organization (Include Link)", value: "organization" },
];
const source_obj = await tp.system.suggester(
  (item) => item.key,
  source_obj_arr,
  false,
  `Connection source to the ${type_value}?`
);
let source_name = source_obj.key;
let source_value = source_obj.value;
let source_value_link = yaml_li(source_value);

let temp_link;
if (source_value == "contact") {
  temp_link = await tp.user.include_template(tp, "51_contact_name_alias");
} else if (source_value == "organization") {
  temp_link = await tp.user.include_template(tp, "52_organization_name_alias");
}
if (temp_link) {
  source_value_link = temp_link.split(";")[1];
};

//-------------------------------------------------------------------
// SET ORGANIZATION LOGO PICTURE
//-------------------------------------------------------------------
// const picture_obj_arr = [
//   { key: "User Input", value: "_user_input" },
//   { key: "Vault", value: "vault" },
//   { key: "Null", value: "null" },
// ];

// const picture_obj = await tp.system.suggester(
//   (item) => item.key,
//   picture_obj_arr,
//   false,
//   "Organization logo source?"
// );

// let picture_path = picture_obj.value;
// const picture_files = await tp.user.vault_file(
//  sys_atch_organizations_dir
//);

// if (picture_path == "vault") {
//   picture_path = await tp.system.suggester(
//     picture_files,
//     picture_files,
//     false,
//     "Organization logo file?"
//   );
// } else if (picture_path == "_user_input") {
//   picture_path = `${file_name}_pic.jpg`;
// }

const picture_path = `${file_name}_pic.webp`;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: "Related Directory",
    toc_key: "Directory",
    file: "100_50_related_dir_sect",
  },
  {
    head_key: "Related Tasks and Events",
    toc_key: "Related Tasks",
    file: "100_40_related_task_sect_general",
  },
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

const toc =
  [toc_title, call_start, toc_body_high, toc_body_div].join(new_line);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info = await tp.user.include_file("52_organization_info_callout");

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = organizations_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
country: <%* tR += country_value %>
city: <%* tR += city_value %>
phone: <%* tR += phone %>
email: <%* tR += email %>
url: <%* tR += website_url %>
linkedin_url: <%* tR += linkedin_url %>
about: |
 <%* tR += about_value %>
industry: <%* tR += industry_value_link %>
specialties: <%* tR += specialties_value %>
connection: <%* tR += connection_value_link %>
source: <%* tR += source_value_link %>
picture_path: <%* tR += picture_path %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags: <%* tR += specialties_tag %>
<%* tR += hr_line %>
# <%* tR += organization_title %>

![[<%* tR += picture_path %>|200]]

<%* tR += info %>
<%* tR += sections_content %>