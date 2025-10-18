<%*
/* ---------------------------------------------------------- */
/*                          CONSTANTS                         */
/* ---------------------------------------------------------- */

// FOLDER PATH VARIABLES
const DIR = {
  PILLARS: '20_pillars/',
  GOALS: '30_goals/',
  PERSONAL: '41_personal/',
  EDUCATION: '42_education/',
  PROFESSIONAL: '43_professional/',
  WORK: '44_work/',
  HABITS: '45_habit_ritual/',
  CONTACTS: '51_contacts/',
  ORGANIZATIONS: '52_organizations/',
};

const PROJECT_DIRS = [
  DIR.PERSONAL,
  DIR.EDUCATION,
  DIR.PROFESSIONAL,
  DIR.WORK,
  DIR.HABITS,
];

// FILE TYPE AND CLASS
const FILE_TYPES = {
  ACTION_ITEM: {
    name: 'Action Item',
    value: 'action_item',
    class: 'task_child',
  },
};

// FORMATTING CHARACTERS
const CHAR = {
  NEW_LINE: String.fromCodePoint(0xa), // \n
  SPACE: String.fromCodePoint(0x20), //
  HASH: String.fromCodePoint(0x23), // #
  HYPHEN: String.fromCodePoint(0x2d), // -
  BACKTICK: String.fromCodePoint(0x60), // `
  COLON: String.fromCodePoint(0x3a), // :
  PERCENT: String.fromCodePoint(0x25), // %
  PIPE: String.fromCodePoint(0x7c), // |
  LESS_THAN: String.fromCodePoint(0x3c), // <
  GREATER_THAN: String.fromCodePoint(0x3e), // >
  EXCLAMATION: String.fromCodePoint(0x21), // !
  ASTERISK: String.fromCodePoint(0x2a), // *
};

const MD = {
  TWO_SPACES: CHAR.SPACE.repeat(2),
  TWO_NEW_LINES: CHAR.NEW_LINE.repeat(2),
  HR_LINE: CHAR.HYPHEN.repeat(3),
  CODE_FENCE: CHAR.BACKTICK.repeat(3),
  DOUBLE_COLON: CHAR.COLON.repeat(2),
  UL_PREFIX: `${CHAR.HYPHEN}${CHAR.SPACE}`,
  CHECKBOX: `[${CHAR.SPACE}]${CHAR.SPACE}`,
};

const REGEX = {
  SNAKE_CASE_PUNCTUATION: /[':;,]/g,
  SNAKE_CASE_WHITESPACE: /[\s-]+/g,
  WIKI_LINK_WITH_ALIAS: /\[\[([^|\]]+)\|([^\]]+)\]\]/g,
  JOHNNY_DECIMAL_PREFIX: /^\d\d_/,
};

/* ---------------------------------------------------------- */
/*                     COMPONENT BUILDERS                     */
/* ---------------------------------------------------------- */

/* --------------------- MARKDOWN & HTML -------------------- */
const createHeading = (level, text) =>
  `${CHAR.HASH.repeat(level)}${CHAR.SPACE}${text}`;
const formatAsQuote = (content) => `"${content}"`;

/* ------------------- INLINE & BLOCK CODE ------------------ */
const formatAsInlineCode = (content) =>
  `${CHAR.BACKTICK}${content}${CHAR.BACKTICK}`;
const createCodeBlock = (content, language = '') =>
  [`${MD.CODE_FENCE}${language}`, content, MD.CODE_FENCE].join(CHAR.NEW_LINE);

/* ---------------- OBSIDIAN & HTML COMMENTS ---------------- */
const createObsidianComment = (content) => {
  const delimiter = CHAR.PERCENT.repeat(2);
  return `${delimiter}${CHAR.SPACE}${content}${CHAR.SPACE}${delimiter}`;
};
const createHtmlComment = (content) => {
  const opening = `${CHAR.LESS_THAN}${CHAR.EXCLAMATION}${CHAR.HYPHEN.repeat(
    2
  )}${CHAR.SPACE}`;
  const closing = `${CHAR.SPACE}${CHAR.HYPHEN.repeat(2)}${CHAR.GREATER_THAN}`;
  return `${opening}${content}${closing}`;
};

/* ------------------------ WIKILINKS ----------------------- */
const createWikiLink = (filePath, alias = '') =>
  alias ? `[[${filePath}${CHAR.PIPE}${alias}]]` : `[[${filePath}]]`;
const createTblEscapedLink = (filePath, alias) =>
  `[[${filePath}\\${CHAR.PIPE}${alias}]]`;

