<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const sys_atch_organizations_dir =
  "00_system/02_attachments/_52_organizations/";
const organizations_dir = "52_organizations";

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
const contact_name_alias = "51_contact_name_alias";
const org_name_alias = "52_organization_name_alias";

const related_task_sect = "100_40_related_task_sect_general";
const related_dir_sect = "100_50_related_dir_sect";
const related_lib_sect = "100_60_related_lib_sect";
const related_pkm_sect = "100_70_related_pkm_sect";

//-------------------------------------------------------------------
// RELATED DIRECTORY SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_dir_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_dir_section = include_arr;

//-------------------------------------------------------------------
// RELATED TASKS AND EVENTS SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_task_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_task_section = include_arr;

//-------------------------------------------------------------------
// RELATED PKM SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_pkm_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_pkm_section = include_arr;

//-------------------------------------------------------------------
// RELATED LIBRARY SECTION
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_lib_sect}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();
const related_lib_section = include_arr;

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const type_name = "Organization";
const type_value = type_name.toLowerCase();
const file_class = `dir_${type_value}`;

const fmatter_type = `type:${space}${type_value}${new_line}`;
const fmatter_file_class = `file_class:${space}${file_class}${new_line}`;

const org_obj_arr = [{}];

let file_content;
let file_name;
const directory = organizations_dir;

