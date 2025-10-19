<%*
/* ---------------------------------------------------------- */
/*                    FOLDER PATH VARIABLES                   */
/* ---------------------------------------------------------- */
const sys_temp_include_dir = "00_system/06_template_include/";
const pillars_dir = "20_pillars/";
const goals_dir = "30_goals/";
const insights_dir = "80_insight/";
const prompt_journals_dir = "80_insight/98_prompt_journals/";

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
// TEMPLATE FILES TO INCLUDE PATH VARIABLES
//-------------------------------------------------------------------
const pillar_name_alias_preset_mental = "20_03_pillar_name_alias_preset_mental";
const related_project = "40_related_project";
const pdev_journal_info_callout = "90_pdev_journal_info_callout";

//-------------------------------------------------------------------
// FILE CREATION AND MODIFIED DATE
//-------------------------------------------------------------------
const date_created = moment().format("YYYY-MM-DD[T]HH:mm");
const date_modified = moment().format("YYYY-MM-DD[T]HH:mm");

//-------------------------------------------------------------------
// JOURNAL TYPE, SUBTYPE, AND FILE CLASS
//-------------------------------------------------------------------
const full_type_name = "Five Year Vision Prompt Journal";
const full_type_value = full_type_name.replaceAll(/\s/g, "_").toLowerCase();
const type_name = full_type_name.split(" ")[3];
const type_value = full_type_value.split("_")[3];
const subtype_name = "Five Year Vision";
const subtype_value = "vision_five_year";
const file_class = `pdev_${full_type_value.split("_")[4]}`;

//-------------------------------------------------------------------
// SET WRITING DATE
//-------------------------------------------------------------------
const date = await tp.user.nl_date(tp, "");
const date_link = `[[${date}]]`;
const date_value_link = `"${date_link}"`;
const full_date_time = moment(`${date}T00:00`);
const short_date = moment(full_date_time).format("YY-MM-DD");
const short_date_value = moment(full_date_time).format("YY_MM_DD");

