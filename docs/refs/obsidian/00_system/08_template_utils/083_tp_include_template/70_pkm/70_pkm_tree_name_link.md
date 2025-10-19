<%*
/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
const space = String.fromCodePoint(0x20);
const new_line = String.fromCodePoint(0xa);
const ul_yaml = `${space.repeat(2)}${String.fromCodePoint(0x2d)}${space}`;
const yaml_li = (value) => `${new_line}${ul_yaml}"${value}"`;
const link_alias = (file, alias) => ["[[" + file, alias + "]]"].join("|");

//-------------------------------------------------------------------
// PKM TREE FILES AND NAMES
//-------------------------------------------------------------------
// Knowledge Tree directory
const pkm_dir = "70_pkm/";

const null_arr = ["", "null", "[[null|Null]]", null];
const null_link = "[[null|Null]]";
const null_yaml_li = yaml_li(null_link);
const null_obj = {
  index: 0,
  key: "Null",
  value: "null",
  value_link: null_yaml_li,
};

const pkm_tree_obj_arr = [
  { index: 6, key: "Subtopic", value: "subtopic", value_link: null_yaml_li },
  { index: 5, key: "Topic", value: "topic", value_link: null_yaml_li },
  { index: 4, key: "Subject", value: "subject", value_link: null_yaml_li },
  { index: 3, key: "Field", value: "field", value_link: null_yaml_li },
  { index: 2, key: "Branch", value: "branch", value_link: null_yaml_li },
  { index: 1, key: "Category", value: "category", value_link: null_yaml_li },
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
  pkm_link = link_alias(pkm_file_obj.value, pkm_file_obj.key)

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
const subtopic_value_link = pkm_tree_value_link[5];

tR += pkm_file_dir;
tR += ";";
tR += category_value_link;
tR += ";";
tR += branch_value_link;
tR += ";";
tR += field_value_link;
tR += ";";
tR += subject_value_link;
tR += ";";
tR += topic_value_link;
tR += ";";
tR += subtopic_value_link;
%>
