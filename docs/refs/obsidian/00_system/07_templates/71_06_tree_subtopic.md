<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const contacts_dir = "51_contacts/";
const organizations_dir = "52_organizations/";
const library_dir = "60_library/";
const pkm_dir = "70_pkm/";

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
const null_obj = {
  index: 0,
  key: "Null",
  value: "null",
  value_link: null_yaml_li,
};

const dv_yaml = "file.frontmatter";
const dv_content_link = `${backtick}dv:${space}link(this.file.name${space}+${space}"#"${space}+${space}this.${dv_yaml}.aliases[0],${space}"Contents")${backtick}`;

//-------------------------------------------------------------------
// PKM TREE BUTTON
//-------------------------------------------------------------------
const button_start = `${three_backtick}button`;
const button_end = `${three_backtick}${two_new_line}`;
const button_comment = `${cmnt_html_start}Adjust replace lines${cmnt_html_end}`;

const button_obj_arr = [
  {
    name: "name 🗃️PKM Tree Info",
    type: "type append template",
    action: "action 170_00_dvmd_pkm_tree_info",
    replace: "replace [54, 86]",
    color: "color purple",
  },
  {
    name: "name 🗃️PKM Tree Context",
    type: "type append template",
    action: "action 170_01_dvmd_pkm_tree_context",
    replace: "replace [103, 209]",
    color: "color purple",
  },
];

const button_arr = button_obj_arr.map(
  (b) =>
    (b.replace
      ? [button_start, b.name, b.type, b.action, b.replace, b.color, button_end]
      : [button_start, b.name, b.type, b.action, b.color, button_end]
    ).join(new_line) + button_comment
);

const pkm_info_button = button_arr[0];
const pkm_context_button = button_arr[1];

//-------------------------------------------------------------------
// TYPE, SUBTYPE, AND FILE CLASS
//-------------------------------------------------------------------
const type_name = "Subtopic";
const type_value = type_name.toLowerCase();
const file_class = "pkm_tree";

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
const has_title = !tp.file.title.startsWith("Untitled");
let title;
if (!has_title) {
  title = await tp.system.prompt(
    `Title for PKM Tree ${type_name}`,
    null,
    true,
    false
  );
} else {
  title = tp.file.title;
}
title = title.trim();
title = await tp.user.title_case(title);

