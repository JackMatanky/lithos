---
title: pkm_tree_name_link
aliases:
  - Knowledge Tree File Name and Link Suggester
  - knowledge tree file name and link suggester
  - Knowledge Tree File Name and Link
  - knowledge tree file name and link
  - pkm tree name link
plugin: templater
language:
  - javascript
module:
  - system
  - file
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-06-20T13:15
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript, obsidian/tp/system/suggester, obsidian/tp/file/include
---
# Knowledge Tree File Name and Link Suggester

## Description

> [!snippet] Snippet Details
>
> Plugin: [[Templater]]
> Language: [[JavaScript]]
> Input:: Object Array
> Output:: String
> Description:: Return the knowledge tree objects' name and link from a suggester.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// Template file to include
const pkm_tree_name_link = "70_pkm_tree_name_link";

//---------------------------------------------------------
// SET PKM TREE FILE NAMES AND LINKS
//---------------------------------------------------------
// Retrieve the Knowledge Tree Names and Links template and content
temp_file_path = `${sys_temp_include_dir}${pkm_tree_name_link}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pkm_file_dir = include_arr[0];
const category_value_link = include_arr[1];
const branch_value_link = include_arr[2];
const field_value_link = include_arr[3];
const subject_value_link = include_arr[4];
const topic_value_link = include_arr[5];
const subtopic_value_link = include_arr[6];
```

### Templater