// Loop through the array of objects
for (var i = 0; i < org_obj_arr.length; i++) {
  //-------------------------------------------------------------------
  // FILE CREATION AND MODIFIED DATE
  //-------------------------------------------------------------------
  date_created = org_obj_arr[i].date_created;
  date_modified = org_obj_arr[i].date_modified;

  fmatter_date_created = `date_created:${space}${date_created}${new_line}`;
  fmatter_date_modified = `date_modified:${space}${date_modified}${new_line}`;

  //-------------------------------------------------------------------
  // SET FILE'S TITLE
  //-------------------------------------------------------------------
  org_title = org_obj_arr[i].title;
  title = await tp.user.title_case(org_title);

  //-------------------------------------------------------------------
  // ORGANIZATION TITLES, ALIAS, AND FILE NAME
  //-------------------------------------------------------------------
  full_title_name = title;
  short_title_name = full_title_name.toLowerCase();
  short_title_value = short_title_name
    .replaceAll(/[,']|\.$/g, "")
    .replaceAll(/[\s\.\/\+:-]/g, "_")
    .replaceAll(/&/g, "and");

  alias_arr = [org_title, full_title_name, short_title_name, short_title_value];
  file_alias = "";
  for (let j = 0; j < alias_arr.length; j++) {
    alias = yaml_li(alias_arr[j]);
    file_alias += alias;
  }

  file_name = short_title_value;
  file_section = `${file_name}${hash}`;

  fmatter_title = `title:${space}${file_name}${new_line}`;
  fmatter_aliases = `aliases:${space}${file_alias}${new_line}`;

  //-------------------------------------------------------------------
  // SET LINKEDIN URL
  //-------------------------------------------------------------------
  linkedin_url = org_obj_arr[i].linkedin_url;
  if (linkedin_url.endsWith("about/")) {
    linkedin_url = linkedin_url.substring(0, linkedin_url.indexOf("about/"));
  }
  fmatter_linkedin_url = `linkedin_url:${space}${linkedin_url}${new_line}`;

  //-------------------------------------------------------------------
  // SET WEBSITE URL
  //-------------------------------------------------------------------
  website_url = org_obj_arr[i].url;
  fmatter_url = `url:${space}${website_url}${new_line}`;

  //-------------------------------------------------------------------
  // SET THE ORGANIZATION'S LINKEDIN ABOUT
  //-------------------------------------------------------------------
  about = org_obj_arr[i].about;
  about_value = about
    .replaceAll(/(\n\n)(\w)/g, "  $1  $2")
    .replaceAll(/(\n)(-\s)/g, "$1  $2");
  about_value = `${String.fromCodePoint(0x7c)}${new_line}${two_space}${about_value}`;
  fmatter_about = `about:${space}${about_value}${new_line}`;

  //-------------------------------------------------------------------
  // SET LINKEDIN INDUSTRY
  //-------------------------------------------------------------------
  industry_link = org_obj_arr[i].industry;
  industry_value_link = yaml_li(industry_link);
  fmatter_industry = `industry:${space}${industry_value_link}${new_line}`;
  
  //-------------------------------------------------------------------
  // SET LINKEDIN SPECIALTIES
  //-------------------------------------------------------------------
  org_specialties = org_obj_arr[i].specialties;
  specialties_name = "null";
  specialties_value = "null";
  specialties_tag = "null";

  if (org_specialties.match(/\w/g)) {
    specialties_name = await tp.user.title_case(org_specialties);
    specialties_value = specialties_name
      .replaceAll(/&/g, "and")
      .replace(/(\s|,\s)(and)\s([\w\s]+)$/g, ", $3")
      .toLowerCase();
    specialties_tag = specialties_value
      //.replace(/(\s|,\s)(and)\s([\w\s]+)$/g, " $3")
      .split(", ")
      .map((s) => s.replaceAll(/[\s-]/g, "_"))
      .join(" ")
      .toLowerCase();
  }

  fmatter_specialties = `specialties:${space}${specialties_value}${new_line}`;
  fmatter_tags = `tags:${space}${specialties_tag}${new_line}`;
  //-------------------------------------------------------------------
  // SET COUNTRY
  //-------------------------------------------------------------------
  country_value = org_obj_arr[i].country;
  fmatter_country = `country:${space}${country_value}${new_line}`;

  //-------------------------------------------------------------------
  // SET CITY
  //-------------------------------------------------------------------
  city_value = org_obj_arr[i].city;
  fmatter_city = `city:${space}${city_value}${new_line}`;

  //-------------------------------------------------------------------
  // SET EMAIL
  //-------------------------------------------------------------------
  email = org_obj_arr[i].email;
  email.toLowerCase();
  fmatter_email = `email:${space}${email}${new_line}`;

  //-------------------------------------------------------------------
  // SET EMAIL
  //-------------------------------------------------------------------
  phone = org_obj_arr[i].phone;

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
  fmatter_phone = `phone:${space}${phone}${new_line}`;

  //-------------------------------------------------------------------
  // SET MY CONNECTION WITH THE ORGANIZATION
  //-------------------------------------------------------------------
  connection_value = org_obj_arr[i].connection;
  fmatter_connection = `connection:${space}${connection_value}${new_line}`;
  
  //-------------------------------------------------------------------
  // SET CONNECTION SOURCE
  //-------------------------------------------------------------------
  source_value = yaml_li(org_obj_arr[i].source);
  fmatter_source = `source:${space}${source_value}${new_line}`;

  //-------------------------------------------------------------------
  // SET ORGANIZATION LOGO PICTURE
  //-------------------------------------------------------------------
  picture_path = `${file_name}_pic.jpg`;
  picture_path_embed = `![[${picture_path}|200]]`;
  fmatter_picture_path = `picture_path:${space}${picture_path}${new_line}`;
  //-------------------------------------------------------------------
  // RELATED DIRECTORY SECTION
  //-------------------------------------------------------------------
  heading = "Related Directory";
  head_related_dir_sect = `${head_lvl(2)}${heading}${two_new_line}`;
  toc_related_dir_sect = `[[${file_section}${heading}\\|Directory]]`;

  //-------------------------------------------------------------------
  // RELATED TASKS AND EVENTS SECTION
  //-------------------------------------------------------------------
  heading = "Related Tasks and Events";
  head_related_task_sect = `${head_lvl(2)}${heading}${two_new_line}`;
  toc_related_task_sect = `[[${file_section}${heading}\\|Tasks]]`;

  //-------------------------------------------------------------------
  // RELATED PKM SECTION
  //-------------------------------------------------------------------
  heading = "Related Knowledge";
  head_related_pkm_sect = `${head_lvl(2)}${heading}${two_new_line}`;
  toc_related_pkm_sect = `[[${file_section}${heading}\\|PKM]]`;

  //-------------------------------------------------------------------
  // RELATED LIBRARY SECTION
  //-------------------------------------------------------------------
  heading = "Related Library Content";
  head_related_lib_sect = `${head_lvl(2)}${heading}${two_new_line}`;
  toc_related_lib_sect = `[[${file_section}${heading}\\|Library]]`;

  //-------------------------------------------------------------------
  // TABLE OF CONTENTS CALLOUT
  //-------------------------------------------------------------------
  toc_title = `${call_start}[!toc]${space}[[${file_section}${full_title_name}|Contents]]${two_space}${new_line}${call_start}${new_line}`;

  toc_body_high = `${call_tbl_start}${toc_related_dir_sect}${tbl_pipe}${toc_related_task_sect}${tbl_pipe}${toc_related_pkm_sect}${tbl_pipe}${toc_related_lib_sect}${call_tbl_end}${new_line}`;
  toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}${new_line}`;
  toc_body = `${toc_body_high}${toc_body_div}`;

  toc = `${toc_title}${toc_body}${two_new_line}`;

  //-------------------------------------------------------------------
  // FILE SECTIONS
  //-------------------------------------------------------------------
  related_dir = `${head_related_dir_sect}${toc}${related_dir_section}`;
  related_task = `${head_related_task_sect}${toc}${related_task_section}`;
  related_pkm = `${head_related_pkm_sect}${toc}${related_pkm_section}`;
  related_lib = `${head_related_lib_sect}${toc}${related_lib_section}`;

  details_title = `${call_start}[!${type_value}]${space}${type_name}${space}Details${new_line}${call_start}${new_line}`;
  details_industry = `${call_start}**Industry**::${space}${backtick}dv:${space}this.file.frontmatter.industry${backtick}${two_space}${new_line}`;
  details_specialties = `${call_start}**Specialties**::${space}${specialties_name}${two_space}${new_line}`;
  details_about = `${call_start}**About**::${space}${backtick}dv:${space}this.file.frontmatter.about${backtick}${new_line}${call_start}${new_line}`;
  details_linkedin = `${call_start}**LinkedIn**::${space}${backtick}dv:${space}elink(this.file.frontmatter.linkedin_url, this.file.frontmatter.aliases[0] + " LinkedIn")${backtick}${two_space}${new_line}`;
  details_website = `${call_start}**Website**::${space}${backtick}dv:${space}elink(this.file.frontmatter.url, this.file.frontmatter.aliases[0] + " Website")${backtick}`;
  details = `${details_title}${details_industry}${details_specialties}${details_about}${details_linkedin}${details_website}${two_new_line}${hr_line}${new_line}`;

  frontmatter = `${hr_line}${new_line}${fmatter_title}${fmatter_aliases}${fmatter_country}${fmatter_city}${fmatter_phone}${fmatter_email}${fmatter_url}${fmatter_linkedin_url}${fmatter_about}${fmatter_industry}${fmatter_specialties}${fmatter_connection}${fmatter_source}${fmatter_picture_path}${fmatter_file_class}${fmatter_date_created}${fmatter_date_modified}${fmatter_tags}${hr_line}`;

  file_content = `${frontmatter}
${head_lvl(1)}${org_title}${new_line}
${picture_path_embed}${new_line}
${details}
${related_dir}
${related_task}
${related_pkm}
${related_lib}`;

  // Create subdirectory file
  await tp.file.create_new(
    file_content,
    file_name,
    false,
    app.vault.getAbstractFileByPath(directory)
  );
}
%>