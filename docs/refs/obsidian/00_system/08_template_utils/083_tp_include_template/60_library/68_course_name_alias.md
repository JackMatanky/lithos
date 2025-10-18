<%*
//-------------------------------------------------------------------  
// SET COURSE
//-------------------------------------------------------------------  
// Directory
const directory = "60_library/68_courses/";

// Filter array to only include courses in the Course Directory
const obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: directory,
    file_class: "lib",
    type: "course",
  });

const obj = await tp.system.suggester(
  (item) => item.key,
  obj_arr,
  false,
  "Course?"
);

const value = obj.value;
const name = obj.key;

tR += value;
tR += ";";
tR += name;
%>