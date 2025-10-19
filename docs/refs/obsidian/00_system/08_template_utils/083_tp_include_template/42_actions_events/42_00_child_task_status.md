<%*
/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const status_obj_arr = [
  { key: "🔜To do", value: "to_do", symbol: " " },
  { key: "✔️Done", value: "done", symbol: "x" },
  { key: "📅Schedule", value: "schedule", symbol: "?" },
];

const status_obj = await tp.system.suggester(
  (item) => item.key,
  status_obj_arr,
  false,
  "Task Status?"
);

const status_value = status_obj.value;
const status_name = status_obj.key;
const status_symbol = status_obj.symbol;

tR += status_value;
tR += ";";
tR += status_name;
tR += ";";
tR += status_symbol;
%>