//-------------------------------------------------------------------
// JOURNAL TITLES, ALIAS, AND FILE NAME
//-------------------------------------------------------------------
const full_title_name = `${date} ${subtype_name}`;
const short_title_name = subtype_name
  .replaceAll(/[#:\*<>\|\\/-]/g, " ")
  .replaceAll(/\?/g, "")
  .replaceAll(/"/g, "'")
  .toLowerCase();
const short_title_value = short_title_name.replaceAll(/\s/g, "_");
const full_title_value = `${short_date_value}_${short_title_value}`;

const alias_arr = [subtype_name, full_title_name, short_title_name, short_title_value, full_title_value];
let file_alias = "";
for (let i = 0; i < alias_arr.length; i++) {
  alias = yaml_li(alias_arr[i]);
  file_alias += alias;
};

const file_name = full_title_value;

//-------------------------------------------------------------------
// SET PILLAR FILE AND FULL NAME; PRESET MENTAL HEALTH
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pillar_name_alias_preset_mental}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const pillar_value = include_arr[0];
const pillar_value_link = include_arr[1];

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(goals_dir);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${full_type_name}?`
);

//-------------------------------------------------------------------
// SET RELATED PROJECT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${related_project}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString().split(";");

const project_value = include_arr[0];
const project_name = include_arr[1];
const project_link = `[[${project_value}|${project_name}]]`;
const project_value_link = yaml_li(project_link);

//-------------------------------------------------------------------
// SET RELATED PARENT TASK
//-------------------------------------------------------------------
let parent_task_link = "[[null|Null]]";
if (project_value != "null") {
  const parent_task_obj_arr = await tp.user.file_name_alias_by_class_type({
    dir: project_value,
    file_class: "task",
    type: "parent_task",
  });
  const parent_task_obj = await tp.system.suggester(
    (item) => item.key,
    parent_task_obj_arr,
    false,
    `Related Parent Task to the ${full_type_name}?`
);
  parent_task_value = parent_task_obj.value;
  parent_task_name = parent_task_obj.key;
  parent_task_link = `[[${parent_task_value}|${parent_task_name}]]`;
};
const parent_task_value_link = yaml_li(parent_task_link);

//-------------------------------------------------------------------
// PDEV JOURNAL INFO CALLOUT
//-------------------------------------------------------------------
temp_file_path = `${sys_temp_include_dir}${pdev_journal_info_callout}.md`;
abstract_file = await app.vault.getAbstractFileByPath(temp_file_path);
tp_include = await tp.file.include(abstract_file);
include_arr = tp_include.toString();

const journal_info = include_arr;

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const info_title = `${call_start}[!${type_value}]${space}${full_type_name}${space}Details${new_line}${call_start}${new_line}`;

const info = `${info_title}${journal_info}`;

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const directory = prompt_journals_dir;
const folder_path = `${tp.file.folder(true)}/`;

if (folder_path!= directory) {
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
# <%* tR += subtype_name %>

<%* tR += info %>

---

> [!vision] Life Vision
>
> A life vision list is a collection of statements that describe your ideal life and what you want to achieve in various aspects of it. It can help you clarify your values, set your goals, and align your actions with your purpose. Writing a life vision list can be a creative and inspiring process, but it can also be challenging and overwhelming. Here are some steps you can follow to write your own life vision list:
> - Start with free writing. Write down whatever comes to your mind when you think about your life vision. Don’t worry about grammar, structure, or coherence. Just let your thoughts flow and capture them on paper or on a digital device.
> - Review your free writing and identify the main themes or categories that emerge. Some common categories are personal identity, companionship, family, personal growth, career, networks, friends and colleagues, recreation and renewal, spiritual growth, financial stability, health and fitness, and personal legacy. You can use these categories as a guide or create your own based on your free writing.
> - For each category, write a clear and concise statement that summarizes your vision for that aspect of your life. Use positive and affirmative language and avoid words like “don’t”, “can’t”, or “should”. Be specific and realistic, but also ambitious and optimistic. Include details that make your vision vivid and compelling. For example, instead of writing “I want to be healthy”, you could write “I want to exercise regularly, eat nutritious food, and sleep well”.
> - Review your statements and make sure they are consistent with each other and with your core values. If you find any contradictions or conflicts, revise your statements until they are aligned. You can also prioritize your statements according to their importance or urgency for you.
> - Write a summary paragraph that captures the essence of your life vision list. This paragraph should be a brief overview of what you want to achieve in your life and why. You can use this paragraph as a personal vision statement that you can read daily or whenever you need motivation or guidance.
> - Review your life vision list periodically and update it as necessary. Your life vision list is not a static document, but a dynamic one that reflects your growth and changes over time. You can add new statements, modify existing ones, or delete ones that are no longer relevant. You can also track your progress and celebrate your achievements as you move closer to your life vision.

## Personal

1. Personal Identity
	- What issues do you care about?
	- What are your likes and dislikes?
	- What are your strengths and weaknesses?
	- What’s special about you?
	- What do you want to own?
	- What would you most like to accomplish?
	- What are the words, phrases, or labels that you use or want to use to describe yourself? How do they reflect your personality, values, or beliefs?
	- What are the stories, memories, or events that have shaped or influenced your identity? How do they reveal your strengths, weaknesses, or passions?
	- What are the roles or identities that you have or want to have in different contexts or situations? How do they affect your behavior, choices, or relationships?
	- What are the aspects of your identity that you are proud of or want to celebrate? How do you express or share them with others?
	- What are the aspects of your identity that you are unhappy with or want to change? How do you cope with or improve them?

1. Personal Growth
	- How do you plan to enrich your life?
	- What qualities would you like to develop?
	- What are your educational goals?
	- What books would you like to have read and/or add to your library?
	- What types of seminars would you find beneficial?
	- What are the personal qualities, traits, or values that you want to develop or enhance in the next five years? How will you cultivate them and demonstrate them in your life and work?
	- What are the personal challenges, fears, or limitations that you want to overcome or transform in the next five years? How will you face them and learn from them?
	- What are the personal passions, hobbies, or interests that you want to explore or pursue in the next five years? How will you find time and resources for them and enjoy them?
	- What are the personal milestones, achievements, or experiences that you want to celebrate or have in the next five years? How will you plan for them and make them memorable?
	- What are the personal habits, routines, or rituals that you want to establish or maintain in the next five years? How will they improve your productivity, efficiency, or well-being?

1. Recreation and Renewal
	- How do you have fun?
	- Describe your dream vacation

1. Spiritual Growth
	- What steps will you take in order to live more authentically?
	- How are you giving back?
	- Do you set aside one day each week to focus on your spiritual growth and development?
	- Have you scheduled uplifting and spiritually enriching moments into your daily routine?

1. Financial Stability
	- What are your immediate financial needs and goals?
	- Do you have a workable budget? Does it need to be revised?
	- Describe your retirement plan
	- Describe your saving and investment plan

1. Health and Fitness
	- Do you have an exercise program?
	- Do you have a nutritional plan?
	- Do you get sufficient rest?

1. Personal Legacy
	- How do you want to be remembered?
	- What kind of inheritance do you want to leave your children?
	- How will the world know you were here? (What is the footprint you want to leave behind?)

## Interpersonal

1. Companionship (Marriage)
	- What type of person do you want to grow old with?
	- What is your love language?
	- What is your communication style/preference?
	- Describe your ideal “Date Night”
2. Family (Immediate and Extended Family)
	- What are your favorite family traditions?
	- Describe your family values
	- Describe your family’s heritage
	- What is your family’s legacy?
1. Networks
	- Describe your ideal:
		- Friendships
		- Professional Networks
		- Alliances
		- Partnerships

1. Friends and Colleagues
	- What kind of close relationships do you need to develop?
	- Do you have a master-mind group?
	- What does your support system look like?
	- List and describe your ideal mentors

## Professional

1. Career
	- What would you attempt to do if you knew you would never fail?
	- Set aside money for a moment; what do you want in your career?
