<%*
//-------------------------------------------------------------------
// SET LIBRARY STATUS
//-------------------------------------------------------------------
const status_obj_arr = [
  { key: "❓Undetermined", value: "undetermined" },
  { key: "🔜To do", value: "to_do" },
  { key: "👟In progress", value: "in_progress" },
  { key: "✔️Done", value: "done" },
  { key: "🗃️Resource", value: "resource" },
  { key: "📅Schedule", value: "schedule" },
  { key: "🤌On hold", value: "on_hold" },
];

const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  "Resource status?"
);

const status_value = status_obj.value;
const status_name = status_obj.key;

tR += status_value;
tR += ";";
tR += status_name;
%>