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
// SET PILLAR FILE AND FULL NAME; PRESET CAREER DEV.
//-------------------------------------------------------------------
// CAREER DEVELOPMENT PILLAR FILE AND FULL NAME
const preset_pillar_name = "Career Development";
const preset_pillar_value = preset_pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const preset_pillar_link = `[[${preset_pillar_value}|${preset_pillar_name}]]`;
const preset_pillar_value_link = yaml_li(preset_pillar_link);

// Pillar Files Directory
const pillars_dir = "20_pillars/";
const type_name = "Pillar";

// Files Directory
const directory = pillars_dir;

// Arrays of Objects
const bool_obj_arr = [
  { key: "✔️ YES ✔️", value: "yes" },
  { key: "❌ NO ❌", value: "no" },
];

// Retrieve all files in the Pillars directory
const file_by_status_obj_arr = await tp.user.file_by_status({
  dir: directory,
  status: "active",
});

let file_obj_arr = [{ key: preset_pillar_name, value: preset_pillar_value }];
let file_filter = [preset_pillar_value];
for (let i = 0; i < 10; i++) {
  // File Suggester
  const file_by_status_obj = await tp.system.suggester(
    (item) => item.key,
    file_by_status_obj_arr.filter(
      (file) => !file_filter.includes(file.value)
    ),
    false,
    `${type_name} (${preset_pillar_name} already included)?`
  );
  file_basename = file_by_status_obj.value;
  file_alias_name = file_by_status_obj.key;
  
  if (file_basename == "null") {
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
  .map((file) => link_alias(file.value, file.key))
  .join(", ");
const property_link = file_obj_arr
  .map((file) => yaml_li(link_alias(file.value, file.key)))
  .join("");

tR += value;
tR += ";";
tR += property_link;
%>