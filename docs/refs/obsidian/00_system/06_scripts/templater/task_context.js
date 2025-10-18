const task_context_obj_arr = [
  {
    key: "Personal",
    value: "personal",
    directory: "41_personal/",
  },
  {
    key: "Habits and Rituals",
    value: "habit_ritual",
    directory: "45_habit_ritual/",
  },
  {
    key: "Education",
    value: "education",
    directory: "42_education/",
  },
  {
    key: "Professional",
    value: "professional",
    directory: "43_professional/",
  },
  {
    key: "Work",
    value: "work",
    directory: "44_work/",
  },
];

async function task_context(tp) {
  const task_context_obj = await tp.system.suggester(
    (item) => item.key,
    task_context_obj_arr,
    false,
    "Task Context?"
  );

  return task_context_obj;
}

module.exports = task_context;
