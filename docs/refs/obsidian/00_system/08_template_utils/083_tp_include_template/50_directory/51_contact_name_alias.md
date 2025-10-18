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
// SET CONTACT FILE NAME, ALIAS, LINK, AND YAML LINK
//-------------------------------------------------------------------
// Contact Files Directory
const contacts_dir = "51_contacts/";
const type_name = "Contact";

// Files Directory
const directory = contacts_dir;

// Arrays of Objects
const bool_obj_arr = [
  { key: "✔️ YES ✔️", value: "yes" },
  { key: "❌ NO ❌", value: "no" },
];
const md_file_name_alias_obj_arr = await tp.user.md_file_name_alias(directory);

let file_obj_arr = [];
let file_filter_arr = [];
for (let i = 0; i < 10; i++) {
  // File Suggester
  const md_file_name_alias_obj = await tp.system.suggester(
    (item) => item.key,
    md_file_name_alias_obj_arr.filter(
      (file) => !file_filter_arr.includes(file.value)
    ),
    false,
    `${type_name}?`
  );
  file_basename = md_file_name_alias_obj.value;
  file_alias_name = md_file_name_alias_obj.key;

  if (file_basename == "_user_input") {
    dir_contact_names = await tp.user.dirContactNames(tp);
    file_alias_name = dir_contact_names.full_name;
    file_basename = dir_contact_names.last_first_name
      .replaceAll(/,/g, "")
      .replaceAll(/[^\w]/g, "_")
      .toLowerCase();
  } else if (file_basename == "null") {
    file_obj = { key: file_alias_name, value: file_basename };
    file_obj_arr.push(file_obj);
    break;
  }
  file_obj = { key: file_alias_name, value: file_basename };
  file_obj_arr.push(file_obj);
  file_filter_arr.push(file_basename);

  const bool_obj = await tp.system.suggester(
    (item) => item.key,
    bool_obj_arr,
    false,
    `Another ${type_name}?`
  );

  if (bool_obj.value == "no") {
    break;
  }
}

const value = file_obj_arr.map((file) => file.value).join(", ");
const name = file_obj_arr.map((file) => file.key).join(", ");
const link = file_obj_arr
  .map((file) => yaml_li(link_alias(file.value, file.key)))
  .join("");

tR += value;
tR += ";";
tR += name;
tR += ";";
tR += link;
%>