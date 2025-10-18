<%*
//-------------------------------------------------------------------
// SET LIBRARY CONTENT FILE NAME AND ALIAS
//-------------------------------------------------------------------
// Library Files Directory
const library_dir = "60_library/";

const lib_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: library_dir,
  file_class: "lib",
  type: "",
});
const lib_obj = await tp.system.suggester(
  (item) => item.key,
  lib_obj_arr,
  false,
  "Library resource?"
);

let lib_resource_value = lib_obj.value;
let lib_resource_name = lib_obj.key;

let lib_resource_link = "";
if (lib_obj.value == "_user_input") {
  lib_resource_name = await tp.system.prompt(
    "Resource title?",
    null,
    false,
    false
  );
  lib_resource_value = await tp.system.prompt(
    "Resource link?",
    null,
    false,
    false
  );
  lib_resource_link = `[${lib_resource_name}](${lib_resource_value})`;
} else {
  lib_resource_link = `[[${lib_resource_value}|${lib_resource_name}]]`;
};

tR += lib_resource_value;
tR += ";";
tR += lib_resource_name;
tR += ";";
tR += lib_resource_link;
%>