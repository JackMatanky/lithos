<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const pillars_dir = "20_pillars/";
const insights_dir = "80_insight/";
const weekly_reflection_dir = "80_insight/95_reflection/02_weekly/";
const cal_week_dir = "10_calendar/12_weeks/";
const goals_dir = "30_goals/";

/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const hr_line = hyphen.repeat(3);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const colon = String.fromCodePoint(0x3a);
const two_percent = String.fromCodePoint(0x25).repeat(2);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

//Text Formatting
const head_lvl = (int) => `${hash.repeat(int)}${space}`;
const yaml_li = (value) => `${new_line}${ul_yaml}"${value}"`;
const link_alias = (file, alias) => ["[[" + file, alias + "]]"].join("|");
const link_tbl_alias = (file, alias) => ["[[" + file, alias + "]]"].join("\\|");
const cmnt_ob_start = `${two_percent}${space}`;
const cmnt_ob_end = `${space}${two_percent}`;
const cmnt_html_start = `${less_than}${excl}${two_hyphen}${space}`;
const cmnt_html_end = `${space}${two_hyphen}${great_than}`;
const tbl_start = `${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end = `${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${hyphen}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${great_than}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_start}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;
const dv_colon = `${colon.repeat(2)}${space}`;

//-------------------------------------------------------------------
// FORMATTING FUNCTIONS
//-------------------------------------------------------------------
const snake_case_fmt = (name) =>
  name.replaceAll(/(\-\s\-)|(\s)|(\-)]/g, "_").toLowerCase();

//-------------------------------------------------------------------
// SET THE FILE'S CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// JOURNAL WRITING DATE AND PREVIOUS DATE
//-------------------------------------------------------------------
const date = moment().format("YYYY-MM-DD");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const prev_date = moment().subtract(1, "days").format("YYYY-MM-DD");

//-------------------------------------------------------------------
// JOURNAL TYPE AND FILE CLASS
//-------------------------------------------------------------------
const full_type_name = "Weekly Reflection Journal";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const long_type_name = `${full_type_name.split(" ")[0]} ${full_type_name.split(" ")[1]}`;
const long_type_value = long_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[1];
const type_value = full_type_value.split("_")[1];
const subtype_name = full_type_name.split(" ")[0];
const subtype_value = full_type_value.split("_")[0];
const file_class = `pdev_${full_type_value.split("_")[2]}`;

//-------------------------------------------------------------------
// SET WEEK
//-------------------------------------------------------------------
const week_file_regex = /\d{4}.W\d{2}$/;
const week_obj_arr = await tp.user.file_name_alias_by_class_type({
  dir: cal_week_dir,
  file_class: "cal_week",
  type: "",
});
const week_obj = await tp.system.suggester(
  (item) => item.key,
  week_obj_arr.filter((f) => f.value.match(week_file_regex)).reverse(),
  false,
  "Review Week?"
);
const week_value = week_obj.value;
const week_name = week_obj.key;
const week_link = `[[${week_value}|${week_name}]]`;

//-------------------------------------------------------------------
// JOURNAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${week_name} ${full_type_name}`;
const partial_title_name = `${week_name} ${long_type_name}`;
const short_title_name = `${week_name} ${type_name}`;
const full_title_value = `${week_value}_${full_type_value}`;
const partial_title_value = `${week_value}_${long_type_value}`;
const short_title_value = `${week_value}_${type_value}`;

const alias_arr = [long_type_name, full_type_name, full_title_name, partial_title_name, short_title_name, full_title_value, partial_title_value, short_title_value];
let file_alias = "";
for (let i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
};

const file_name = partial_title_value;

//-------------------------------------------------------------------
// PILLAR FILE AND FULL NAME
//-------------------------------------------------------------------
const pillar_name = "Mental Health";
const pillar_value = pillar_name.replaceAll(/\s/g, "_").toLowerCase();
const pillar_link = `[[${pillar_value}|${pillar_name}]]`;
const pillar_value_link = yaml_li(pillar_link);

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  "Goal?"
);

//-------------------------------------------------------------------
// RELATED PROJECT FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const project_value = "durational_reviews";
const project_name = "Durational Reviews";
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);