<!-- Add the full code as it appears in the template  -->
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// SET PKM TREE FILE NAMES AND LINKS
//---------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pkm_tree_name_link}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pkm_file_dir = include_arr[0];
const category_value_link = include_arr[1];
const branch_value_link = include_arr[2];
const field_value_link = include_arr[3];
const subject_value_link = include_arr[4];
const topic_value_link = include_arr[5];
const subtopic_value_link = include_arr[6];
```

#### Referenced Template

<!-- If applicable, add the referenced template  -->

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const space = String.fromCodePoint(0x20);
const new_line = String.fromCodePoint(0xa);
const ul_yaml = `${space.repeat(2)}${String.fromCodePoint(0x2d)}${space}`;

//---------------------------------------------------------
// PKM TREE FILES AND NAMES
//---------------------------------------------------------
// Knowledge Tree directory
const pkm_dir = "70_pkm/";

// INITIALIZE VALUES
let pkm_file_dir = "";
let category_link = "[[null|Null]]";
let category_value_link = `${new_line}${ul_yaml}"${category_link}"`;
let branch_link = "[[null|Null]]";
let branch_value_link = `${new_line}${ul_yaml}"${branch_link}"`;
let field_link = "[[null|Null]]";
let field_value_link = `${new_line}${ul_yaml}"${field_link}"`;
let subject_link = "[[null|Null]]";
let subject_value_link = `${new_line}${ul_yaml}"${subject_link}"`;
let topic_link = "[[null|Null]]";
let topic_value_link = `${new_line}${ul_yaml}"${topic_link}"`;
let subtopic_link = "[[null|Null]]";
let subtopic_value_link = `${new_line}${ul_yaml}"${subtopic_link}"`;

const pkm_type_obj_arr = [
  { key: "Null", value: "null" },
  { key: "Subtopic", value: "subtopic" },
  { key: "Topic", value: "topic" },
  { key: "Subject", value: "subject" },
  { key: "Field", value: "field" },
  { key: "Branch", value: "branch" },
  { key: "Category", value: "category" },
];

const pkm_type_obj = await tp.system.suggester(
  (item) => item.key,
  pkm_type_obj_arr,
  false,
  "Direct Knowledge Tree Level?"
);

const pkm_type_value = pkm_type_obj.value;
const pkm_type_name = pkm_type_obj.key;

if (pkm_type_value != "null") {
  // SET KNOWLEDGE TREE OBJECT NAME AND VALUE
  const pkm_obj_arr = await tp.user.file_name_alias_by_class_type_subtype({
    dir: pkm_dir,
    file_class: "pkm",
    type: "tree",
    subtype: pkm_type_value,
  });

  const pkm_obj = await tp.system.suggester(
    (item) => item.key,
    pkm_obj_arr,
    false,
    `${pkm_type_name}?`
  );

  const pkm_value = pkm_obj.value;
  const pkm_name = pkm_obj.key;

  // PKM METADATA CACHE
  const pkm_value_ext = `${pkm_value}.md`;
  const pkm_file_path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(`/${pkm_value_ext}`))
    .map((file) => file.path)[0];
  pkm_file_dir = pkm_file_path.replace(pkm_value_ext, "");

  const pkm_tfile = await app.vault.getAbstractFileByPath(pkm_file_path);
  const pkm_file_cache = await app.metadataCache.getFileCache(pkm_tfile);

  // CATEGORY METADATA CACHE
  if (pkm_type_value == "category") {
    category_value = pkm_value;
    category_name = pkm_name;
    category_link = `[[${pkm_value}|${pkm_name}]]`;
    category_value_link = `${new_line}${ul_yaml}"${category_link}"`;
  } else {
    let category_arr;
    let category_fmatter = pkm_file_cache?.frontmatter?.category;
    if (
      category_fmatter == "" ||
      category_fmatter == "null" ||
      category_fmatter == null ||
      typeof category_fmatter == "undefined"
    ) {
      category_fmatter = "null";
    } else {
      category_arr = category_fmatter.toString().split(";");
    }

    let category_value_arr = [];
    let category_name_arr = [];
    let category_link_arr = [];
    if (category_fmatter != "null") {
      for (var i = 0; i < category_arr.length; i++) {
        category_file_name = category_arr[i];
        category_value_arr.push(category_file_name);
        category_tfile = tp.file.find_tfile(`${category_arr[i]}.md`);
        category_file_cache = await app.metadataCache.getFileCache(
          category_tfile
        );
        category_alias = category_file_cache?.frontmatter?.aliases[0];
        category_name_arr.push(category_alias);
        category_file_link = `[[${category_file_name}|${category_alias}]]`;
        category_link_arr.push(category_file_link);
      }
      category_value = category_value_arr.join(", ");
      category_name = category_name_arr.join(", ");
      category_link = category_link_arr.join(", ");

      category_value_link = "";
      for (var i = 0; i < category_link_arr.length; i++) {
        category = `${new_line}${ul_yaml}"${category_link_arr[i]}"`;
        category_value_link += category;
      }
    }
  }

  // BRANCH METADATA CACHE
  if (pkm_type_value == "branch") {
    branch_value = pkm_value;
    branch_name = pkm_name;
    branch_link = `[[${pkm_value}|${pkm_name}]]`;
    branch_value_link = `${new_line}${ul_yaml}"${branch_link}"`;
  } else {
    let branch_arr;
    let branch_fmatter = pkm_file_cache?.frontmatter?.branch;
    if (
      branch_fmatter == "" ||
      branch_fmatter == "null" ||
      branch_fmatter == null ||
      typeof branch_fmatter == "undefined"
    ) {
      branch_fmatter = "null";
    } else {
      branch_arr = branch_fmatter.toString().split(";");
    }

    let branch_value_arr = [];
    let branch_name_arr = [];
    let branch_link_arr = [];
    if (branch_fmatter != "null") {
      for (var i = 0; i < branch_arr.length; i++) {
        branch_file_name = branch_arr[i];
        branch_value_arr.push(branch_file_name);
        branch_tfile = tp.file.find_tfile(`${branch_arr[i]}.md`);
        branch_file_cache = await app.metadataCache.getFileCache(branch_tfile);
        branch_alias = branch_file_cache?.frontmatter?.aliases[0];
        branch_name_arr.push(branch_alias);
        branch_file_link = `[[${branch_file_name}|${branch_alias}]]`;
        branch_link_arr.push(branch_file_link);
      }
      branch_value = branch_value_arr.join(", ");
      branch_name = branch_name_arr.join(", ");
      branch_link = branch_link_arr.join(", ");

      branch_value_link = "";
      for (var i = 0; i < branch_link_arr.length; i++) {
        branch = `${new_line}${ul_yaml}"${branch_link_arr[i]}"`;
        branch_value_link += branch;
      }
    }
  }

  // FIELD METADATA CACHE
  if (pkm_type_value == "field") {
    field_value = pkm_value;
    field_name = pkm_name;
    field_link = `[[${pkm_value}|${pkm_name}]]`;
    field_value_link = `${new_line}${ul_yaml}"${field_link}"`;
  } else {
    let field_arr;
    let field_fmatter = pkm_file_cache?.frontmatter?.field;
    if (
      field_fmatter == "" ||
      field_fmatter == "null" ||
      field_fmatter == null ||
      typeof field_fmatter == "undefined"
    ) {
      field_fmatter = "null";
    } else {
      field_arr = field_fmatter.toString().split(";");
    }

    let field_value_arr = [];
    let field_name_arr = [];
    let field_link_arr = [];
    if (field_fmatter != "null") {
      for (var i = 0; i < field_arr.length; i++) {
        field_file_name = field_arr[i];
        field_value_arr.push(field_file_name);
        field_tfile = tp.file.find_tfile(`${field_arr[i]}.md`);
        field_file_cache = await app.metadataCache.getFileCache(field_tfile);
        field_alias = field_file_cache?.frontmatter?.aliases[0];
        field_name_arr.push(field_alias);
        field_file_link = `[[${field_file_name}|${field_alias}]]`;
        field_link_arr.push(field_file_link);
      }
      field_value = field_value_arr.join(", ");
      field_name = field_name_arr.join(", ");
      field_link = field_link_arr.join(", ");

      field_value_link = "";
      for (var i = 0; i < field_link_arr.length; i++) {
        field = `${new_line}${ul_yaml}"${field_link_arr[i]}"`;
        field_value_link += field;
      }
    }
  }

  // SUBJECT METADATA CACHE
  if (pkm_type_value == "subject") {
    subject_value = pkm_value;
    subject_name = pkm_name;
    subject_link = `[[${pkm_value}|${pkm_name}]]`;
    subject_value_link = `${new_line}${ul_yaml}"${subject_link}"`;
  } else {
    let subject_arr;
    let subject_fmatter = pkm_file_cache?.frontmatter?.subject;
    if (
      subject_fmatter == "" ||
      subject_fmatter == "null" ||
      subject_fmatter == null ||
      typeof subject_fmatter == "undefined"
    ) {
      subject_fmatter = "null";
    } else {
      subject_arr = subject_fmatter.toString().split(";");
    }

    let subject_value_arr = [];
    let subject_name_arr = [];
    let subject_link_arr = [];
    if (subject_fmatter != "null") {
      for (var i = 0; i < subject_arr.length; i++) {
        subject_file_name = subject_arr[i];
        subject_value_arr.push(subject_file_name);
        subject_tfile = tp.file.find_tfile(`${subject_arr[i]}.md`);
        subject_file_cache = await app.metadataCache.getFileCache(
          subject_tfile
        );
        subject_alias = subject_file_cache?.frontmatter?.aliases[0];
        subject_name_arr.push(subject_alias);
        subject_file_link = `[[${subject_file_name}|${subject_alias}]]`;
        subject_link_arr.push(subject_file_link);
      }
      subject_value = subject_value_arr.join(", ");
      subject_name = subject_name_arr.join(", ");
      subject_link = subject_link_arr.join(", ");

      subject_value_link = "";
      for (var i = 0; i < subject_link_arr.length; i++) {
        subject = `${new_line}${ul_yaml}"${subject_link_arr[i]}"`;
        subject_value_link += subject;
      }
    }
  }

  // TOPIC METADATA CACHE
  if (pkm_type_value == "topic") {
    topic_value = pkm_value;
    topic_name = pkm_name;
    topic_link = `[[${pkm_value}|${pkm_name}]]`;
    topic_value_link = `${new_line}${ul_yaml}"${topic_link}"`;
  } else {
    let topic_arr;
    let topic_fmatter = pkm_file_cache?.frontmatter?.topic;
    if (
      topic_fmatter == "" ||
      topic_fmatter == "null" ||
      topic_fmatter == null ||
      typeof topic_fmatter == "undefined"
    ) {
      topic_fmatter = "null";
    } else {
      topic_arr = topic_fmatter.toString().split(";");
    }

    let topic_value_arr = [];
    let topic_name_arr = [];
    let topic_link_arr = [];
    if (topic_fmatter != "null") {
      for (var i = 0; i < topic_arr.length; i++) {
        topic_file_name = topic_arr[i];
        topic_value_arr.push(topic_file_name);
        topic_tfile = tp.file.find_tfile(`${topic_arr[i]}.md`);
        topic_file_cache = await app.metadataCache.getFileCache(topic_tfile);
        topic_alias = topic_file_cache?.frontmatter?.aliases[0];
        topic_name_arr.push(topic_alias);
        topic_file_link = `[[${topic_file_name}|${topic_alias}]]`;
        topic_link_arr.push(topic_file_link);
      }
      topic_value = topic_value_arr.join(", ");
      topic_name = topic_name_arr.join(", ");
      topic_link = topic_link_arr.join(", ");

      topic_value_link = "";
      for (var i = 0; i < topic_link_arr.length; i++) {
        topic = `${new_line}${ul_yaml}"${topic_link_arr[i]}"`;
        topic_value_link += topic;
      }
    }
  }

  if (pkm_type_value == "subtopic") {
    subtopic_value = pkm_value;
    subtopic_name = pkm_name;
    subtopic_link = `[[${pkm_value}|${pkm_name}]]`;
    subtopic_value_link = `${new_line}${ul_yaml}"${subtopic_link}"`;
  }
}

tR += pkm_file_dir;
tR += ";";
tR += category_value_link;
tR += ";";
tR += category_link;
tR += ";";
tR += branch_value_link;
tR += ";";
tR += branch_link;
tR += ";";
tR += field_value_link;
tR += ";";
tR += field_link;
tR += ";";
tR += subject_value_link;
tR += ";";
tR += subject_link;
tR += ";";
tR += topic_value_link;
tR += ";";
tR += topic_link;
tR += ";";
tR += subtopic_value_link;
tR += ";";
tR += subtopic_link;
```