/* ----------------------- LIST ITEMS ----------------------- */
const createIndentedListPrefix = (indentLevel = 0) =>
  `${CHAR.SPACE.repeat(indentLevel * 2)}${MD.UL_PREFIX}`;
const createListItem = (content, indentLevel = 0) =>
  `${createIndentedListPrefix(indentLevel)}${content}`;
const createCheckboxItem = (content, indentLevel = 0) =>
  `${createIndentedListPrefix(indentLevel)}${MD.CHECKBOX}${content}`;
const createYamlListItem = (content, isLink = false) => {
  const value = formatAsQuote(content);
  return CHAR.NEW_LINE + createListItem(value, 1);
};

/* ------------------------- TABLES ------------------------- */
const createTablePipe = () => `${CHAR.SPACE}${CHAR.PIPE}${CHAR.SPACE}`;
const createTableAligner = (align = 'left', width = 8) => {
  const line = CHAR.HYPHEN.repeat(width);
  if (align === 'center') return `${CHAR.COLON}${line}${CHAR.COLON}`;
  if (align === 'right') return `${CHAR.SPACE}${line}${CHAR.COLON}`;
  return `${CHAR.COLON}${line}${CHAR.SPACE}`;
};

/* ------------------------ CALLOUTS ------------------------ */
const createCalloutHeader = (type, title = '') =>
  `${CHAR.GREATER_THAN}${CHAR.SPACE}[${CHAR.EXCLAMATION}${type}]${
    title ? CHAR.SPACE + title : ''
  }`;
const formatAsCallout = (content) =>
  `${CHAR.GREATER_THAN}${CHAR.SPACE}${content}`;
const createCalloutTableRow = (cells) => {
  const content = cells.join(createTablePipe());
  return formatAsCallout(
    `${CHAR.PIPE}${CHAR.SPACE}${content}${CHAR.SPACE}${CHAR.PIPE}`
  );
};
const createCalloutTableDivider = (columnCount) => {
  const aligners = Array(columnCount).fill(createTableAligner('center'));
  return createCalloutTableRow(aligners);
};

/* ---------------------------------------------------------- */
/*                          DATAVIEW                          */
/* ---------------------------------------------------------- */
const dv = {
  inline: (key, value) => `[${key}${MD.DOUBLE_COLON} ${value}]`,
  yamlProperty: (property) => `this.file.frontmatter.${property}`,
  contentsLink: () => {
    const alias = dv.yamlProperty('aliases[0]');
    const link = `link(this.file.name + "#" + ${alias}, "Contents")`;
    return formatAsInlineCode(`dv: ${link}`);
  },
};

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */

/* ---------------------- OBSIDIAN API ---------------------- */
// Get the first alias from the file's frontmatter
async function getFirstAlias(fileName) {
  if (!fileName || fileName === 'null') return undefined;
  const file = await app.metadataCache.getFirstLinkpathDest(fileName, '');
  if (!file) return;
  const cache = await app.metadataCache.getFileCache(file);
  return cache?.frontmatter?.aliases?.[0];
}

/* -------------------------- DATES ------------------------- */
const formatDate = (format, date, time = '00:00') =>
  moment(`${date}T${time}`).format(format);
const modifyAndFormatDate = (format, unit, value, date, time = '00:00') => {
  const d = moment(`${date}T${time}`);
  return value > 0
    ? d.add(value, unit).format(format)
    : d.subtract(Math.abs(value), unit).format(format);
};

/* ------------------------- UTILITY ------------------------ */
const toCapitalCase = (text) => {
  if (!text) return '';
  return text.charAt(0).toUpperCase() + text.substring(1);
};
const toSnakeCase = (text) =>
  text
    .toLowerCase()
    .replace(REGEX.SNAKE_CASE_PUNCTUATION, '')
    .replace(REGEX.SNAKE_CASE_WHITESPACE, '_');

const toMdExt = (fileName) => `${fileName}.md`;

// Split a semicolon-delimited string into trimmed components
function parseSemicolonValues(input, expectedCount) {
  if (!input) return [];
  const parts = input.split(';').map((s) => s.trim());
  if (expectedCount !== undefined && parts.length < expectedCount) {
    const errorMsg = `Expected ${expectedCount} values but got ${parts.length}`;
    new Notice(`Input error: ${errorMsg} from "${input}"`, 5000);
    throw new Error(errorMsg);
  }
  return parts;
}
function parseWikiLink(text) {
  if (!text) return null;
  const match = REGEX.WIKI_LINK_WITH_ALIAS.exec(text);
  if (!match) return null;
  return { filePath: match[1], alias: match[2] };
}