//-------------------------------------------------------------------
// RELATED PARENT TASK FILE NAME, ALIAS, AND LINK
//-------------------------------------------------------------------
const parent_task_value = "weekly_reviews";
const parent_task_name = "Weekly Reviews";
const parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
const parent_task_value_link = yaml_li(parent_task_link);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${full_type_name}${space}Details`;

const journal_info = await tp.user.include_file("90_pdev_journal_info_callout");
const info_week = `${call_ul}Week Review::${space}${week_link}`;

const info = [info_title, call_start, journal_info, call_start, info_week].join(new_line)

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = weekly_reflection_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path != directory) {
   await tp.file.move(`${directory}${file_name}`);
};

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
date: <%* tR += date_value_link %>
pillar: <%* tR += pillar_value_link %>
goal: <%* tR += goal %>
project: <%* tR += project_value_link %>
parent_task: <%* tR += parent_task_value_link %>
subtype: <%* tR += subtype_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_type_name %>

<%* tR += info %>

---

## Recall the Week

### What Happened Last Week?

> [!definition] Last Week's Experiences
>
> 1. Write key points from the week's recollections.
> 2. Write insights from trends arising from the above key points.

#### Key Points of Last Week's Experiences

- **Sunday Recount**::
- **Monday Recount**::
- **Tuesday Recount**::
- **Wednesday Recount**::
- **Thursday Recount**::
- **Friday Recount**::
- **Saturday Recount**::

#### Trends and Insight from Last Week's Experiences

> [!insight] My experiences from last week revolve around…

- **Weekly Recount**::
- **Weekly Recount**::
- **Weekly Recount**::

---

### What Are the Best Experience From Last Week?

> [!definition] Last Week's Best Experiences
>
> 1. Write key points from the week's best experiences.
> 2. Write insights from trends arising from the above key points.

#### Key Points of Last Week's Best Experiences

- **Sunday Best Experience**::
- **Monday Best Experience**::
- **Tuesday Best Experience**::
- **Wednesday Best Experience**::
- **Thursday Best Experience**::
- **Friday Best Experience**::
- **Saturday Best Experience**::

#### Trends and Insight from Last Week's Best Experiences

> [!insight] My best experiences from last week revolve around…

1. **Weekly Best Experience**::
2. **Weekly Best Experience**::
3. **Weekly Best Experience**::

---

### What Are My Achievements From Last Week?

> [!definition] Last Week's Achievements
>
> 1. Write key points from the week's achievement.
> 2. Write insights from trends arising from the above key points.

#### Key Points of Last Week's Achievements

- **Sunday Achievement**::
- **Monday Achievement**::
- **Tuesday Achievement**::
- **Wednesday Achievement**::
- **Thursday Achievement**::
- **Friday Achievement**::
- **Saturday Achievement**::

#### Trends and Insight from Last Week's Achievements

> [!insight] My achievements from last week revolve around…

1. **Weekly Achievement**::
2. **Weekly Achievement**::
3. **Weekly Achievement**::

---

### For What Am I Grateful From Last Week?

> [!definition] Last Week's Gratitude
>
> 1. Write key points from the week's gratitude and self gratitude.
> 2. Write insights from trends arising from the above key points.

#### Key Points of Last Week's Gratitude

- **Sunday Gratitude**::
- **Sunday Self Gratitude**::
- **Monday Gratitude**::
- **Monday Self Gratitude**::
- **Tuesday Gratitude**::
- **Tuesday Self Gratitude**::
- **Wednesday Gratitude**::
- **Wednesday Self Gratitude**::
- **Thursday Gratitude**::
- **Thursday Self Gratitude**::
- **Friday Gratitude**::
- **Friday Self Gratitude**::
- **Saturday Gratitude**::
- **Saturday Self Gratitude**::

#### Trends and Insight from Last Week's Gratitude

> [!insight] My gratitude from last week revolves around…

1. **Weekly Gratitude**::
2. **Weekly Gratitude**::
3. **Weekly Gratitude**::
4. **Weekly Self Gratitude**::
5. **Weekly Self Gratitude**::
6. **Weekly Self Gratitude**::

---

### What Unplanned Occurrences Happened Last Week?

> [!definition] Last Week's Bad Experiences
>
> 1. Write key points from the week's blindspots and detachments.
> 2. Write insights from trends arising from the above key points.

#### Key Points of Last Week's Blind Spots

- **Sunday Blindspot**::
- **Monday Blindspot**::
- **Tuesday Blindspot**::
- **Wednesday Blindspot**::
- **Thursday Blindspot**::
- **Friday Blindspot**::
- **Saturday Blindspot**::

#### Trends and Insight from Last Week's Blind Spots

> [!insight] My blind spots from last week revolve around…

1. **Weekly Blindspot**::
2. **Weekly Blindspot**::
3. **Weekly Blindspot**::

---

### From What Did I Detach Last Week?

> [!definition] Last Week's Detachment
>
> 1. Write key points from the week's detachments.
> 2. Write insights from trends arising from the above key points.

#### Key Points of Last Week's Detachments

- **Sunday Detachment Issue**::
- **Sunday Detachment Solution**::
- **Monday Detachment Issue**::
- **Monday Detachment Solution**::
- **Tuesday Detachment Issue**::
- **Tuesday Detachment Solution**::
- **Wednesday Detachment Issue**::
- **Wednesday Detachment Solution**::
- **Thursday Detachment Issue**::
- **Thursday Detachment Solution**::
- **Friday Detachment Issue**::
- **Friday Detachment Solution**::
- **Saturday Detachment Issue**::
- **Saturday Detachment Solution**::

#### Trends and Insight from Last Week's Detachments

> [!insight] My detachments from last week revolve around…

- **Weekly Detachment Issue**::
- **Weekly Detachment Solution**::
- **Weekly Detachment Issue**::
- **Weekly Detachment Solution**::
- **Weekly Detachment Issue**::
- **Weekly Detachment Solution**::

---

### What Did I Learn Last Week?

> [!definition] Last Week's Lessons Learned
>
> 1. Write key points from the week's lessons learned.
> 2. Write insights from trends arising from the above key points and from the insights from above.

#### Key Points of Last Week's Lessons Learned

- **Sunday Lesson**::
- **Monday Lesson**::
- **Tuesday Lesson**::
- **Wednesday Lesson**::
- **Thursday Lesson**::
- **Friday Lesson**::
- **Saturday Lesson**::

#### Trends and Insight from Last Week's Lessons Learned

1. **Weekly Lesson**::
2. **Weekly Lesson**::
3. **Weekly Lesson**::

---

1. Did I achieve my weekly goal(s)? If not, why? If yes, what went well?
2. What activities, products, people, or actions have contributed to the biggest majority of my results over the past week?
3. What effort, activities, products, or people have consumed my time and energy but did not lead to a lot of results?
4. What could I have done better in order to be more productive and focused?
5. What habits have been essential to my happiness, productivity, focus, and energy?
6. What were the blind spots and unforeseen events that happened from which I should learn?