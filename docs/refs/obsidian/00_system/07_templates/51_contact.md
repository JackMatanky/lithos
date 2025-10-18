<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const contacts_dir = "51_contacts/";
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
const type_name = "Contact";
const type_value = type_name.toLowerCase();
const file_class = `dir_${type_value}`;

/* ---------------------------------------------------------- */
/*     SET CONTACT NAMES, ALIAS, FILE NAME, AND FILE CLASS    */
/* ---------------------------------------------------------- */
const {
  fullName: full_name,
  firstName: name_first,
  lastName: name_last,
  maidenName: name_last_maiden,
  surnamePrefix: surname_prefix,
  lastFirstName: name_last_first
} = await tp.user.dirContactNames(tp);

const file_name = name_last_first
  .replaceAll(/,/g, "")
  .replaceAll(/[^\w]/g, "_")
  .toLowerCase();
const file_section = file_name + hash;

const file_alias =
  new_line +
  [full_name, name_last_first, file_name]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = file_name;
} else {
  title = tp.file.title;
}
title = title.trim();

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
  { key: "Army", value: "army" },
  { key: "Book", value: "book" },
  { key: "Contact (Include Link)", value: "contact" },
  { key: "Course", value: "course" },
  { key: "Family", value: "family" },
  { key: "Garin Tsabar", value: "garin_tsabar" },
  { key: "Internet", value: "internet" },
  { key: "Job Application", value: "job_application" },
  { key: "LinkedIn", value: "[[linkedin|LinkedIn]]" },
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
}

//-------------------------------------------------------------------
// SET GENDER
//-------------------------------------------------------------------
const gender_obj_arr = [
  { key: "Female", value: "female" },
  { key: "Male", value: "male" },
  { key: "Other", value: "other" },
];
const gender_obj = await tp.system.suggester(
  (item) => item.key,
  gender_obj_arr,
  false,
  `Gender?`
);
const gender_name = gender_obj.key;
const gender_value = gender_obj.value;

//-------------------------------------------------------------------
// SET BIRTHDAY
//-------------------------------------------------------------------
let date_birth = await tp.user.suggester_date(tp);

//-------------------------------------------------------------------
// SET COUNTRY AND CITY
//-------------------------------------------------------------------
const location = await tp.user.suggester_location({
  tp,
  country: true,
  city: true,
  utc_dst: true,
});
const country_name = location.country_key;
const country_value = location.country_value;
const city_name = location.city_key;
const city_value = location.city_value;
const utc_value = location.utc;
const dst_value = location.dst;

//-------------------------------------------------------------------
// SET ADDRESS
//-------------------------------------------------------------------
const address = await tp.system.prompt("Home Address?", null, false, false);

//-------------------------------------------------------------------
// SET MOBILE, HOME, AND WORK PHONE NUMBERS
//-------------------------------------------------------------------
const number_regex = /\d/g;

async function phone_number(type) {
  const phone = await tp.system.prompt(
    `${type} Phone Number?`,
    null,
    false,
    false
  );
  if (!phone) {
    return "";
  }
  const clean_number = phone.match(number_regex).join("");
  const phone_length = clean_number.length;
  if (phone_length == 10) {
    const phone_slice_0_3 = clean_number.slice(0, 3);
    const phone_slice_3_6 = clean_number.slice(3, 6);
    const phone_slice_6 = clean_number.slice(6);
    return [phone_slice_0_3, phone_slice_3_6, phone_slice_6].join("-");
  } else if (phone_length == 9) {
    const phone_slice_0_2 = clean_number.slice(0, 2);
    const phone_slice_2_5 = clean_number.slice(2, 5);
    const phone_slice_5 = clean_number.slice(5);
    return [phone_slice_0_2, phone_slice_2_5, phone_slice_5].join("-");
  }
}

const phone_mobile = await phone_number("Mobile");
const phone_home = await phone_number("Home");
const phone_work = await phone_number("Work");

//-------------------------------------------------------------------
// SET PERSONAL AND WORK EMAIL
//-------------------------------------------------------------------
const email_personal = (
  await tp.system.prompt("Personal email?", null, false, false)
).toLowerCase();

const email_work = (
  await tp.system.prompt("Work email?", null, false, false)
).toLowerCase();

//-------------------------------------------------------------------
// SET LINKEDIN URL
//-------------------------------------------------------------------
const linkedin_url = await tp.system.prompt(
  "LinkedIn URL? (CTRL + L: Search Bar Shortcut)",
  null,
  false,
  false
);
const linkedin_elink = `[${full_name} LinkedIn](${linkedin_url})`;

