# Totals

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🌀Date,
    plan AS Plan,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN plan_total AS plan
FLATTEN due_total AS due
FLATTEN done_total AS done
FLATTEN due - done AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-17T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-23T23:59)
SORT
    file.frontmatter.date ASC
```

# Habits and Rituals

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🤖Date,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_habit_rit AS due
FLATTEN done_habit_rit AS done
FLATTEN discard_habit_rit AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```

# Specific Habits and Rituals

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🦿Habits,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_habit AS due
FLATTEN done_habit AS done
FLATTEN discard_habit AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🍵Morning,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_morning_rit AS due
FLATTEN done_morning_rit AS done
FLATTEN discard_morning_rit AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🌇Startup,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_startup_rit AS due
FLATTEN done_startup_rit AS done
FLATTEN discard_startup_rit AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🌆Shutdown,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_shutdown_rit AS due
FLATTEN done_shutdown_rit AS done
FLATTEN discard_shutdown_rit AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🛌Evening,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_evening_rit AS due
FLATTEN done_evening_rit AS done
FLATTEN discard_evening_rit AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```

# Tasks and Events

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS ⚒️Date,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_task_event AS due
FLATTEN done_task_event AS done
FLATTEN due - done AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-17T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-23T23:59)
SORT
    file.frontmatter.date ASC
```

# Specific Tasks and Events

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🔨Action,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_action AS due
FLATTEN done_action AS done
FLATTEN discard_action + reschedule_action AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```

```dataview
TABLE WITHOUT ID
    file.frontmatter.date AS 🤝Event,
    due AS 📆Due,
    done AS ✅Done,
    discard AS ❌Discard,
    string(round((done/due) * 100, 2)) + "%" AS "✔️Comp %",
    string(round((discard/due) * 100, 2)) + "%" AS "🗑️Disc %"
FROM
    "10_calendar"
FLATTEN due_event AS due
FLATTEN done_event AS done
FLATTEN discard_event + reschedule_event AS discard
WHERE
    contains(file.frontmatter.file_class, "cal_day")
    AND date(file.frontmatter.date) >= date(2023-12-10T00:00)
    AND date(file.frontmatter.date) <= date(2023-12-16T23:59)
SORT
    file.frontmatter.date ASC
```
