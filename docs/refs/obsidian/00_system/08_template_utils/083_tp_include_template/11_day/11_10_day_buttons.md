<%*
/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

//-------------------------------------------------------------------
// INSERT TEMPLATER INCLUDE TEMPLATE STRING
//-------------------------------------------------------------------
const tp_start = [
  less_than + String.fromCodePoint(0x25) + "*",
  "tR",
  "+=",
  "await",
].join(space);
const tp_end = String.fromCodePoint(0x25) + great_than;
const tp_include = ["tp", "user", "include_template"].join(".");
const temp_include = (file) =>
  [tp_start, tp_include + `(tp, "${file}")`, tp_end].join(space);

//-------------------------------------------------------------------
// BUTTONS GENERATING FUNCTION
//-------------------------------------------------------------------
const button_start = `${three_backtick}button`;
const button_end = three_backtick;
const button_arr = (obj_arr) =>
  obj_arr.map((b) =>
    [
      (b.replace
        ? [
            button_start,
            b.name,
            b.type,
            b.action + b.file,
            b.replace,
            b.color,
            button_end,
          ]
        : [button_start, b.name, b.type, b.action + b.file, b.color, button_end]
      ).join(new_line),
      temp_include(b.file),
      cmnt_html("Adjust replace lines"),
    ].join(two_new_line)
  );

//-------------------------------------------------------------------
// BUTTONS OBJECT ARRAY
//-------------------------------------------------------------------
const day_button_obj_arr = [
  {
    name: "name 🕯️Daily Personal Development",
    type: "type append template",
    action: "action ",
    file: "111_90_dvmd_day_pdev",
    replace: "replace [53, 400]",
    color: "color purple",
  },
  {
    name: "name 🏫Daily Library Content",
    type: "type append template",
    action: "action ",
    file: "111_60_dvmd_day_lib",
    replace: "replace [56, 400]",
    color: "color green",
  },
  {
    name: "name 🗃️Daily PKM Files",
    type: "type append template",
    action: "action ",
    file: "111_70_dvmd_day_pkm",
    replace: "replace [63, 400]",
    color: "color green",
  },
  {
    name: "name ✅Planned Tasks and Events",
    type: "type append template",
    action: "action ",
    file: "111_40_dvmd_day_tasks",
    replace: "replace [59, 500]",
    color: "color yellow",
  },
];

const day_button_arr = button_arr(day_button_obj_arr);
const button_pdev_day = day_button_arr[0];
const button_lib_day = day_button_arr[1];
const button_pkm_day = day_button_arr[2];
const button_task_day = day_button_arr[3];

tR += button_pdev_day;
tR += ";";
tR += button_lib_day;
tR += ";";
tR += button_pkm_day;
tR += ";";
tR += button_task_day;
%>