/* -------------------------- TASKS ------------------------- */
const createTaskText = (text, type, date, time, duration, symbol) => {
  const timeStart = dv.inline('time_start', formatDate('HH:mm', date, time));
  const timeEnd = dv.inline(
    'time_end',
    modifyAndFormatDate('HH:mm', 'minutes', Number(duration), date, time)
  );
  const durationEst = dv.inline('duration_est', Number(duration));
  const timeTracking = [timeStart, timeEnd, durationEst].join(MD.TWO_SPACES);

  let dateLine = `➕ ${formatDate('YYYY-MM-DD')} 📅 ${formatDate(
    'YYYY-MM-DD',
    date,
    time
  )}`;
  if (symbol === 'x') {
    dateLine += ` ✅ ${formatDate('YYYY-MM-DD', date, time)}`;
  }
  const taskLine = `${MD.UL_PREFIX}[${symbol}] #task ${text}_${type} ${timeTracking} ${dateLine}`;
  return [taskLine, MD.HR_LINE].join(CHAR.NEW_LINE) + CHAR.NEW_LINE;
};

/* ---------------------- Title Helpers --------------------- */
// Functions for transforming and formatting titles.
function extractNumberPrefix(libraryValue) {
  // Formats a 3–4 digit prefix as decimal (e.g., "1234" → "1.234")
  const match = libraryValue.match(/^(\d{1,4})/);
  if (!match) return '';
  const rawNumber = match[1];
  return rawNumber.length >= 3
    ? `${rawNumber[0]}.${rawNumber.slice(1)}`
    : rawNumber;
}
function extractProjectSetup(title, titleValue) {
  // Removes "Project Setup" labels from both display and value strings
  const actionName = title.replace('Project Setup: ', '');
  const actionValue = titleValue.replace('proj_', '');
  return { actionName, actionValue };
}
function buildTitleWithContext({
  baseTitle,
  baseValue,
  contextName,
  contextValue,
}) {
  // Assembles a contextual title and short ID from a task and its reference
  const title = [baseTitle, 'for', contextName].join(CHAR.SPACE);
  const shortTitleValue = `${toSnakeCase(baseValue)}_${contextValue}`;
  return { title, shortTitleValue };
}
function getTitlePrefixInfo(title, titleValue) {
  // Extracts the action prefix and chooses "ch" or default suffix
  const actionName = title.split(' ')[0];
  const titlePrefixValue = titleValue.includes('chap')
    ? `${actionName.toLowerCase()}_ch`
    : `${actionName.toLowerCase()}_`;
  return { actionName, titlePrefixValue };
}
async function getContentTypeAndLesson(titleValue, tp) {
  // Determines whether content is a Chapter, Lecture, or Unit and optionally prompts for lesson title
  let contentType = 'Chapter';
  let unitLessonTitle = '';
  if (titleValue === 'watch_course') {
    contentType = 'Lecture';
  } else if (titleValue.endsWith('_course')) {
    contentType = 'Unit';
    unitLessonTitle = await tp.system.prompt(`${contentType} Lesson Title?`);
  }
  return { contentType, unitLessonTitle };
}
async function handleLibraryTitle({
  title,
  titleValue,
  libraryValue,
  libraryName,
  tp,
}) {
  // Builds full and short titles for library items including optional lesson name
  const numberPrefix = extractNumberPrefix(libraryValue);
  const { actionName, titlePrefixValue } = getTitlePrefixInfo(
    title,
    titleValue
  );
  const { contentType, unitLessonTitle } = await getContentTypeAndLesson(
    titleValue,
    tp
  );
  let fullTitle = [actionName, contentType, numberPrefix, libraryName].join(
    CHAR.SPACE
  );
  let shortTitleValue = `${titlePrefixValue}${libraryValue}`;
  if (unitLessonTitle) {
    const formatted = await tp.user.title_case(unitLessonTitle);
    fullTitle += `, ${formatted}`;
    shortTitleValue += `_${toSnakeCase(formatted)}`;
  }
  return { title: fullTitle, shortTitleValue };
}

/* ---------------------------------------------------------- */

/* ---------------------------------------------------------- */
/*                      GENERAL VARIABLES                     */
/* ---------------------------------------------------------- */

/* --------------------- NULL VARIABLES --------------------- */
const nullLink = createWikiLink('null', 'Null');

/* ------------------- FILE PATH VARIABLES ------------------ */
const folderPath = tp.file.folder(true);
const folderPathSplit = folderPath.split('/');

