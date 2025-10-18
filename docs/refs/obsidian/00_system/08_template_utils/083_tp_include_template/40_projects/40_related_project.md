<%*
//-------------------------------------------------------------------
// SET PROJECT
//-------------------------------------------------------------------
// Projects directory
const project_path_arr = [
  "41_personal/",
  "42_education/",
  "43_professional/",
  "44_work/",
  "45_habit_ritual/",
];

// Filter array to only include projects in the Projects Directory
let projects_obj_arr = [{ key: "Null", value: "null" }];
for (let i = 0; i < project_path_arr.length; i++) {
  const obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_path_arr[i],
    file_class: "task",
    type: "project",
  });
  projects_obj_arr.push(
    ...obj_arr.filter((x) => !["null", "_user_input"].includes(x.value))
  );
}

const project_obj = await tp.system.suggester(
  (item) => item.key,
  projects_obj_arr,
  false,
  "Is this file related to a project?"
);

const project_value = project_obj.value;
const project_name = project_obj.key;

tR += project_value;
tR += ";";
tR += project_name;
%>