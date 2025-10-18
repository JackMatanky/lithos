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
const button_obj_arr = [
  {
    name: "name 🕯️Weekly PDEV Files",
    type: "type append template",
    action: "action ",
    file: "112_90_dvmd_week_pdev",
    replace: "replace [61, 600]",
    color: "color purple",
  },
  {
    name: "name 🏫Weekly Library Content",
    type: "type append template",
    action: "action ",
    file: "112_60_dvmd_week_lib",
    replace: "replace [56, 600]",
    color: "color green",
  },
  {
    name: "name 🗃️Weekly PKM Files",
    type: "type append template",
    action: "action ",
    file: "112_70_dvmd_week_pkm",
    replace: "replace [63, 600]",
    color: "color green",
  },
  {
    name: "name 🦾Weekly Habits and Rituals",
    type: "type append template",
    action: "action ",
    file: "112_45_dvmd_week_habit_rit",
    replace: "replace [55, 500]",
    color: "color blue",
  },
  {
    name: "name ✅Weekly Tasks and Events",
    type: "type append template",
    action: "action ",
    file: "112_40_dvmd_week_tasks",
    replace: "replace [55, 500]",
    color: "color blue",
  },
];

const week_button_arr = button_arr(button_obj_arr);
const button_pdev_week = week_button_arr[0];
const button_lib_week = week_button_arr[1];
const button_pkm_week = week_button_arr[2];
const button_habit_rit_week = week_button_arr[3];
const button_tasks_event_week = week_button_arr[4];

tR += button_pdev_week;
tR += ";";
tR += button_lib_week;
tR += ";";
tR += button_pkm_week;
tR += ";";
tR += button_habit_rit_week;
tR += ";";
tR += button_tasks_event_week;
%>