/* --------------------- DATE VARIABLES --------------------- */
const dateCreated = formatDate('YYYY-MM-DD[T]HH:mm');
const dateModified = formatDate('YYYY-MM-DD[T]HH:mm');
const taskDateCreated = formatDate('YYYY-MM-DD');

/* ---------------------------------------------------------- */
/*          TITLE AND PROJECT DIRECTORY OBJECT ARRAYS         */
/* ---------------------------------------------------------- */

/* --------------------- PERSONAL TASKS --------------------- */
const personalTasks = [
  { key: 'Personal Task', value: 'personal_input' },
  {
    key: 'Coaching Assignments',
    value: 'coaching_assignments_input',
    due_do: 'do',
    pillar: 'mental_health',
    project: 'coaching_with_nir_zer',
    parent_task: 'coaching_assignments',
    organization: 'null',
    contact: '[[zer_nir|Nir Zer]]',
  },
  {
    key: 'Collect Prescriptions',
    value: 'collect_prescriptions',
    due_do: 'do',
    pillar: 'physical_health',
    project: 'medical',
    parent_task: 'medical_prescriptions',
    organization: '[[clalit_health_services|Clalit Health Services]]',
    contact: 'null',
  },
  {
    key: 'Create Template',
    value: 'create_template',
    due_do: 'do',
    pillar: 'null',
    project: 'obsidian_workspace_development',
    parent_task: 'template_development',
  },
  {
    key: 'Revise Template',
    value: 'revise_template',
    due_do: 'do',
    pillar: 'null',
    project: 'obsidian_workspace_development',
    parent_task: 'template_development',
  },
  {
    key: 'Organize and Update Tasks for Previous Days',
    value: 'organize_update_previous_days_tasks',
    due_do: 'do',
    pillar: 'null',
    project: 'general_tasks_and_events',
    parent_task: 'general_tasks',
  },
  {
    key: 'House Chores',
    value: 'house_chores',
    due_do: 'do',
    pillar: '[[partner|Partner]]',
    project: 'general_tasks_and_events',
    parent_task: 'house_chores',
  },
  {
    key: 'Pick Up Mail',
    value: 'pick_up_mail',
    due_do: 'do',
    pillar: 'null',
    project: 'general_tasks_and_events',
    parent_task: 'pick_up_mail',
    organization: '[[israel_post|Israel Post]]',
  },
  {
    key: 'Revise Ergogen Config',
    value: 'revise_ergogen_config',
    due_do: 'do',
    pillar: 'null',
    project: 'keyboard_dev',
    parent_task: 'ergogen_pcb_development',
  },
  {
    key: 'Revise ZMK Config',
    value: 'revise_zmk_config',
    due_do: 'do',
    pillar: 'null',
    project: 'keyboard_dev',
    parent_task: 'zmk_keyboard_layout_development',
  },
  {
    key: 'Typing Practice',
    value: 'typing_practice',
    due_do: 'do',
    pillar: 'null',
    project: 'keyboard_dev',
    parent_task: 'learn_keymap_layout',
  },
];

const chores = [
  'Wash Laundry',
  'Hang Laundry',
  'Fold Laundry',
  'Buy Groceries',
  'Cook Dinner',
  'Wash Dishes',
  'Clean the Apartment',
];

/* --------------------- EDUCATION TASKS -------------------- */
const educationTasks = [
  { key: 'Education Task', value: 'education_input' },
  { key: 'Learn Book Chapter', value: 'learn_chapter' },
  { key: 'Review Book Chapter', value: 'review_chapter' },
  { key: 'Learn Course Unit', value: 'learn_course' },
  { key: 'Watch Course Lecture', value: 'watch_course' },
  { key: 'Watch Educational Video', value: 'watch' },
  { key: 'Read Content', value: 'read' },
  {
    key: 'NAYA College Data Science Course Unit',
    value: 'learn_naya_course',
    due_do: 'do',
    pillar: ['knowledge_expansion', 'career_development', 'data_analyst'],
    project: 'course_naya_college_practical_data_science',
  },
  {
    key: 'Create Anki Cards for Course',
    value: 'create_anki_cards_for_course',
    due_do: 'do',
    pillar: 'knowledge_expansion',
  },
  {
    key: 'Learn Anki Cards',
    value: 'learn_anki_cards',
    due_do: 'do',
    pillar: 'knowledge_expansion',
    project: 'spaced_repetition_learning',
    parent_task: 'spaced_repetition_learning_with_anki',
  },
];

