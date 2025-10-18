<%*
/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
const space = String.fromCodePoint(0x20);
const new_line = String.fromCodePoint(0xa);
const ul_yaml = `${space.repeat(2)}${String.fromCodePoint(0x2d)}${space}`;

//-------------------------------------------------------------------
// SET PILLAR FILE NAME, ALIAS, LINK, AND YAML LINK
//-------------------------------------------------------------------
// Pillar Files Directory
const pillars_dir = "20_pillars/";
const type_name = "Pillar";

// Files Directory
const directory = pillars_dir;

// Boolean Arrays of Objects
const bool_obj_arr = [
  { key: "✔️ YES ✔️", value: "yes" },
  { key: "❌ NO ❌", value: "no" },
];

// Retrieve all files in the Pillars directory
const file_by_status_obj_arr = await tp.user.file_by_status({
  dir: directory,
  status: "active",
});

let file_obj_arr = [];
let file_filter = [];
for (let i = 0; i < 10; i++) {
  // File Suggester
  const file_by_status_obj = await tp.system.suggester(
    (item) => item.key,
    file_by_status_obj_arr.filter(
      (file) => !file_filter.includes(file.value)
    ),
    false,
    `${type_name}?`
  );
  file_basename = file_by_status_obj.value;
  file_alias_name = file_by_status_obj.key;
  
  if (file_basename == "null") {
    file_obj = { key: file_alias_name, value: file_basename };
    file_obj_arr.push(file_obj);
    break;
  }
  file_obj = { key: file_alias_name, value: file_basename };
  file_obj_arr.push(file_obj);
  file_filter.push(file_basename);

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
  .map((file) => `[[${file.value}|${file.key}]]`)
  .join(", ");
const property_link = file_obj_arr
  .map((file) => `${new_line}${ul_yaml}"[[${file.value}|${file.key}]]"`)
  .join("");

tR += value;
tR += ";";
tR += property_link;
%>