/* ---------------------------------------------------------- */
/*          FRONTMATTER TITLE, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
const full_title_name = title;
const short_title_name = title.toLowerCase();
const short_title_value = short_title_name.replaceAll(/\s/g, "_");

const file_alias =
  new_line +
  [full_title_name, short_title_name, short_title_value]
    .map((x) => `${ul_yaml}"${x}"`)
    .join(new_line);

const file_name = short_title_value;
const file_section = file_name + hash;

const file_dir = `${pkm_dir}${file_name}`;

/* ---------------------------------------------------------- */
/*                       SET DESCRIPTION                      */
/* ---------------------------------------------------------- */
const about = await tp.system.prompt(
  `${type_name} Description?`,
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
  .replaceAll(/(<new_line>)(-\s|\d\.\s)/g, "\n $2");

/* ---------------------------------------------------------- */
/*                      SET REFERENCE URL                     */
/* ---------------------------------------------------------- */
const url = await tp.system.prompt(`${type_name} Reference URL?`);

/* ---------------------------------------------------------- */
/*   SET PILLAR FILE NAME AND TITLE; PRESET KNOW. EXPANSION   */
/* ---------------------------------------------------------- */
const pillar_name_alias = await tp.user.include_template(
  tp,
  "20_02_pillar_name_alias_preset_know"
);
const pillar_value = pillar_name_alias.split(";")[0];
const pillar_value_link = pillar_name_alias.split(";")[1];

/* ---------------------------------------------------------- */
/*                      SET RELATED GOAL                      */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for PKM Tree ${type_name}?`
);
const goal_link = `${ul}[[${goal}]]`;

/* ---------------------------------------------------------- */
/*                       SET NOTE STATUS                      */
/* ---------------------------------------------------------- */
const note_status = await tp.user.include_template(tp, "80_note_status");
const status_name = note_status.split(";")[0];
const status_value = note_status.split(";")[1];

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const section_obj_arr = [
  {
    head_key: "Knowledge Tree Information",
    toc_level: 1,
    toc_key: "Tree Info",
    file: null,
  },
  {
    head_key: "Knowledge Context",
    toc_level: 1,
    toc_key: "Tree Context",
    file: null,
  },
  {
    head_key: "Related Notes",
    toc_level: 1,
    toc_key: "Notes",
    file: "100_70_related_pkm_lab_sect",
  },
  {
    head_key: "Related Library Content",
    toc_level: 2,
    toc_key: "Library",
    file: "100_61_related_lib_sect_related_file",
  },
  {
    head_key: "Related Goals",
    toc_level: 2,
    toc_key: "Goals",
    file: null,
  },
  {
    head_key: "Related Tasks and Events",
    toc_level: 2,
    toc_key: "Tasks",
    file: "100_41_related_task_sect_related_proj",
  },
  {
    head_key: "Related Directory",
    toc_level: 2,
    toc_key: "Directory",
    file: "100_50_related_dir_sect",
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

/* ---------------------------------------------------------- */
/*                        GOAL SECTION                        */
/* ---------------------------------------------------------- */
const goal_outgoing = [
  head_lvl(3) + "Outgoing Goals Links",
  `${cmnt_html_start}Link related goals here${cmnt_html_end}`,
].join(two_new_line);

const goal_value = head_lvl(3) + "Value Goals";
const goal_outcome = head_lvl(3) + "Outcome Goals";

section_obj_arr[4].content =
  [goal_outgoing, goal_link, goal_value, goal_outcome, hr_line].join(
    two_new_line
  ) + new_line;

/* ---------------------------------------------------------- */
/*              SET PKM TREE FILE NAMES AND LINKS             */
/* ---------------------------------------------------------- */
const pkm_tree_obj_arr = [
  { index: 5, key: "Topic", value: "topic" },
  { index: 4, key: "Subject", value: "subject" },
  { index: 3, key: "Field", value: "field" },
  { index: 2, key: "Branch", value: "branch" },
  { index: 1, key: "Category", value: "category" },
];
const pkm_type_obj_arr = [null_obj, pkm_tree_obj_arr].flat();

// SET KNOWLEDGE LEVEL
const pkm_type_obj = await tp.system.suggester(
  (item) => item.key,
  pkm_type_obj_arr,
  false,
  "Direct Knowledge Tree Level?"
);
const pkm_type_value = pkm_type_obj.value;
const pkm_type_name = pkm_type_obj.key;

// SET KNOWLEDGE TREE OBJECT NAME AND VALUE
let pkm_file_dir = "";
let pkm_file_cache = "";
let pkm_link = null_link;
if (pkm_type_value != "null") {
  const pkm_file_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: pkm_dir,
    file_class: "pkm_tree",
    type: pkm_type_value,
  });
  const pkm_file_obj = await tp.system.suggester(
    (item) => item.key,
    pkm_file_obj_arr,
    false,
    `${pkm_type_name}?`
  );
  pkm_link = `[[${pkm_file_obj.value}|${pkm_file_obj.key}]]`;

  // PKM METADATA CACHE
  const pkm_file_ext = `${pkm_file_obj.value}.md`;
  const pkm_file_path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${pkm_file_ext}`))
    .map((file) => file.path)[0];
  pkm_file_dir = pkm_file_path.replace(pkm_file_ext, "");

  const pkm_tfile = await app.vault.getAbstractFileByPath(pkm_file_path);
  pkm_file_cache = await app.metadataCache.getFileCache(pkm_tfile);
}
const pkm_value_link = yaml_li(pkm_link);

const tree_index = pkm_tree_obj_arr
  .filter((tree) => tree.value == pkm_type_value)
  .map((tree) => tree.index);

if (pkm_type_value != "null") {
  pkm_tree_obj_arr.filter((tree) => tree.index >= tree_index);
  for (let i = 0; i < pkm_tree_obj_arr.length; i++) {
    if (pkm_type_value == pkm_tree_obj_arr[i].value) {
      pkm_tree_obj_arr[i].value_link = pkm_value_link;
    } else {
      const pkm_yaml = pkm_file_cache?.frontmatter?.[pkm_tree_obj_arr[i].value];
      if (!null_arr.includes(pkm_yaml) && typeof pkm_yaml != "undefined") {
        pkm_tree_obj_arr[i].value_link = pkm_yaml
          .toString()
          .split(",")
          .map((tree_yaml) => yaml_li(tree_yaml))
          .join("");
      } else {
        pkm_tree_obj_arr[i].value_link = null_yaml_li;
      }
    }
  }
}
const pkm_tree_value_link = pkm_tree_obj_arr
  .map((tree) => tree.value_link)
  .reverse();

const category_value_link = pkm_tree_value_link[0];
const branch_value_link = pkm_tree_value_link[1];
const field_value_link = pkm_tree_value_link[2];
const subject_value_link = pkm_tree_value_link[3];
const topic_value_link = pkm_tree_value_link[4];
const subtopic_value_link = null_yaml_li;