/* ------------------ // PROFESSIONAL TASKS ----------------- */
const professionalTasks = [
  { key: 'Professional Task', value: 'professional_input' },
  {
    key: 'First Job Application Assignment',
    value: 'first_job_assignment',
    due_do: 'due',
    project: 'job_hunting_2023',
  },
  {
    key: 'Daily LinkedIn Job Search',
    value: 'daily_linkedin_job_search',
    due_do: 'do',
    pillar: 'career_development',
    project: 'job_hunting_2023',
    parent_task: 'daily_job_search_2023',
    organization: '[[linkedin|LinkedIn]]',
  },
];

/* ----------------------- WORK TASKS ----------------------- */
const workTasks = [
  { key: 'Work Task', value: 'work_input' },
  {
    key: 'Task for Hive Urban',
    value: 'hive_urban_input',
    pillar: 'data_analyst',
    project: 'hive_research_assistant',
    organization: '[[hive|Hive]]',
  },
];

/* ------------------- PROJECT SETUP TASKS ------------------ */
const projectSetupTasks = [
  {
    key: 'Project Setup: Write Objective',
    value: 'proj_setup_objective',
    due_do: 'do',
  },
  {
    key: 'Project Setup: Create Parent Tasks',
    value: 'proj_setup_parent_tasks',
    due_do: 'do',
  },
  {
    key: 'Project Setup: Gather Resources',
    value: 'proj_setup_resources',
    due_do: 'do',
  },
];

// Sort title object array
const sortedTaskTypes = [
  ...personalTasks,
  ...educationTasks,
  ...professionalTasks,
  ...workTasks,
  ...projectSetupTasks,
].sort((a, b) => a.key.localeCompare(b.key));

const allTaskTypes = [
  { key: 'User Input', value: '_user_input' },
  ...sortedTaskTypes,
];

/* ---------------------------------------------------------- */
/*                      SET FILE'S TITLE                      */
/* ---------------------------------------------------------- */
let titleObj;
let title;

if (tp.file.title.startsWith('Untitled')) {
  titleObj = await tp.system.suggester(
    (item) => item.key,
    allTaskTypes,
    false,
    `${FILE_TYPES.ACTION_ITEM.name} Title?`
  );
  if (!titleObj)
    throw new Error('Template execution cancelled by user at title selection.');
  title = titleObj.key;
} else {
  title = tp.file.title;
  titleObj = allTaskTypes.find((t) => t.key === title) || {
    value: toSnakeCase(title),
  };
}

if (titleObj.value === 'house_chores') {
  title = await tp.system.suggester(chores, chores, false, 'House Chore?');
  if (!title)
    throw new Error('Template execution cancelled by user at chore selection.');
} else if (titleObj.value.endsWith('_template')) {
  const templateTitle = await tp.system.prompt(
    `Template to ${titleObj.value.split('_')[0]}?`,
    null,
    true,
    false
  );
  if (!templateTitle)
    throw new Error(
      'Template execution cancelled by user at template title prompt.'
    );
  const titleParts = title.split(' ');
  title = `${titleParts[0]} ${templateTitle.trim()} ${titleParts[1]}`;
} else if (titleObj.value.endsWith('_input')) {
  title = await tp.system.prompt(
    `${FILE_TYPES.ACTION_ITEM.name} Title?`,
    null,
    true,
    false
  );
  if (!title)
    throw new Error(
      'Template execution cancelled by user at custom title prompt.'
    );
}

title = await tp.user.title_case(title.trim());
const selectedTask = allTaskTypes.find((t) => t.value === titleObj.value) || {};

/* ---------------------------------------------------------- */
/* RESOLVE CONTEXT, DATES, AND PROPERTIES           */
/* ---------------------------------------------------------- */
/* ---------------------------------------------------------- */
/*                       RESOLVE CONTEXT                      */
/* ---------------------------------------------------------- */
const contextMap = {
  personal: personalTasks,
  education: educationTasks,
  professional: professionalTasks,
  work: workTasks,
};

const getContextFromTaskMap = (taskValue) =>
  Object.entries(contextMap).find(([, tasks]) =>
    tasks.some((t) => t.value === taskValue)
  )?.[0] || null;

const getContextFromPath = (pathString) => {
  if (!pathString) return null;
  const pathParts = pathString.split('/');
  if (pathParts.length < 1) return null;
  const pathContext = pathParts[0].replace(REGEX.JOHNNY_DECIMAL_PREFIX, '');
  const validContexts = PROJECT_DIRS.map((p) =>
    p.replace(REGEX.JOHNNY_DECIMAL_PREFIX, '').replace('/', '')
  );
  return validContexts.includes(pathContext) ? pathContext : null;
};
let contextValue =
  getContextFromTaskMap(titleObj.value) || getContextFromPath(folderPath);