#### Previous Template

```javascript
//---------------------------------------------------------
// FORMATTING CHARACTERS
//---------------------------------------------------------
const space = String.fromCodePoint(0x20);
const new_line = String.fromCodePoint(0xa);
const ul_yaml = `${space.repeat(2)}${String.fromCodePoint(0x2d)}${space}`;

//---------------------------------------------------------
// PKM TREE FILES AND NAMES
//---------------------------------------------------------
// Knowledge Tree directory
const pkm_dir = "70_pkm/";

// INITIALIZE VALUES
let category_value = "null";
let category_name = "Null";
let category_link = `[[${category_value}|${category_name}]]`;
let category_value_link = `${new_line}${ul_yaml}"${category_link}"`;
let branch_value = "null";
let branch_name = "Null";
let branch_link = `[[${branch_value}|${branch_name}]]`;
let branch_value_link = `${new_line}${ul_yaml}"${branch_link}"`;
let field_value = "null";
let field_name = "Null";
let field_link = `[[${field_value}|${field_name}]]`;
let field_value_link = `${new_line}${ul_yaml}"${field_link}"`;
let subject_value = "null";
let subject_name = "Null";
let subject_link = `[[${subject_value}|${subject_name}]]`;
let subject_value_link = `${new_line}${ul_yaml}"${subject_link}"`;
let topic_value = "null";
let topic_name = "Null";
let topic_link = `[[${topic_value}|${topic_name}]]`;
let topic_value_link = `${new_line}${ul_yaml}"${topic_link}"`;
let subtopic_value = "null";
let subtopic_name = "Null";
let subtopic_link = `[[${subtopic_value}|${subtopic_name}]]`;
let subtopic_value_link = `${new_line}${ul_yaml}"${subtopic_link}"`;

const pkm_type_obj_arr = [
  { key: "Null", value: "null" },
  { key: "Subtopic", value: "subtopic" },
  { key: "Topic", value: "topic" },
  { key: "Subject", value: "subject" },
  { key: "Field", value: "field" },
  { key: "Branch", value: "branch" },
  { key: "Category", value: "category" },
];

const pkm_type_obj = await tp.system.suggester(
  (item) => item.key,
  pkm_type_obj_arr,
  false,
  "Direct Knowledge Tree Level?"
);

const pkm_type_value = pkm_type_obj.value;
const pkm_type_name = pkm_type_obj.key;

if (pkm_type_value != "null") {
  // SET KNOWLEDGE TREE OBJECT NAME AND VALUE
  const pkm_obj_arr = await tp.user.file_name_alias_by_class_type_subtype({
    dir: pkm_dir,
    file_class: "pkm",
    type: "know",
    subtype: pkm_type_value,
  });

  const pkm_obj = await tp.system.suggester(
    (item) => item.key,
    pkm_obj_arr,
    false,
    `${pkm_type_name}?`
  );

  const pkm_value = pkm_obj.value;
  const pkm_name = pkm_obj.key;

  // PKM METADATA CACHE
  const pkm_tfile = tp.file.find_tfile(`${pkm_value}.md`);
  const pkm_file_cache = await app.metadataCache.getFileCache(pkm_tfile);

  // CATEGORY METADATA CACHE
  if (pkm_type_value == "category") {
    category_value = pkm_value;
    category_name = pkm_name;
    category_link = `[[${pkm_value}|${pkm_name}]]`;
    category_value_link = `${new_line}${ul_yaml}"${category_link}"`;
  } else {
    let category_arr;
    let category_fmatter = pkm_file_cache?.frontmatter?.category;
    if (
      category_fmatter == "" ||
      category_fmatter == "null" ||
      category_fmatter == null ||
      typeof category_fmatter == "undefined"
    ) {
      category_fmatter = "null";
    } else {
      category_arr = category_fmatter.toString().split(";");
    }

    let category_value_arr = [];
    let category_name_arr = [];
    let category_link_arr = [];
    if (category_fmatter != "null") {
      for (var i = 0; i < category_arr.length; i++) {
        category_file_name = category_arr[i];
        category_value_arr.push(category_file_name);
        category_tfile = tp.file.find_tfile(`${category_arr[i]}.md`);
        category_file_cache = await app.metadataCache.getFileCache(
          category_tfile
        );
        category_alias = category_file_cache?.frontmatter?.aliases[0];
        category_name_arr.push(category_alias);
        category_file_link = `[[${category_file_name}|${category_alias}]]`;
        category_link_arr.push(category_file_link);
      }
      category_value = category_value_arr.join(", ");
      category_name = category_name_arr.join(", ");
      category_link = category_link_arr.join(", ");

      category_value_link = "";
      for (var i = 0; i < category_link_arr.length; i++) {
        category = `${new_line}${ul_yaml}"${category_link_arr[i]}"`;
        category_value_link += category;
      }
    }
  }

  // BRANCH METADATA CACHE
  if (pkm_type_value == "branch") {
    branch_value = pkm_value;
    branch_name = pkm_name;
    branch_link = `[[${pkm_value}|${pkm_name}]]`;
    branch_value_link = `${new_line}${ul_yaml}"${branch_link}"`;
  } else {
    let branch_arr;
    let branch_fmatter = pkm_file_cache?.frontmatter?.branch;
    if (
      branch_fmatter == "" ||
      branch_fmatter == "null" ||
      branch_fmatter == null ||
      typeof branch_fmatter == "undefined"
    ) {
      branch_fmatter = "null";
    } else {
      branch_arr = branch_fmatter.toString().split(";");
    }

    let branch_value_arr = [];
    let branch_name_arr = [];
    let branch_link_arr = [];
    if (branch_fmatter != "null") {
      for (var i = 0; i < branch_arr.length; i++) {
        branch_file_name = branch_arr[i];
        branch_value_arr.push(branch_file_name);
        branch_tfile = tp.file.find_tfile(`${branch_arr[i]}.md`);
        branch_file_cache = await app.metadataCache.getFileCache(branch_tfile);
        branch_alias = branch_file_cache?.frontmatter?.aliases[0];
        branch_name_arr.push(branch_alias);
        branch_file_link = `[[${branch_file_name}|${branch_alias}]]`;
        branch_link_arr.push(branch_file_link);
      }
      branch_value = branch_value_arr.join(", ");
      branch_name = branch_name_arr.join(", ");
      branch_link = branch_link_arr.join(", ");

      branch_value_link = "";
      for (var i = 0; i < branch_link_arr.length; i++) {
        branch = `${new_line}${ul_yaml}"${branch_link_arr[i]}"`;
        branch_value_link += branch;
      }
    }
  }

  // FIELD METADATA CACHE
  if (pkm_type_value == "field") {
    field_value = pkm_value;
    field_name = pkm_name;
    field_link = `[[${pkm_value}|${pkm_name}]]`;
    field_value_link = `${new_line}${ul_yaml}"${field_link}"`;
  } else {
    let field_arr;
    let field_fmatter = pkm_file_cache?.frontmatter?.field;
    if (
      field_fmatter == "" ||
      field_fmatter == "null" ||
      field_fmatter == null ||
      typeof field_fmatter == "undefined"
    ) {
      field_fmatter = "null";
    } else {
      field_arr = field_fmatter.toString().split(";");
    }

    let field_value_arr = [];
    let field_name_arr = [];
    let field_link_arr = [];
    if (field_fmatter != "null") {
      for (var i = 0; i < field_arr.length; i++) {
        field_file_name = field_arr[i];
        field_value_arr.push(field_file_name);
        field_tfile = tp.file.find_tfile(`${field_arr[i]}.md`);
        field_file_cache = await app.metadataCache.getFileCache(field_tfile);
        field_alias = field_file_cache?.frontmatter?.aliases[0];
        field_name_arr.push(field_alias);
        field_file_link = `[[${field_file_name}|${field_alias}]]`;
        field_link_arr.push(field_file_link);
      }
      field_value = field_value_arr.join(", ");
      field_name = field_name_arr.join(", ");
      field_link = field_link_arr.join(", ");

      field_value_link = "";
      for (var i = 0; i < field_link_arr.length; i++) {
        field = `${new_line}${ul_yaml}"${field_link_arr[i]}"`;
        field_value_link += field;
      }
    }
  }

  // SUBJECT METADATA CACHE
  if (pkm_type_value == "subject") {
    subject_value = pkm_value;
    subject_name = pkm_name;
    subject_link = `[[${pkm_value}|${pkm_name}]]`;
    subject_value_link = `${new_line}${ul_yaml}"${subject_link}"`;
  } else {
    let subject_arr;
    let subject_fmatter = pkm_file_cache?.frontmatter?.subject;
    if (
      subject_fmatter == "" ||
      subject_fmatter == "null" ||
      subject_fmatter == null ||
      typeof subject_fmatter == "undefined"
    ) {
      subject_fmatter = "null";
    } else {
      subject_arr = subject_fmatter.toString().split(";");
    }

    let subject_value_arr = [];
    let subject_name_arr = [];
    let subject_link_arr = [];
    if (subject_fmatter != "null") {
      for (var i = 0; i < subject_arr.length; i++) {
        subject_file_name = subject_arr[i];
        subject_value_arr.push(subject_file_name);
        subject_tfile = tp.file.find_tfile(`${subject_arr[i]}.md`);
        subject_file_cache = await app.metadataCache.getFileCache(
          subject_tfile
        );
        subject_alias = subject_file_cache?.frontmatter?.aliases[0];
        subject_name_arr.push(subject_alias);
        subject_file_link = `[[${subject_file_name}|${subject_alias}]]`;
        subject_link_arr.push(subject_file_link);
      }
      subject_value = subject_value_arr.join(", ");
      subject_name = subject_name_arr.join(", ");
      subject_link = subject_link_arr.join(", ");

      subject_value_link = "";
      for (var i = 0; i < subject_link_arr.length; i++) {
        subject = `${new_line}${ul_yaml}"${subject_link_arr[i]}"`;
        subject_value_link += subject;
      }
    }
  }

  // TOPIC METADATA CACHE
  if (pkm_type_value == "topic") {
    topic_value = pkm_value;
    topic_name = pkm_name;
    topic_link = `[[${pkm_value}|${pkm_name}]]`;
    topic_value_link = `${new_line}${ul_yaml}"${topic_link}"`;
  } else {
    let topic_arr;
    let topic_fmatter = pkm_file_cache?.frontmatter?.topic;
    if (
      topic_fmatter == "" ||
      topic_fmatter == "null" ||
      topic_fmatter == null ||
      typeof topic_fmatter == "undefined"
    ) {
      topic_fmatter = "null";
    } else {
      topic_arr = topic_fmatter.toString().split(";");
    }

    let topic_value_arr = [];
    let topic_name_arr = [];
    let topic_link_arr = [];
    if (topic_fmatter != "null") {
      for (var i = 0; i < topic_arr.length; i++) {
        topic_file_name = topic_arr[i];
        topic_value_arr.push(topic_file_name);
        topic_tfile = tp.file.find_tfile(`${topic_arr[i]}.md`);
        topic_file_cache = await app.metadataCache.getFileCache(topic_tfile);
        topic_alias = topic_file_cache?.frontmatter?.aliases[0];
        topic_name_arr.push(topic_alias);
        topic_file_link = `[[${topic_file_name}|${topic_alias}]]`;
        topic_link_arr.push(topic_file_link);
      }
      topic_value = topic_value_arr.join(", ");
      topic_name = topic_name_arr.join(", ");
      topic_link = topic_link_arr.join(", ");

      topic_value_link = "";
      for (var i = 0; i < topic_link_arr.length; i++) {
        topic = `${new_line}${ul_yaml}"${topic_link_arr[i]}"`;
        topic_value_link += topic;
      }
    }
  }

  if (pkm_type_value == "subtopic") {
    subtopic_value = pkm_value;
    subtopic_name = pkm_name;
    subtopic_link = `[[${pkm_value}|${pkm_name}]]`;
    subtopic_value_link = `${new_line}${ul_yaml}"${subtopic_link}"`;
  }
}

tR += category_value_link;
tR += ";";
tR += category_link;
tR += ";";
tR += branch_value_link;
tR += ";";
tR += branch_link;
tR += ";";
tR += field_value_link;
tR += ";";
tR += field_link;
tR += ";";
tR += subject_value_link;
tR += ";";
tR += subject_link;
tR += ";";
tR += topic_value_link;
tR += ";";
tR += topic_link;
tR += ";";
tR += subtopic_value_link;
tR += ";";
tR += subtopic_link;
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[90_00_note|General Note Template]]
2. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
3. [[90_11_note_quote|Quote Fleeting Note Template]]
4. [[90_12_note_idea|Idea Fleeting Note Template]]
5. [[90_20_note_literature(X)|General Literature Note Template]]
6. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
7. [[90_31_note_question|QEC Question Note Template]]
8. [[90_32_note_evidence|QEC Evidence Note Template]]
9. [[90_33_note_conclusion|QEC Conclusion Note Template]]
10. [[90_40_note_lit_psa(X)|PSA Note Template]]
11. [[90_41_note_problem|PSA Problem Note Template]]
12. [[90_42_note_steps|PSA Steps Note Template]]
13. [[90_43_note_answer|PSA Answer Note Template]]
14. [[90_50_note_info(X)|General Info Note Template]]
15. [[90_51_note_concept|Concept Note Template]]
16. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[70_pkm_tree_name_link]]

---

## Related

### Outgoing Snippet Links

<!-- Link related snippet here -->

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition
WHERE
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
