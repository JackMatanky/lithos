<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const goals_dir = "30_goals/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";
const library_dir = "60_library/";
const lib_video_dir = "60_library/66_video/";

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
const library_status = "60_library_status";

const related_task_sect = "100_40_related_task_sect_general";
const related_dir_sect = "100_50_related_dir_sect";
const related_lib_sect = "100_60_related_lib_sect";
const related_pkm_sect = "100_70_related_pkm_sect";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE, AND FILE CLASS
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// GENERAL VARIABLES
//-------------------------------------------------------------------
const null_arr = ["", "null", "[[null|Null]]", null];
const null_link = "[[null|Null]]";
const null_yaml_li = yaml_li(null_link);

/* ---------------------------------------------------------- */
/*                     FILE TYPE AND CLASS                    */
/* ---------------------------------------------------------- */
const full_type_name = "YouTube Video";
const class_name = full_type_name.split(" ")[1];
const class_value = class_name.toLowerCase();
const type_name = full_type_name.split(" ")[0];
const type_value = type_name.toLowerCase();
const file_class = `lib_${class_value}`;

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt(`${full_type_name} Title`, null, true, false);
} else {
  title = tp.file.title;
}
title = title.replaceAll(/&/g, "and").trim();
title = await tp.user.title_case(title);

//-------------------------------------------------------------------
// CONTENT TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const lib_content_titles = await tp.user.lib_content_titles(title);
const full_title_name = lib_content_titles.full_title_name;
const short_title_name = full_title_name.toLowerCase();
const short_title_value = lib_content_titles.full_title_value;
const main_title = lib_content_titles.main_title;
const short_main_title = main_title.toLowerCase();
const subtitle = lib_content_titles.sub_title;

const alias_arr = [
  full_title_name,
  short_title_name,
  main_title,
  short_main_title,
  short_title_value,
];
let file_alias = "";
for (let i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
}

const file_name = short_title_value;
const file_section = `${file_name}${hash}`;

//-------------------------------------------------------------------
// SET AUTHOR'S CONTACT FILE NAME AND TITLE
//-------------------------------------------------------------------
const contact_name_alias = await tp.user.include_template(
  tp,
  "51_contact_name_alias"
);
const contact_value = contact_name_alias.split(";")[0];
const contact_name = contact_name_alias.split(";")[1];
const contact_value_link = contact_name_alias.split(";")[2];

//-------------------------------------------------------------------
// SET YOUTUBE PAGE ORGANIZATION FILE NAME AND TITLE
//-------------------------------------------------------------------
const bool_obj_arr = [
  { key: "✔️ YES ✔️", value: "yes" },
  { key: "❌ NO ❌", value: "no" },
];
let bool_obj = await tp.system.suggester(
  (item) => item.key,
  bool_obj_arr,
  false,
  "View the creator's or all organizations for YouTube page?"
);

const organization_obj_arr = await tp.user.md_file_name_alias(organizations_dir);

let organization_value_link;
if (bool_obj.value == "yes") {
  temp_file_path = `${contacts_dir}${contact_value}.md`;
  abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
  file_cache = await app.metadataCache.getFileCache(abstract_file);
  let yaml_organization = file_cache?.frontmatter?.organization;
  if (
    !null_arr.includes(yaml_organization) &&
    typeof yaml_organization != "undefined"
  ) {
    yaml_organization.toString().split(",");
    yaml_organization.map((org) => org.split("|").slice(2));
  }
  organization_obj = await tp.system.suggester(
    (item) => item.key,
    organization_obj_arr.filter(
      (file) => yaml_organization.includes(file.value)
    ),
    false,
    "YouTube Page?"
  )
  organization_link = `[[${organization_obj.value}|${organization_obj.key}]]`;
  organization_value_link = yaml_li(organization_link);
} else {
  temp_file_path = `${sys_temp_include_dir}${org_name_alias}.md`;
  abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
  tp_include = await tp.file.include(abstract_file);
  include_arr = tp_include.toString().split(";");
  organization_value = include_arr[0];
  organization_value_link = include_arr[1];
}

const youtube_value_link = `${new_line}${ul_yaml}"[[${type_value}|${type_name}]]"`;
if (organization_value_link != null_yaml_li) {
  organization_value_link = `${organization_value_link}${youtube_value_link}`;
} else {
  organization_value_link = youtube_value_link;
}

//-------------------------------------------------------------------
// SET URL
//-------------------------------------------------------------------
const url = await tp.system.prompt("YouTube Video URL?");
const video_embed = `![${full_title_name}](${url})`;