/* ---------------------------------------------------------- */
/*                  SET DATE, TIME & DURATION                 */
/* ---------------------------------------------------------- */
const date = await tp.user.nl_date(tp, 'start');
const time = await tp.user.nl_time(
  tp,
  `${FILE_TYPES.ACTION_ITEM.name} Start Time?`
);
const durationMin = await tp.user.durationMin(tp);

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const { link: pillarYaml } = selectedTask.pillar
  ? {
      link: Array.isArray(selectedTask.pillar)
        ? selectedTask.pillar
            .map((p) => createYamlListItem(createWikiLink(p)))
            .join('')
        : createYamlListItem(createWikiLink(selectedTask.pillar)),
    }
  : await tp.user.multi_suggester({
      tp,
      items: await tp.user.file_by_status({
        dir: DIR.PILLARS,
        status: 'active',
      }),
      type: 'pillar',
    });

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goal =
  selectedTask.goal ||
  (await tp.system.suggester(
    await tp.user.md_file_name(DIR.GOALS),
    await tp.user.md_file_name(DIR.GOALS),
    false,
    `${FILE_TYPES.ACTION_ITEM.name} Goal?`
  ));

/* ---------------------------------------------------------- */
/*            SET PROJECT BY FILE PATH OR SUGGESTER           */
/* ---------------------------------------------------------- */
let projectValue, projectName;
if (selectedTask.project) {
  projectValue = selectedTask.project;
  projectName =
    (await getFirstAlias(projectValue)) ||
    toCapitalCase(projectValue.replace(/_/g, ' '));
} else {
  let projects;
  const projectDirPath = contextValue ? DIR[contextValue.toUpperCase()] : null;
  if (projectDirPath) {
    projects = await tp.user.file_name_alias_by_class_type_status({
      dir: projectDirPath,
      file_class: 'task',
      type: 'project',
      status: 'active',
    });
  } else {
    projects = [{ key: 'Null', value: 'null' }];
    for (const dir of PROJECT_DIRS) {
      const projectsInDir = await tp.user.file_name_alias_by_class_type({
        dir: dir,
        file_class: 'task',
        type: 'project',
      });
      projects.push(
        ...projectsInDir.filter(
          (p) => p.value && p.value !== 'null' && p.value !== '_user_input'
        )
      );
    }
  }
  const projectObj = await tp.system.suggester(
    (p) => p.key,
    projects,
    false,
    'Project?'
  );
  if (!projectObj)
    throw new Error(
      'Template execution cancelled by user at project selection.'
    );
  projectValue = projectObj.value;
  projectName = projectObj.key;
}
const projectYaml = createYamlListItem(
  createWikiLink(projectValue, projectName),
  true
);
const projectFilePath = await tp.user.getFilePath(projectValue);

if (!projectFilePath && projectValue !== 'null') {
  throw new Error(
    `Could not find the file path for the selected project: "${projectName}". Please ensure the file exists.`
  );
}

const projectDir = projectFilePath
  ? projectFilePath.replace(`/${projectValue}.md`, '')
  : '';

/* --------------------- Resolve Context -------------------- */
// Re-evaluate context if it was not found before project selection
if (!contextValue && projectDir) {
  contextValue = getContextFromPath(projectDir);
}

/* ---------------------------------------------------------- */
/*          SET PARENT TASK BY FILE PATH OR SUGGESTER         */
/* ---------------------------------------------------------- */
let parentTaskValue, parentTaskName;
if (selectedTask.parent_task) {
  parentTaskValue = selectedTask.parent_task;
  parentTaskName =
    (await getFirstAlias(parentTaskValue)) ||
    toCapitalCase(parentTaskValue.replace(/_/g, ' '));
} else {
  const parentTasks = await tp.user.file_name_alias_by_class_type({
    dir: projectValue,
    file_class: 'task',
    type: 'parent_task',
  });
  const parentTaskObj = await tp.system.suggester(
    (pt) => pt.key,
    parentTasks,
    false,
    'Parent Task?'
  );
  if (!parentTaskObj)
    throw new Error(
      'Template execution cancelled by user at parent task selection.'
    );
  parentTaskValue = parentTaskObj.value;
  parentTaskName = parentTaskObj.key;
}
const parentYaml = createYamlListItem(
  createWikiLink(parentTaskValue, parentTaskName),
  true
);
const parentTaskFilePath =
  projectDir && parentTaskValue
    ? `${projectDir}/${parentTaskValue}/${parentTaskValue}.md`
    : '';

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
let organizationYaml;

