---
title: side_panel_tasks_done
aliases:
  - Side Panel Tasks Done
  - side panel tasks done
  - Side Panel Daily Tasks Done
  - side panel daily tasks done
  - Dataview Done Today
  - dv task table done today
  - side_panel_dv_done_today
  - side_panel_tasks_done
cssclasses:
  - inline_title_hide
  - read_hide_properties
  - read_narrow_margin
  - side_panel_style
  - table_narrow_margin
file_class: task
date_created: 2023-09-03T19:26
date_modified: 2024-11-08T14:46
---
```dataviewjs
//-------------------------------------------------------------------
// GENERAL FUNCTIONS
//-------------------------------------------------------------------
const title_case = (str) => str.charAt(0).toUpperCase() + str.slice(1);

//-------------------------------------------------------------------
// DATE FUNCTIONS
//-------------------------------------------------------------------
const { Duration, DateTime } = dv.luxon;
const day_minus = (day_int) => DateTime.now().minus({ days: day_int });
const day_minus_fmt = (day_int) => day_minus(day_int).toFormat('yyyy-MM-dd');
const day_minus_name = (day_int) =>
  day_int < 2
    ? title_case(day_minus(day_int).toRelativeCalendar({ unit: 'days' }))
    : day_minus(day_int).toFormat('cccc');
const day_file_link = (day_int) =>
  dv.fileLink(
    `${day_minus_fmt(day_int)}_task_event`,
    false,
    day_minus_name(day_int)
  );

//-------------------------------------------------------------------
// REGEX VARIABLES
//-------------------------------------------------------------------
const regex_patterns = {
  task_name:
    /#task\s(.+)_((action_|(phone|video)_call|interview|appointment|lecture|tutorial|event|hangout|habit|(meet|gather)ing|(morn|even)ing_|workday_).+?)\[.+/g,
  date_due: /#task.+📅\s*(\d{4}-\d{2}-\d{2}).*/g,
  time_start: /#task.+time_start::\s*([\d:]+).+/g,
  time_end: /#task.+time_end::\s*([\d:]+).+/g,
  duration: /#task.+duration_est::\s*([\d]+).+/g,
  task_file_link: /.+\/(.+)\s>.+\]\]$/g,
  task_section_link: /.+\/.+\s>(.+)\]\]$/g,
};

//-------------------------------------------------------------------
// OBJECT ARRAYS
//-------------------------------------------------------------------
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

//-------------------------------------------------------------------
// DATA FETCHING
//-------------------------------------------------------------------
const task_pages = dv
  .pages(project_path_arr.join(' OR '))
  .file.tasks.filter((task) => task.due > day_minus(4));

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
const get_task_info = (task, info) =>
  info !== 'duration'
    ? task.text.replace(regex_patterns[info], '$1')
    : Number(task.text.replace(regex_patterns.duration, '$1'));

const get_task_datetime = (task, time) =>
  DateTime.fromISO(
    `${get_task_info(task, 'date_due')}T${get_task_info(task, time)}`
  );

const get_task_duration_act = (task) =>
  get_task_datetime(task, 'time_end')
    .diff(get_task_datetime(task, 'time_start'))
    .as('minutes');

const get_task_duration_str = (task) =>
  [
    '⏳' + get_task_info(task, 'time_start'),
    '→',
    get_task_info(task, 'time_end') + '⌛',
    ':',
    '⏲️' + get_task_duration_act(task),
  ].join(' ');

const get_file_name = (task) => {
  const file_path = task.path.split('/');
  return file_path[file_path.length - 1].replace('.md', '');
};

const get_file_section = (task) =>
  task.link.toString().replace(regex_patterns.task_section_link, '$1');

const get_task_link = (task) =>
  dv.sectionLink(
    get_file_name(task),
    get_file_section(task),
    false,
    get_task_info(task, 'task_name')
  );

const task_obj = (task) => ({
  name: get_task_info(task, 'task_name'),
  link: get_task_link(task),
  start_datetime: get_task_datetime(task, 'time_start'),
  end_datetime: get_task_datetime(task, 'time_end'),
  duration_est: get_task_info(task, 'duration'),
  duration_act: get_task_duration_act(task, 'duration'),
  text: task.text,
  status: task.status,
  path: task.path,
});

//-------------------------------------------------------------------
// FILTER AND SORTING FUNCTIONS
//-------------------------------------------------------------------
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

//-------------------------------------------------------------------
// OBJECT ARRAY FILTER, SORTING, AND MAPPING FUNCTIONS
//-------------------------------------------------------------------
const task_page_filter = (date_arg, type_arg, status_arg) =>
  task_pages.filter((task) =>
    task_filter(task, date_arg, type_arg, status_arg)
  );

const task_map = (task) => ({
  name: get_task_info(task, 'task_name'),
  link: get_task_link(task),
  duration_str: get_task_duration_str(task),
  outcome: dv.pages(`"${task.path}"`).outcome,
  feeling: dv.pages(`"${task.path}"`).feeling,
  datetime: get_task_datetime(task, 'time_start'),
});

const task_sort = (task) => task.datetime;

const task_obj_arr = (date_arg, type_arg, status_arg) =>
  task_page_filter(date_arg, type_arg, status_arg)
    .map(task_map)
    .sort(task_sort);

//-------------------------------------------------------------------
// COUNTING FUNCTIONS
//-------------------------------------------------------------------
const task_count = (date_arg, type_arg, status_arg) =>
  [...new Set(task_obj_arr(date_arg, type_arg, status_arg))].length;

const status_count_bool = (day_int, status_arg) =>
  Boolean(task_count(day_minus_fmt(day_int), 'all', status_arg));

const status_count_arr = (date_arg, type_arg) => {
  const total = task_count(date_arg, type_arg, 'all');
  const done = task_count(date_arg, type_arg, 'done');
  const discard = task_count(date_arg, type_arg, 'discard');
  return { total, done, discard };
};

function calculate_status_rate(status_total, total, round_digits = 2) {
  const fraction = `(${status_total}/${total})`;
  const percentage = total
    ? ((status_total / total) * 100).toFixed(round_digits)
    : '0%';

  return round_digits === 1
    ? percentage
    : [percentage + '%', fraction].join(' ');
}

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

//-------------------------------------------------------------------
// GENERATING FUNCTIONS
//-------------------------------------------------------------------
const generate_task_list = (date_arg, status_arg) => {
  const tasks = task_obj_arr(date_arg, 'all', status_arg);
  const list_root = dv.el('ul', '');
  return tasks.forEach((task) => {
    const task_root = dv.el('ul', task.link, { container: list_root });
    dv.el('li', task.duration_str, { container: task_root });
    const task_review = (review, emoji) =>
      review.length > 0
        ? dv.el('li', `${emoji}: ${review.join('<br>')}`, {
            container: task_root,
          })
        : null;
    task_review(task.outcome, '🏁');
    task_review(task.feeling, '🎭');
  });
};

const generate_table_head = (day_int) => [
  'Type',
  '✅Done (' +
    date_status_rate(day_minus_fmt(day_int), 'all', 1).completion_rate +
    '%)',
  '❌Discard (' +
    date_status_rate(day_minus_fmt(day_int), 'all', 1).discard_rate +
    '%)',
];

const generate_table_rows = (day_int) =>
  type_arg_arr.map(({ key, head }) => [
    head,
    date_status_rate(day_minus_fmt(day_int), key).completion_rate,
    date_status_rate(day_minus_fmt(day_int), key).discard_rate,
  ]);

const generate_sub_header = (status_arg = 'done') =>
  dv.header(2, status_obj_arr.find((x) => x.status === status_arg)?.head) ??
  null;

//-------------------------------------------------------------------
// RENDERING FUNCTIONS
//-------------------------------------------------------------------
const render_task_table = (day_int) =>
  dv.table(generate_table_head(day_int), generate_table_rows(day_int));

function render_head_section(day_int) {
  if (status_count_bool(day_int, 'all')) {
    dv.header(1, day_file_link(day_int));
    render_task_table(day_int);
  }
}

function render_sub_section(day_int, status_arg = 'done') {
  if (status_count_bool(day_int, status_arg)) {
    generate_sub_header(status_arg);
    generate_task_list(day_minus_fmt(day_int), status_arg);
  }
}

function render_day_section(day_int) {
  render_head_section(day_int);
  render_sub_section(day_int, 'done');
  render_sub_section(day_int, 'discard');
}

//-------------------------------------------------------------------
// TASK DATA RENDERING
//-------------------------------------------------------------------
render_day_section(0);
render_day_section(1);
render_day_section(2);
```