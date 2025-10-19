<%*
//-------------------------------------------------------------------
// SET NOTE STATUS
//-------------------------------------------------------------------
const status_obj_arr = [
  { key: "🌱️Review", value: "review" },
  { key: "🌿️Clarify", value: "clarify" },
  { key: "🪴Develop", value: "develop" },
  { key: "🌳Permanent", value: "permanent" },
  { key: "🗃️Resource", value: "resource" },
];

const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  "Status?"
);

const status_name = status_obj.key;
const status_value = status_obj.value;

tR += status_name;
tR += ";";
tR += status_value;
%>