if (selectedTask.organization) {
  organizationYaml = createYamlListItem(selectedTask.organization, true);
} else if (
  titleObj.value.endsWith('job_assignment') ||
  titleObj.value === 'interview_prep'
) {
  const parentFileCache = await app.metadataCache.getFileCache(
    app.vault.getAbstractFileByPath(parentTaskFilePath)
  );
  organizationYaml = createYamlListItem(
    parentFileCache?.frontmatter?.organization,
    true
  );
} else {
  const { link } = await tp.user.multi_suggester({
    tp,
    items: await tp.user.md_file_name_alias(DIR.ORGANIZATIONS),
    type: 'organization',
  });
  if (!link)
    throw new Error(
      'Template execution cancelled by user at organization selection.'
    );
  organizationYaml = link;
}

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
let contactYaml;

if (selectedTask.contact) {
  contactYaml = createYamlListItem(selectedTask.contact, true);
} else {
  const { link } = await tp.user.multi_suggester({
    tp,
    items: await tp.user.md_file_name_alias(DIR.CONTACTS),
    type: 'contact',
  });
  if (!link)
    throw new Error(
      'Template execution cancelled by user at contact selection.'
    );
  contactYaml = link;
}

/* ---------------------------------------------------------- */
/*               SET LIBRARY FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
let libraryValue = 'null',
  libraryName = 'Null';

if (titleObj.value.endsWith('_chapter') || titleObj.value.endsWith('_course')) {
  const sourceFilePath =
    parentTaskValue !== 'null' ? parentTaskFilePath : projectFilePath;
  const sourceFileCache = await app.metadataCache.getFileCache(
    app.vault.getAbstractFileByPath(sourceFilePath)
  );
  const libraryLink = sourceFileCache?.frontmatter?.library?.[0];
  if (libraryLink) {
    const parsed = parseWikiLink(libraryLink);
    libraryValue = parsed.filePath;
    libraryName = parsed.alias;
  }
}
const libraryYaml = createYamlListItem(
  createWikiLink(libraryValue, libraryName),
  true
);

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const dueDoValue =
  selectedTask.due_do ||
  parseSemicolonValues(
    await tp.user.include_template(tp, '40_task_do_due_date')
  )[0];

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const [statusValue, statusName, statusSymbol] = parseSemicolonValues(
  await tp.user.include_template(tp, '42_00_child_task_status')
);

/* ---------------------------------------------------------- */
/*          FRONTMATTER TITLE, ALIASES, AND FILE NAME         */
/* ---------------------------------------------------------- */
let finalTitle = title;
let finalShortTitleValue = toSnakeCase(title);

if (titleObj.value.startsWith('proj_setup_')) {
  const { actionName, actionValue } = extractProjectSetup(
    title,
    titleObj.value
  );
  finalTitle = `${actionName} for ${projectName}`;
  finalShortTitleValue = `${toSnakeCase(actionValue)}_${projectValue}`;
} else if (titleObj.value === 'create_anki_cards_for_course') {
  finalTitle = title.replace('Course', projectName);
  finalShortTitleValue = titleObj.value.replace('course', projectValue);
} else if (
  titleObj.value.endsWith('_chapter') ||
  titleObj.value.endsWith('_course') ||
  titleObj.value === 'watch_course'
) {
  const result = await handleLibraryTitle({
    title,
    titleValue: titleObj.value,
    libraryValue,
    libraryName,
    tp,
  });
  finalTitle = result.title;
  finalShortTitleValue = result.shortTitleValue;
}

const fullTitleName = `${formatDate('YY-MM-DD', date)} ${finalTitle}`;
const fullTitleValue = `${formatDate(
  'YY_MM_DD',
  date
)}_${finalShortTitleValue}`;
const fileAlias = [
  finalTitle,
  fullTitleName,
  finalTitle.toLowerCase(),
  finalShortTitleValue,
  fullTitleValue,
]
  .map((a) => createYamlListItem(a))
  .join('');

const fileName = fullTitleValue;
const fileSection = fileName + CHAR.HASH;

/* ---------------------------------------------------------- */
/*            ACTION ITEM PREVIEW, PLAN, AND REVIEW           */
/* ---------------------------------------------------------- */
const previewReviewFileMap = {
  default: '42_00_action_item_preview_review',
  interview_prep: '42_01_act_pre_interview_preview_review',
  typing_practice: '42_02_act_typing_preview_review',
  daily_linkedin_job_search: '42_03_act_job_search_preview_review',
  _chapter: '42_21_act_preview_review_ed_book_chapter',
};

