<%*
/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const due_do_obj_arr = [
  { key: "DO Date", value: "do" },
  { key: "DUE Date", value: "due" },
];

const due_do_obj = await tp.system.suggester(
  (item) => item.key,
  due_do_obj_arr,
  false,
  "Do or Due Date?"
);

const due_do_value = due_do_obj.value;
const due_do_name = due_do_obj.key;

tR += due_do_value;
tR += ";";
tR += due_do_name;
%>