//-------------------------------------------------------------------
// KNOWLEDGE OBJECT INFORMATION
//-------------------------------------------------------------------
const pkm_info_key_term = [
  head_lvl(3) + "Key Terms",
  `${cmnt_html_start}Link the ${type_value}'s key terms here${cmnt_html_end}`,
].join(two_new_line);
const pkm_info_gen_info = [
  head_lvl(3) + "General Information",
  `${cmnt_html_start}Link general info about the ${type_value} here${cmnt_html_end}`,
].join(two_new_line);

section_obj_arr[0].content =
  [pkm_info_button, pkm_info_key_term, pkm_info_gen_info, hr_line].join(
    two_new_line
  ) + new_line;

//-------------------------------------------------------------------
// KNOWLEDGE TREE CALLOUT
//-------------------------------------------------------------------
const tree_call_title = call_start + "[!tree] Knowledge Tree";

const dv_inline = "dv: this.file.frontmatter.";
const tree_hierarchy = ["Category", "Branch", "Field", "Subject", "Topic"].map(
  (x) =>
    `${space}**${x}**` +
    dv_colon +
    backtick +
    dv_inline +
    x.toLowerCase() +
    backtick
);

const tree_call_category = `${call_start}1.${tree_hierarchy[0]}`;
const tree_call_branch = `${call_start}2.${tree_hierarchy[1]}`;
const tree_call_field = `${call_start}3.${tree_hierarchy[2]}`;
const tree_call_subject = `${call_start}4.${tree_hierarchy[3]}`;
const tree_call_topic = `${call_start}5.${tree_hierarchy[3]}`;

const tree_callout = [
  tree_call_title,
  call_start,
  tree_call_category,
  tree_call_branch,
  tree_call_field,
  tree_call_subject,
  tree_call_topic,
].join(new_line);

//-------------------------------------------------------------------
// KNOWLEDGE CONTEXT
//-------------------------------------------------------------------
const query_pkm_parent = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: type_value,
  relation: "parent",
  md: "false",
});
const pkm_parent = [head_lvl(3) + "Knowledge Ancestors", query_pkm_parent].join(
  two_new_line
);

const query_pkm_sibling = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: type_value,
  relation: "sibling",
  md: "false",
});
const pkm_sibling = [
  head_lvl(3) + "Knowledge Siblings",
  query_pkm_sibling,
].join(two_new_line);

const query_pkm_unrelated = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: type_value,
  relation: "unrelated",
  md: "false",
});
const pkm_unrelated = [
  head_lvl(3) + "General PKM Tree Items",
  `${cmnt_html_start}Link general pkm tree items here${cmnt_html_end}`,
  query_pkm_unrelated,
].join(two_new_line);

section_obj_arr[1].content =
  [
    tree_callout,
    pkm_context_button,
    pkm_parent,
    pkm_sibling,
    pkm_unrelated,
    hr_line,
  ].join(two_new_line) + new_line;

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sections_content = section_obj_arr
  .map((s) => [s.head, toc, s.content].join(two_new_line))
  .join(new_line);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const folder_path = tp.file.folder(true);

let directory = file_dir;
const nested_directory_value = `${pkm_file_dir}${file_name}`;
const directory_obj_arr = [
  { key: "Standalone", value: file_dir },
  { key: "Nested", value: nested_directory_value },
];

if (pkm_file_dir) {
  const directory_obj = await tp.system.suggester(
    (item) => item.key,
    directory_obj_arr,
    false,
    "Standalone or Nested Directory?"
  );
  directory = directory_obj.value;
}

if (folder_path != directory) {
  await tp.file.move(`${directory}/${file_name}`);
}

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
pillar: <%* tR += pillar_value_link %>
category: <%* tR += category_value_link %>
branch: <%* tR += branch_value_link %>
field: <%* tR += field_value_link %>
subject: <%* tR += subject_value_link %>
topic: <%* tR += topic_value_link %>
subtopic: <%* tR += subtopic_value_link %>
library:
about: |-
 <%* tR += about_value %>
url: <%* tR += url %>
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_title_name %>

> [!tree] Knowledge Tree Details
>
> - **Name**: `dv: choice(regextest("\w", this.file.frontmatter.url), elink(this.file.frontmatter.url, this.file.frontmatter.aliases[0]), this.file.frontmatter.aliases[0])`
>
> - **Description**: `dv: this.file.frontmatter.about`

---

<%* tR += sections_content %>