let previewReviewFile =
  previewReviewFileMap[titleObj.value] ||
  (titleObj.value.endsWith('_chapter')
    ? previewReviewFileMap['_chapter']
    : previewReviewFileMap.default);

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const sectionObjArr = [
  {
    headKey: 'Tasks and Events',
    tocLevel: 1,
    tocKey: 'Tasks and Events',
    file: null,
  },
  {
    headKey: 'Prepare and Reflect',
    tocLevel: 1,
    tocKey: 'Insight',
    file: previewReviewFile,
  },
  {
    headKey: 'Related Tasks and Events',
    tocLevel: 1,
    tocKey: 'Related Tasks',
    file: '142_00_related_sect_task_child',
  },
  {
    headKey: 'Related Knowledge',
    tocLevel: 2,
    tocKey: 'PKM',
    file: '100_70_related_pkm_sect',
  },
  {
    headKey: 'Related Library Content',
    tocLevel: 2,
    tocKey: 'Library',
    file: '100_60_related_lib_sect',
  },
  {
    headKey: 'Related Directory',
    tocLevel: 2,
    tocKey: 'Directory',
    file: '100_50_related_dir_sect',
  },
];

// Content, heading, and table of contents link
for (let i = 0; i < sectionObjArr.length; i++) {
  sectionObjArr[i].content =
    i === 0
      ? createTaskText(
          finalTitle,
          FILE_TYPES.ACTION_ITEM.value,
          date,
          time,
          durationMin,
          statusSymbol
        )
      : await tp.user.include_template(tp, sectionObjArr[i].file);
  sectionObjArr[i].head = createHeading(2, sectionObjArr[i].headKey);
  sectionObjArr[i].toc = createTblEscapedLink(
    fileSection + sectionObjArr[i].headKey,
    sectionObjArr[i].tocKey
  );
  console.log(`${sectionObjArr[i].head} section complete`)
}

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const createTocLevelRow = (level) =>
  createCalloutTableRow(
    sectionObjArr.filter((s) => s.tocLevel === level).map((s) => s.toc)
  );

const toc = [
  createCalloutHeader('toc', dv.contentsLink()),
  formatAsCallout(''),
  createTocLevelRow(1),
  createCalloutTableDivider(3),
  createTocLevelRow(2),
].join(CHAR.NEW_LINE);

/* ---------------------------------------------------------- */
/*                        FILE SECTIONS                       */
/* ---------------------------------------------------------- */
const sectionsContent = sectionObjArr
  .map((s) =>
    (s.file ? [s.head, toc, s.content] : [s.head, s.content]).join(
      MD.TWO_NEW_LINES
    )
  )
  .join(MD.TWO_NEW_LINES);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const infoFileMap = {
  default: '42_00_action_info_callout',
  _chapter: '42_21_act_ed_book_ch_info_callout',
  _course: '42_22_act_ed_course_lect_info_callout',
};
const infoFileKey =
  Object.keys(infoFileMap).find(
    (suffix) => suffix !== 'default' && titleObj.value.endsWith(suffix)
  ) || 'default';
const infoCallout = await tp.user.include_file(infoFileMap[infoFileKey]);

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const destinationDir =
  parentTaskValue === 'null' || !projectDir
    ? `${projectDir}/`
    : `${projectDir}/${parentTaskValue}/`;
if (folderPath !== destinationDir) {
  await tp.file.move(`${destinationDir}${fileName}`);
}

/* ---------------------------------------------------------- */
/*                 YAML FRONTMATTER PROPERTIES                */
/* ---------------------------------------------------------- */
const mainYamlProperties = [
  `title: ${fileName}`,
  `uuid: ${await tp.user.uuid()}`,
  `aliases: ${fileAlias}`,
  `date: ${formatAsQuote(createWikiLink(date))}`,
  `due_do: ${dueDoValue}`,
  `pillar: ${pillarYaml}`,
  `context: ${contextValue}`,
  `goal: ${goal}`,
  `project: ${projectYaml}`,
  `parent_task: ${parentYaml}`,
  `organization: ${organizationYaml}`,
  `contact: ${contactYaml}`,
  `library: ${libraryYaml}`,
  `type: ${FILE_TYPES.ACTION_ITEM.value}`,
  `file_class: ${FILE_TYPES.ACTION_ITEM.class}`,
  `date_created: ${dateCreated}`,
  `date_modified: ${dateModified}`,
  `tags:`,
];

const mainYaml = [MD.HR_LINE, ...mainYamlProperties, MD.HR_LINE].join(
  CHAR.NEW_LINE
);

const mainFileContent = [
  mainYaml,
  createHeading(1, finalTitle),
  infoCallout,
  sectionsContent,
].join(MD.TWO_NEW_LINES);

tR += mainFileContent;
%>