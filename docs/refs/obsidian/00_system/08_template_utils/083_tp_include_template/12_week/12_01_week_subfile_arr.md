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

//-------------------------------------------------------------------
// WEEK SUBFILE DETAILS
//-------------------------------------------------------------------
const subfile_obj_arr = [
  {
    head_key: "Personal Development",
    key: "PDEV",
    value: "pdev",
  },
  {
    head_key: "Library",
    key: "Library",
    value: "lib",
  },
  {
    head_key: "Knowledge Management",
    key: "PKM",
    value: "pkm",
  },
  {
    head_key: "Habits and Rituals",
    key: "Habits and Rituals",
    value: "habit_ritual",
  },
  {
    head_key: "Tasks and Events",
    key: "Tasks and Events",
    value: "task_event",
  },
];

tR += subfile_obj_arr;
%>