//-------------------------------------------------------------------
// SET WEBSITE URL
//-------------------------------------------------------------------
const website_url = await tp.system.prompt(
  "Website URL? (CTRL + L: Search Bar Shortcut)",
  null,
  false,
  false
);
const website_elink = `[${full_name} Website](${website_url})`;

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const org_name_alias = await tp.user.include_template(
  tp,
  "52_organization_name_alias"
);
const organization_value = org_name_alias.split(";")[0];
const organization_value_link = org_name_alias.split(";")[1];

//-------------------------------------------------------------------
// SET JOB TITLE
//-------------------------------------------------------------------
const job_title_obj = await tp.user.suggester_job_title(tp);
const job_title_name = job_title_obj.key;
const job_title_value = job_title_obj.value;
const job_title_value_link = yaml_li(
  `[[${job_title_value}|${job_title_name}]]`
);

//-------------------------------------------------------------------
// SET CONTACT PROFILE PICTURE
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
//   "Contact profile picture source?"
// );

// let picture_path = picture_obj.value;
// const picture_files = await tp.user.vault_file(sys_atch_contacts_dir);

// if (picture_path == "vault") {
//   picture_path = await tp.system.suggester(
//     picture_files,
//     picture_files,
//     false,
//     "Contact profile picture file?"
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
    head_key: "Work Experience",
    toc_level: 1,
    toc_key: "Experience",
    file: null,
  },
  {
    head_key: "Education",
    toc_level: 1,
    toc_key: "Education",
    file: null,
  },
  {
    head_key: "Related Directory",
    toc_level: 1,
    toc_key: "Directory",
    file: "100_50_related_dir_sect",
  },
  {
    head_key: "Related Tasks and Events",
    toc_level: 2,
    toc_key: "Related Tasks",
    file: "100_40_related_task_sect_general",
  },
  {
    head_key: "Related Knowledge",
    toc_level: 2,
    toc_key: "PKM",
    file: "100_70_related_pkm_sect",
  },
  {
    head_key: "Related Library Content",
    toc_level: 2,
    toc_key: "Library",
    file: "100_60_related_lib_sect",
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
}
section_obj_arr.map((x) => (x.head = head_lvl(2) + x.head_key));
section_obj_arr.map(
  (x) => (x.toc = `[[${file_section}${x.head_key}\\|${x.toc_key}]]`)
);

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const toc_title = `${call_start}[!toc]${space}${dv_content_link}`;

const toc_lvl = (int) =>
  call_tbl_start +
  section_obj_arr
    .filter((x) => x.toc_level == int)
    .map((x) => x.toc)
    .join(tbl_pipe) +
  call_tbl_end;

const toc_body_div =
  call_tbl_start + Array(3).fill(tbl_cent).join(tbl_pipe) + call_tbl_end;

const toc = [toc_title, call_start, toc_lvl(1), toc_body_div, toc_lvl(2)].join(
  new_line
);

//-------------------------------------------------------------------
// WORK EXPERIENCE AND EDUCATION SECTIONS
//-------------------------------------------------------------------
section_obj_arr[0].content = `${cmnt_html_start}Copy experience details from LinkedIn here${cmnt_html_end}${two_new_line}${hr_line}${new_line}`;

section_obj_arr[1].content = `${cmnt_html_start}Copy education details from LinkedIn here${cmnt_html_end}${two_new_line}${hr_line}${new_line}`;

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info = await tp.user.include_file("51_contact_info_callout");

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = contacts_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
  await tp.file.move(`${directory}${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
name_first: <%* tR += name_first %>
name_last: <%* tR += name_last %>
name_last_maiden: <%* tR += name_last_maiden %>
gender: <%* tR += gender_value %>
date_birth: <%* tR += date_birth %>
date_death:
country: <%* tR += country_value %>
city: <%* tR += city_value %>
address: <%* tR += address %>
utc: <%* tR += utc_value %>
dst: <%* tR += dst_value %>
phone_mobile: <%* tR += phone_mobile %>
phone_home: <%* tR += phone_home %>
phone_work: <%* tR += phone_work %>
email_personal: <%* tR += email_personal %>
email_work: <%* tR += email_work %>
url: <%* tR += website_url %>
linkedin_url: <%* tR += linkedin_url %>
organization: <%* tR += organization_value_link %>
job_title: <%* tR += job_title_value_link %>
connection: <%* tR += connection_value_link %>
source: <%* tR += source_value_link %>
picture_path: <%* tR += picture_path %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_name %>

![[<%* tR += picture_path %>|200]]

<%* tR += info %>
<%* tR += sections_content %>