<%*
//-------------------------------------------------------------------
// SET TYPE
//-------------------------------------------------------------------
const type_obj_arr = [
  { key: "Clause", value: "clause", value_short: "clause" },
  { key: "Data type", value: "data_type", value_short: "data_type" },
  { key: "Error", value: "error", value_short: "err" },
  { key: "Function", value: "function", value_short: "func" },
  { key: "Keyword", value: "keyword", value_short: "keyword" },
  { key: "Method", value: "method", value_short: "meth" },
  { key: "Operator", value: "operator", value_short: "oper" },
  { key: "Snippet", value: "snippet", value_short: "snip" },
  { key: "Statement", value: "statement", value_short: "stmt" },
];

const type_obj = await tp.system.suggester(
  (item) => item.key,
  type_obj_arr,
  false,
  "Code note type?"
);

const type_name = type_obj.key;
const type_value = type_obj.value;
const type_value_short = type_obj.value_short;

//-------------------------------------------------------------------
// SET SUBTYPE
//-------------------------------------------------------------------
const subtype_obj_arr = [
  { key: "Null", value: "null", value_short: "null" },
  { key: "Aggregate", value: "aggregate", value_short: "agg" },
  { key: "Array", value: "array", value_short: "arr" },
  { key: "Boolean", value: "boolean", value_short: "bool" },
  { key: "Converter", value: "converter", value_short: "convert" },
  { key: "Database", value: "database", value_short: "db" },
  { key: "DataFrame", value: "dataframe", value_short: "df" },
  { key: "Date", value: "date", value_short: "date" },
  { key: "Dictionary", value: "dictionary", value_short: "dict" },
  { key: "Engineering", value: "engineering", value_short: "eng" },
  { key: "File", value: "file", value_short: "file" },
  { key: "Filter", value: "filter", value_short: "fltr" },
  { key: "Financial", value: "financial", value_short: "fin" },
  { key: "Google", value: "google", value_short: "goog" },
  { key: "Info", value: "info", value_short: "info" },
  { key: "List", value: "list", value_short: "list" },
  { key: "Logical", value: "logical", value_short: "logic" },
  { key: "Lookup", value: "lookup", value_short: "look" },
  { key: "Math", value: "math", value_short: "math" },
  { key: "Numeric", value: "numeric", value_short: "num" },
  { key: "Object", value: "object", value_short: "obj" },
  { key: "Operator", value: "operator", value_short: "opr" },
  { key: "Parser", value: "parser", value_short: "prs" },
  {
    key: "Pass-Through RAWSQL",
    value: "pass_through_rawsql",
    value_short: "rawsql",
  },
  { key: "Regular Expression", value: "regex", value_short: "regex" },
  { key: "Series", value: "series", value_short: "ser" },
  { key: "Set", value: "set", value_short: "set" },
  { key: "Spatial", value: "spatial", value_short: "space" },
  { key: "Statistics", value: "statistics", value_short: "stat" },
  { key: "String", value: "string", value_short: "str" },
  {
    key: "Table Calculation",
    value: "table_calculation",
    value_short: "table_calc",
  },
  { key: "Tuple", value: "tuple", value_short: "tupl" },
  { key: "User", value: "user", value_short: "user" },
  { key: "Web", value: "web", value_short: "web" },
  { key: "General", value: "general", value_short: "general" },
];

const subtype_obj = await tp.system.suggester(
  (item) => item.key,
  subtype_obj_arr,
  false,
  `${type_name} subtype?`
);

const subtype_name = subtype_obj.key;
const subtype_value = subtype_obj.value;
const subtype_value_short = subtype_obj.value_short;

tR += type_value;
tR += ";";
tR += type_value_short;
tR += ";";
tR += type_name;
tR += ";";
tR += subtype_value;
tR += ";";
tR += subtype_value_short;
tR += ";";
tR += subtype_name;
%>
