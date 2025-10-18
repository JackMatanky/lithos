---
title: side_panel_tasks_due
aliases:
  - Side Panel Tasks Due
  - side panel tasks due
  - Side Panel Daily Tasks Due
  - side panel daily tasks due
  - Dataview Due Today
  - dv task table due today
  - side_panel_dv_due_today
  - side_panel_tasks_due
cssclasses:
  - inline_title_hide
  - read_hide_properties
  - side_panel_style
  - table_narrow_margin
file_class: task
date_created: 2023-09-03T19:26
date_modified: 2024-11-08T16:32
---
```dataviewjs
/* ---------------------------------------------------------- */
/*                     CONFIG AND IMPORTS                     */
/* ---------------------------------------------------------- */
const { Duration, DateTime } = dv.luxon;
const PAST_TASKS_OVERDUE = 7;
const FUTURE_TASKS_DUE = 4;

/* ---------------------------------------------------------- */
/*                     GENERAL FUNCTIONS                      */
/* ---------------------------------------------------------- */
const title_case = (str) => str.charAt(0).toUpperCase() + str.slice(1);

/* ---------------------------------------------------------- */
/*                     DATE FUNCTIONS                         */
/* ---------------------------------------------------------- */
const day_minus = (day_int) => DateTime.now().minus({ days: day_int });
const day_plus = (day_int) => DateTime.now().plus({ days: day_int });
const day_plus_fmt = (day_int) => day_plus(day_int).toFormat('yyyy-MM-dd');
const day_plus_name = (day_int) =>
  day_int === 1
    ? 'Tomorrow'
    : day_int < 2
    ? title_case(day_plus(day_int).toRelativeCalendar())
    : day_plus(day_int).toFormat('cccc');
const day_file_link = (day_int) =>
  dv.fileLink(
    `${day_plus_fmt(day_int)}_task_event`,
    false,
    day_plus_name(day_int)
  );

/* ---------------------------------------------------------- */
/*                       REGEX VARIABLES                      */
/* ---------------------------------------------------------- */
const regex_patterns = {
  task_name:
    /#task\s(.+)_((action_|(phone|video)_call|interview|appointment|lecture|tutorial|event|hangout|habit|(meet|gather)ing|(morn|even)ing_|workday_).+?)\[.+/g,
  date_due: /#task.+📅\s*(\d{4}-\d{2}-\d{2}).*/g,
  time_start: /#task.+time_start::\s*([\d:]+).+/g,
  time_end: /#task.+time_end::\s*([\d:]+).+/g,
  task_file_link: /.+\/(.+)\s>.+\]\]$/g,
  task_section_link: /.+\/.+\s>(.+)\]\]$/g,
};

/* ---------------------------------------------------------- */
/*                        OBJECT ARRAYS                       */
/* ---------------------------------------------------------- */
const status_obj_arr = [
  { status: 'overdue', head: 'Overdue', key: ' ', value: '🔜' },
  { status: 'due', head: 'Due', key: ' ', value: '🔜' },
  { status: 'done', head: 'Completed', key: 'x', value: '✔️' },
  { status: 'cancel', head: 'Cancelled', key: '<', value: '⏹️' },
  { status: 'discard', head: 'Discarded', key: '-', value: '❌' },
];

const type_arg_arr = [
  { key: 'task', head: 'Tasks & Events' },
  { key: 'habit_rit', head: 'Habits & Rituals' },
  { key: 'all', head: 'Overall' },
];

const project_path_arr = [
  `"41_personal"`,
  `"42_education"`,
  `"43_professional"`,
  `"44_work"`,
  `"45_habit_ritual"`,
];

const general_filter_arr = ['all', '', null];

/* ---------------------------------------------------------- */
/*                        DATA FETCHING                       */
/* ---------------------------------------------------------- */
const task_pages = dv
  .pages(project_path_arr.join(' OR '))
  .file.tasks.filter(
    (task) =>
      task.due > day_minus(PAST_TASKS_OVERDUE) &&
      task.due < day_plus(FUTURE_TASKS_DUE)
  );

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
const get_task_info = (task, info) =>
  task.text.replace(regex_patterns[info], '$1');

const get_task_time_span = (task) =>
  [
    '⏳' + get_task_info(task, 'time_start'),
    '→',
    '⌛' + get_task_info(task, 'time_end'),
  ].join(' ');

const get_task_datetime = (task, time) =>
  DateTime.fromISO(
    `${get_task_info(task, 'date_due')}T${get_task_info(task, time)}`
  ).toISO();

function get_task_visual(task) {
  task.visual = dv
    .array([get_task_time_span(task), get_task_info(task, 'task_name')])
    .join(': ');
  return task;
}

/* ---------------------------------------------------------- */
/*                FILTER AND SORTING FUNCTIONS                */
/* ---------------------------------------------------------- */
const tag_filter = (task) => task.text.includes('#task');
const habit_ritual_filter = (task) =>
  task.text.includes('_habit') || task.text.includes('_ritual');

const date_filter = (task, date_arg, status_arg) => {
  const date_type = status_arg === 'done' ? task.completion : task.due;
  const date_filter = DateTime.fromISO(date_type).toFormat('yyyy-MM-dd');
  return status_arg !== 'overdue'
    ? date_filter === date_arg
    : date_filter < date_arg;
};

const type_filter = (task, type_arg) =>
  general_filter_arr.includes(type_arg)
    ? true
    : type_arg === 'task'
    ? !habit_ritual_filter(task)
    : habit_ritual_filter(task);

const status_filter = (task, status_arg) => {
  const status_type =
    status_obj_arr.find((x) => x.status === status_arg)?.key ?? null;
  return status_type ? task.status === status_type : true;
};

const task_filter = (task, date_arg, type_arg, status_arg) => {
  const tag = tag_filter(task);
  const date = date_filter(task, date_arg, status_arg);
  const type = type_filter(task, type_arg);
  const status = status_filter(task, status_arg);

  return tag && date && type && status;
};

/* ---------------------------------------------------------- */
/*     OBJECT ARRAY FILTER, SORTING, AND MAPPING FUNCTIONS    */
/* ---------------------------------------------------------- */
const task_page_filter = (date_arg, type_arg, status_arg) =>
  task_pages.filter((task) =>
    task_filter(task, date_arg, type_arg, status_arg)
  );

const task_map = (task) => ({
  name: get_task_info(task, 'task_name'),
  visual_name: get_task_visual(task),
  datetime: get_task_datetime(task, 'time_start'),
});

const task_sort = (task) => task.datetime;

const task_obj_arr = (date_arg, type_arg, status_arg) =>
  task_page_filter(date_arg, type_arg, status_arg)
    .map(task_map)
    .sort(task_sort);

const task_visual_name_arr = (date_arg, type_arg, status_arg) =>
  task_obj_arr(date_arg, type_arg, status_arg).map((task) => task.visual_name);

/* ---------------------------------------------------------- */
/*                        COUNTING FUNCTIONS                  */
/* ---------------------------------------------------------- */
const task_count = (date_arg, type_arg, status_arg) =>
  [...new Set(task_obj_arr(date_arg, type_arg, status_arg))].length;

const status_count_arr = (date_arg, type_arg) => {
  const total = task_count(date_arg, type_arg, 'all');
  const overdue = task_count(date_arg, type_arg, 'overdue');
  const due = task_count(date_arg, type_arg, 'due');
  const done = task_count(date_arg, type_arg, 'done');
  const discard = task_count(date_arg, type_arg, 'discard');
  return { total, overdue, due, done, discard };
};

const calculate_status_rate = (status_total, total, round_digits = 2) => {
  const percentage =
    (total ? ((status_total / total) * 100).toFixed(round_digits) : '0') + '%';

  return round_digits === 1
    ? percentage
    : `${percentage} (${status_total}/${total})`;
};

const date_status_rate = (date_arg, type_arg, round_digits = 2) => {
  const counts = status_count_arr(date_arg, type_arg);
  const completion_rate = calculate_status_rate(
    counts.done,
    counts.total,
    round_digits
  );
  const discard_rate = calculate_status_rate(
    counts.discard,
    counts.total,
    round_digits
  );
  return { completion_rate, discard_rate };
};

/* ---------------------------------------------------------- */
/*                    GENERATING FUNCTIONS                    */
/* ---------------------------------------------------------- */
const current_day_task_bool = (day_int, status_arg) =>
  day_int === 0 && status_arg !== 'overdue';

const generate_header = (day_int, status_arg = 'due') =>
  dv.header(
    1,
    status_arg === 'overdue' ? title_case(status_arg) : day_file_link(day_int)
  );

const generate_current_day_head = (day_int, status_arg = 'due') =>
  current_day_task_bool(day_int, status_arg) &&
  task_count(day_plus_fmt(day_int), 'all', 'all')
    ? dv.header(1, day_file_link(day_int))
    : null;

const generate_task_list = (day_int, status_arg = 'due') =>
  dv.taskList(
    task_visual_name_arr(day_plus_fmt(day_int), 'all', status_arg),
    false
  );

const generate_current_day_task_done_str = (day_int, status_arg) =>
  current_day_task_bool(day_int, status_arg) &&
  task_count(day_plus_fmt(day_int), 'all', 'all')
    ? dv.paragraph('No more tasks for today 👍')
    : null;

const generate_table_rows = (day_int) =>
  type_arg_arr.map(({ key, head }) => {
    const counts = status_count_arr(day_plus_fmt(day_int), key);
    return [
      head,
      counts.total,
      counts.done,
      calculate_status_rate(counts.done, counts.total, 1),
    ];
  });

/* ---------------------------------------------------------- */
/*                     RENDERING FUNCTIONS                    */
/* ---------------------------------------------------------- */
const render_header = (day_int, status_arg = 'due') =>
  task_count(day_plus_fmt(day_int), 'all', status_arg)
    ? generate_header(day_int, status_arg)
    : generate_current_day_head(day_int, status_arg);

const render_task_table = (day_int) =>
  dv.table(
    ['Type', '📆Due', '✅Done', '✔️Rate%'],
    generate_table_rows(day_int)
  );

const render_task_list = (day_int, status_arg = 'due') =>
  task_count(day_plus_fmt(day_int), 'all', status_arg)
    ? generate_task_list(day_int, status_arg)
    : generate_current_day_task_done_str(day_int, status_arg);

function render_day_section(day_int, status_arg = 'due') {
  render_header(day_int, status_arg);
  if (current_day_task_bool(day_int, status_arg)) {
    render_task_table(day_int);
  }
  render_task_list(day_int, status_arg);
}

/* ---------------------------------------------------------- */
/*                     TASK DATA RENDERING                    */
/* ---------------------------------------------------------- */
render_day_section(0, 'overdue');
render_day_section(0);
render_day_section(1);
render_day_section(2);
render_day_section(3);
```