//-------------------------------------------------------------------
// SET YOUTUBE PLAYLIST
//-------------------------------------------------------------------
let series_video = await tp.system.suggester(
  (item) => item.key,
  bool_obj_arr,
  false,
  "Is the video a part of a playlist?"
);
let series_value_link;
let series_url;
if (series_video == "yes") {
  series_name = await tp.system.prompt("Playlist Name?");
  series_value = series_name
    .replaceAll(/[,']/g, "")
    .replaceAll(/\s/g, "_")
    .replaceAll(/\//g, "-")
    .replaceAll(/&/g, "and")
    .toLowerCase();
  series_value_link = `${new_line}${ul_yaml}"[[${series_value}|${series_name}]]"`;
  series_url = await tp.system.prompt("Playlist URL?");
}

//-------------------------------------------------------------------
// SET DATE PUBLISHED
//-------------------------------------------------------------------
let date_published = await tp.user.suggester_date(tp);

//-------------------------------------------------------------------
// SET LIBRARY STATUS
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${library_status}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const status_value = include_arr[0];
const status_name = include_arr[1];

//-------------------------------------------------------------------
// VIDEO SECTION
//-------------------------------------------------------------------
heading = "Video";
const head_vid_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_vid_sect = `[[${file_section}${heading}\\|Video]]`;

heading = "Embed";
const head_vid_embed = `${head_lvl(3)}${heading}${two_new_line}`;

heading = "Timestamp Notes";
const head_vid_notes = `${head_lvl(3)}${heading}${two_new_line}`;
const comment_vid_notes = `${cmnt_html_start}Timestamp hotkey: ${backtick}Ctrl + Alt + T${backtick}${cmnt_html_end}`;
const code_block_vid_notes = `${three_backtick}timestamp-url${new_line}${url}${new_line}${three_backtick}`;

const vid_section = `${head_vid_embed}${video_embed}${two_new_line}${head_vid_notes}${comment_vid_notes}${two_new_line}${code_block_vid_notes}${two_new_line}${hr_line}${new_line}`;
//-------------------------------------------------------------------
// RELATED PKM SECTION
//-------------------------------------------------------------------
heading = "Related Knowledge";
const head_pkm_sect = `${head_lvl(2)}${heading}${two_new_line}`;
const toc_pkm_sect = `[[${file_section}${heading}\\|PKM]]`;

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
const toc_lib_sect = `[[${file_section}${heading}\\|Library]]`;

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
const toc_task_sect = `[[${file_section}${heading}\\|Tasks]]`;

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
const toc_dir_sect = `[[${file_section}${heading}\\|Directory]]`;

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

const toc_body_high = `${call_tbl_start}${toc_vid_sect}${tbl_pipe}${toc_related_pkm_sect}${tbl_pipe}${toc_related_lib_sect}${tbl_pipe}${toc_related_task_sect}${tbl_pipe}${toc_related_dir_sect}${call_tbl_end}${new_line}`;
const toc_body_div = `${call_tbl_start}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${tbl_pipe}${tbl_cent}${call_tbl_end}`;
const toc_body = `${toc_body_high}${toc_body_div}`;

const toc = `${toc_title}${toc_body}${two_new_line}`;

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const video = `${head_vid_sect}${toc}${vid_section}`;
const related_pkm = `${head_related_pkm_sect}${toc}${related_pkm_section}`;
const related_lib = `${head_related_lib_sect}${toc}${related_lib_section}`;
const related_task = `${head_related_task_sect}${toc}${related_task_section}`;
const related_dir = `${head_related_dir_sect}${toc}${related_dir_section}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = lib_video_dir;
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
date_published: <%* tR += date_published %>
publisher: <%* tR += organization_value_link %>
series: <%* tR += series_value_link %>
series_url: <%* tR += series_url %>
url: <%* tR += url %>
cssclasses: null
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>

# <%* tR += full_title_name %>

> [!<%* tR += class_value %> ] <%* tR += class_name %> Details
>
> - **Title**: `dv: choice(regextest("\w", this.file.frontmatter.url), elink(this.file.frontmatter.url, this.file.frontmatter.title), this.file.frontmatter.title)`
> - **Author**: `dv: this.file.frontmatter.author`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Series**: `dv: choice((regextest(".", this.file.frontmatter.series) AND regextest(".", this.file.frontmatter.series_url)), this.file.frontmatter.series + ", " + elink(this.file.frontmatter.series_url, "link"), choice(regextest(".", this.file.frontmatter.series), this.file.frontmatter.series, ""))`
> - **Date Published**: `dv: this.file.frontmatter.year_published`
>
> Completed::

---

<%* tR += video %>
<%* tR += related_pkm %>
<%* tR += related_lib %>
<%* tR += related_task %>
<%* tR += related_dir %>
