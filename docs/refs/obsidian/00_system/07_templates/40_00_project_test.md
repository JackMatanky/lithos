<%*
/* ---------------------------------------------------------- */
/*                          CONSTANTS                         */
/* ---------------------------------------------------------- */
const FILE_TYPES = {
  PROJECT: {
    name: 'Project',
    value: 'project',
    class: 'task_project',
  },
  PARENT_TASK: {
    name: 'Parent Task',
    value: 'parent_task',
    class: 'task_parent',
  },
};

const DIR = {
  PILLARS: '20_pillars/',
  GOALS: '30_goals/',
  CONTACTS: '51_contacts/',
  ORGANIZATIONS: '52_organizations/',
  PERSONAL: '41_personal/',
  EDUCATION: '42_education/',
  PROFESSIONAL: '43_professional/',
  WORK: '44_work/',
  HABITS: '45_habit_ritual/',
};

const CHAR = {
  NEW_LINE: String.fromCodePoint(0xa),        // \n
  SPACE: String.fromCodePoint(0x20),          //
  HASH: String.fromCodePoint(0x23),           // #
  HYPHEN: String.fromCodePoint(0x2d),         // -
  BACKTICK: String.fromCodePoint(0x60),       // `
  COLON: String.fromCodePoint(0x3a),          // :
  PERCENT: String.fromCodePoint(0x25),        // %
  PIPE: String.fromCodePoint(0x7c),           // |
  LESS_THAN: String.fromCodePoint(0x3c),      // <
  GREATER_THAN: String.fromCodePoint(0x3e),   // >
  EXCLAMATION: String.fromCodePoint(0x21),    // !
  ASTERISK: String.fromCodePoint(0x2a),       // *
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
};

/* ---------------------------------------------------------- */
/*                     COMPONENT BUILDERS                     */
/* ---------------------------------------------------------- */

/* --------------------- MARKDOWN & HTML -------------------- */
const createHeading = (level, text) =>
  `${CHAR.HASH.repeat(level)}${CHAR.SPACE}${text}`;
const formatAsQuote = (content) => `"${content}"`;

/* ------------------ INLINE AND BLOCK CODE ----------------- */
const formatAsInlineCode = (content) =>
  `${CHAR.BACKTICK}${content}${CHAR.BACKTICK}`;
const createCodeBlock = (content, language = '') =>
  [`${MD.CODE_FENCE}${language}`, content, MD.CODE_FENCE].join(
    CHAR.NEW_LINE
  );

/* ---------------- OBSIDIAN & HTML COMMENTS ---------------- */
const createObsidianComment = (content) =>
  `${CHAR.PERCENT.repeat(2)}${CHAR.SPACE}${content}${
    CHAR.SPACE
  }${CHAR.PERCENT.repeat(2)}`;
const createHtmlComment = (content) =>
  `${CHAR.LESS_THAN}${CHAR.EXCLAMATION}${CHAR.HYPHEN.repeat(2)}${
    CHAR.SPACE
  }${content}${CHAR.SPACE}${CHAR.HYPHEN.repeat(2)}${CHAR.GREATER_THAN}`;

/* ----------------------- WIKI LINKS ----------------------- */
const createWikiLink = (filePath, alias = '') =>
  alias ? `[[${filePath}${CHAR.PIPE}${alias}]]` : `[[${filePath}]]`;
const createTblEscapedLink = (filePath, alias) =>
  `[[${filePath}\\${CHAR.PIPE}${alias}]]`;

/* ----------------------- LIST ITEMS ----------------------- */
const createListItem = (content, indentLevel = 0) =>
  `${CHAR.SPACE.repeat(indentLevel * 2)}${MD.UL_PREFIX}${content}`;
const createCheckboxItem = (content, indentLevel = 0) =>
  `${CHAR.SPACE.repeat(indentLevel * 2)}${MD.UL_PREFIX}${
    MD.CHECKBOX
  }${content}`;
const createYamlListItem = (content, isLink = false) => {
  const value = isLink ? content : formatAsQuote(content);
  return createListItem(value, 1);
};

/* ------------------------- TABLES ------------------------- */
const createTableAlignerLeft = (width = 8) =>
  `${CHAR.COLON}${CHAR.HYPHEN.repeat(width)}${CHAR.SPACE}`;
const createTableAlignerRight = (width = 8) =>
  `${CHAR.SPACE}${CHAR.HYPHEN.repeat(width)}${CHAR.COLON}`;
const createTableAlignerCenter = (width = 8) =>
  `${CHAR.COLON}${CHAR.HYPHEN.repeat(width)}${CHAR.COLON}`;

/**
 * Creates a table column aligner string based on the specified
 * alignment and width.
 * @param {string} align The alignment type for the column
 *                         ('left', 'center', or 'right').
 * @param {number} width The width of the aligner in characters.
 * @returns {string} A string representing the table column aligner.
 */
const createTableAligner = (align = 'left', width = 8) => {
  switch (align) {
    case 'center':
      return createTableAlignerCenter(width);
    case 'right':
      return createTableAlignerRight(width);
    case 'left':
    default:
      return createTableAlignerLeft(width);
  }
};

/* ------------------------ CALLOUTS ------------------------ */
/**
 * Creates a callout header string.
 * @param {string} type The type of the callout
 *                      (e.g. 'note', 'info', etc.).
 * @param {string} [title=''] The title of the callout.
 * @returns {string} A string representing the callout header.
 */
const createCalloutHeader = (type, title = '') =>
  `${CHAR.GREATER_THAN}${CHAR.SPACE}[${CHAR.EXCLAMATION}${type}]${
    title ? CHAR.SPACE + title : ''
  }`;
/**
 * Formats a string as a callout.
 * @param {string} content The string to format.
 * @returns {string} The formatted string.
 */
const formatAsCallout = (content) =>
  `${CHAR.GREATER_THAN}${CHAR.SPACE}${content}`;

// >>> TABLE CALLOUT <<<
/**
 * Creates a row for a callout table.
 * @param {Array<string>} cells An array of strings representing
 *                              the cell contents.
 * @returns {string} A formatted string representing a row in a
 *                   callout table.
 */
const createCalloutTableRow = (cells) => {
  const content = cells.join(createTablePipe());
  return formatAsCallout(
    `${CHAR.PIPE}${CHAR.SPACE}${content}${CHAR.SPACE}${CHAR.PIPE}`
  );
};

/**
 * Creates a divider row for a callout table.
 * @param {number} columnCount The number of columns in the table.
 * @returns {string} A string representing the divider row.
 */
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
/**
 * Finds the file path for a given file name.
 * @param {string} fileName - The name of the markdown file (without extension).
 * @returns {string|undefined} The file path if available, otherwise undefined.
 */
async function findFilePath(fileName) {
  const file = app.vault.getFiles().find((f) => f.name === `${fileName}.md`);
  return file?.path;
}

/**
 * Retrieves the first alias from the frontmatter of a markdown file.
 * @param {string} fileName - The name of the markdown file (without extension).
 * @returns {string|undefined} The first alias if available, otherwise undefined.
 */
async function getFirstAlias(fileName) {
  const file = app.metadataCache.getFirstLinkpathDest(fileName, '');
  if (!file) return;
  const cache = app.metadataCache.getFileCache(file);
  return cache?.frontmatter?.aliases?.[0];
}

/* -------------------------- DATES ------------------------- */

/**
 * Formats a date according to the given format string.
 * @param {string} format - The moment format string
 * @param {string} date - The date in YYYY-MM-DD format
 * @param {string} [time='00:00'] - The time in HH:mm format
 * @returns {string} - The formatted date string
 */
const formatDate = (format, date, time = '00:00') =>
  moment(`${date}T${time}`).format(format);

/**
 * Modifies a date by adding or subtracting a given value of
 * a given unit and formats it.
 * @param {string} format - the moment format string
 * @param {string} unit - the moment unit to modify the date by.
 *                        Can be 'years', 'months', 'weeks', 'days',
 *                        'hours', 'minutes', 'seconds',
 *						 'milliseconds'
 * @param {number} value - the value to add or subtract
 * @param {string} date - the date in YYYY-MM-DD format
 * @param {string} [time='00:00'] - the time in HH:mm format
 * @returns {string} - the modified date formatted according to the
 *                     provided format string
 */
const modifyAndFormatDate = (format, unit, value, date, time = '00:00') => {
  const d = moment(`${date}T${time}`);
  return value > 0
    ? d.add(value, unit).format(format)
    : d.subtract(Math.abs(value), unit).format(format);
};

/* ------------------------- UTILITY ------------------------ */
/**
 * Converts the first character of a string to uppercase and
 * the rest to lowercase.
 * @param {string} text - The input string to be converted.
 * @returns {string} - The converted string with the first character in uppercase.
 */
const toCapitalCase = (text) => {
  if (!text) return '';
  return text.charAt(0).toUpperCase() + text.substring(1);
};

/**
 * Converts a given string to snake_case.
 * @param {string} text - The input string to be converted.
 * @returns {string} - The converted string in snake_case.
 */
const toSnakeCase = (text) =>
  text
    .toLowerCase()
    .replace(REGEX.SNAKE_CASE_PUNCTUATION, '')
    .replace(REGEX.SNAKE_CASE_WHITESPACE, '_');

const toMdExt = (fileName) => `${fileName}.md`;

/**
 * Splits a semicolon-delimited string into trimmed components.
 * If the expectedCount is provided, checks that the input string
 * splits into the expected number of values. If not, throws an error.
 * @param {string} input - the input string
 * @param {number} [expectedCount] - the expected number of semicolon
 *                                   delimited values
 * @returns {string[]} - the array of trimmed values
 * @throws {Error} - if the input string does not split into the
 *                   expected number of values
 */
function parseSemicolonValues(input, expectedCount) {
  if (!input) return [];
  const parts = input.split(';').map((s) => s.trim());
  if (expectedCount !== undefined && parts.length < expectedCount) {
    expected = `${expectedCount} values`;
    actual = parts.length;
    origin = `from "${input}"`;
    errorMsg = `${expected} but got ${actual}`;
    new Notice(`Input error: ${errorMsg} ${origin}`, 5000);
    throw new Error(`Expected ${errorMsg}`);
  }
  return parts;
}

/**
 * Parses a string containing a wikilink with an alias.
 * @param {string} text - The string containing the link.
 * @returns {{filePath: string, alias: string}|null} An object with parts
 *                                                   or null if no match.
 */
function parseWikiLink(text) {
  const match = REGEX.WIKI_LINK_WITH_ALIAS.exec(text);
  if (!match) return null;
  return {
    filePath: match[1],
    alias: match[2],
  };
}

/* ---------------------------------------------------------- */
/*                      GENERAL VARIABLES                     */
/* ---------------------------------------------------------- */

/* --------------------- NULL VARIABLES --------------------- */
const NULL_LINK = createWikiLink('null', 'Null');
const NULL_ARR = ['', 'null', NULL_LINK, null];

/* ------------------- FILE PATH VARIABLES ------------------ */
const folderPath = tp.file.folder(true);
const folderPathSplit = folderPath.split('/');

/* --------------------- DATE VARIABLES --------------------- */
const dateCreated = formatDate('YYYY-MM-DD[T]HH:mm');
const dateModified = formatDate('YYYY-MM-DD[T]HH:mm');

/* ---------------------------------------------------------- */
/*                       Set File Title                       */
/* ---------------------------------------------------------- */
let title = tp.file.title.startsWith('Untitled')
  ? await tp.system.prompt(
      `${FILE_TYPES.PROJECT.name} Title`,
      null,
      true,
      false
    )
  : tp.file.title;
title = await tp.user.title_case(title.trim());

/* ---------------------------------------------------------- */
/*                   Set Start And End Dates                  */
/* ---------------------------------------------------------- */
const taskStart = await tp.user.nl_date(tp, 'start');
const taskStartLink = formatAsQuote(createWikiLink(taskStart));
const taskEnd = await tp.user.nl_date(tp, 'end');
const taskEndLink = formatAsQuote(createWikiLink(taskEnd));

/* ---------------------------------------------------------- */
/*         SET TASK CONTEXT BY FILE PATH OR SUGGESTER         */
/* ---------------------------------------------------------- */
const projectPathArr = [
  DIR.PERSONAL,
  DIR.EDUCATION,
  DIR.PROFESSIONAL,
  DIR.WORK,
  DIR.HABITS,
];
let contextDir, contextValue, contextName;
if (
  projectPathArr.includes(`${folderPathSplit[0]}/`) &&
  folderPathSplit.length > 1
) {
  contextDir = `${folderPathSplit[0]}/`;
  contextValue = folderPathSplit[0].slice(3);
  contextName = contextValue.startsWith('habit')
    ? 'Habits and Rituals'
    : contextValue.charAt(0).toUpperCase() + contextValue.substring(1);
} else {
  const contextObj = await tp.user.task_context(tp);
  contextDir = contextObj.directory;
  contextValue = contextObj.value;
  contextName = contextObj.key;
}

/* ---------------------------------------------------------- */
/*               SET PILLAR FILE NAME AND TITLE               */
/* ---------------------------------------------------------- */
const { value: pillarValue, link: pillarYaml } = await tp.user.multi_suggester({
  tp,
  items: await tp.user.file_by_status({ dir: DIR.PILLARS, status: 'active' }),
  type: 'pillar',
});

/* ---------------------------------------------------------- */
/*                          SET GOAL                          */
/* ---------------------------------------------------------- */
const goals = await tp.user.md_file_name(DIR.GOALS);
const goal = await tp.system.suggester(
  goals,
  goals,
  false,
  `Goal for ${FILE_TYPES.PROJECT.name}?`
);

/* ---------------------------------------------------------- */
/*            SET ORGANIZATION FILE NAME AND TITLE            */
/* ---------------------------------------------------------- */
const { value: organizationValue, link: organizationYaml } =
  await tp.user.multi_suggester({
    tp,
    items: await tp.user.md_file_name_alias(DIR.ORGANIZATIONS),
    type: 'organization',
  });

/* ---------------------------------------------------------- */
/*               SET CONTACT FILE NAME AND TITLE              */
/* ---------------------------------------------------------- */
const {
  value: contactValue,
  name: contactName,
  link: contactYaml,
} = await tp.user.multi_suggester({
  tp,
  items: await tp.user.md_file_name_alias(DIR.CONTACTS),
  type: 'contact',
});

/* ---------------------------------------------------------- */
/*                       SET DO/DUE DATE                      */
/* ---------------------------------------------------------- */
const [doDueDateValue, doDueDateName] = parseSemicolonValues(
  await tp.user.include_template(tp, '40_task_do_due_date')
);

/* ---------------------------------------------------------- */
/*                 SET TASK STATUS AND SYMBOL                 */
/* ---------------------------------------------------------- */
const [statusValue, statusName, statusSymbol] = parseSemicolonValues(
  await tp.user.include_template(tp, '40_task_status')
);

/* ---------------------------------------------------------- */
/*    FRONTMATTER TITLE, ALIASES, FILE NAME, AND DIRECTORY    */
/* ---------------------------------------------------------- */
const fullTitleName = title;
const shortTitleName = fullTitleName.toLowerCase();
const shortTitleValue = toSnakeCase(shortTitleName);

const fileAliases = [fullTitleName, shortTitleName, shortTitleValue]
  .map((alias) => createYamlListItem(alias))
  .join(CHAR.NEW_LINE);

const fileName = shortTitleValue;
const fileSection = fileName + CHAR.HASH;
const projectYamlLink = createYamlListItem(
  createWikiLink(fileName, fullTitleName),
  true
);
const projectDir = `${contextDir}${fileName}/`;

/* ---------------------------------------------------------- */
/*                    SECTION OBJECT ARRAYS                   */
/* ---------------------------------------------------------- */
const sectionObjArr = [
  {
    headKey: 'Prepare and Reflect',
    tocLevel: 1,
    tocKey: 'Insight',
    file: '40_project_preview_review',
  },
  {
    headKey: 'Tasks and Events',
    tocLevel: 1,
    tocKey: 'Tasks and Events',
    file: '140_00_related_task_sect_proj',
  },
  {
    headKey: 'Related Tasks and Events',
    tocLevel: 1,
    tocKey: 'Related Tasks',
    file: '100_42_related_task_sect_task_file',
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

for (const section of sectionObjArr) {
  section.content = await tp.user.include_template(tp, section.file);
  section.head = createHeading(2, section.headKey);
  section.tocLink = createTblEscapedLink(
    fileSection + section.headKey,
    section.tocKey
  );
}

/* ---------------------------------------------------------- */
/*                  TABLE OF CONTENTS CALLOUT                 */
/* ---------------------------------------------------------- */
const createTocLevelRow = (level) => {
  const links = sectionObjArr
    .filter((s) => s.tocLevel === level)
    .map((s) => s.tocLink);
  return createCalloutTableRow(links);
};

const toc = [
  createCalloutHeader('toc', dv.contentsLink()),
  formatAsCallout(''), // Blank line for spacing
  createTocLevelRow(1),
  createCalloutTableDivider(3),
  createTocLevelRow(2),
].join(CHAR.NEW_LINE);

/* ---------------------------------------------------------- */
/*                   ASSEMBLE FILE SECTIONS                   */
/* ---------------------------------------------------------- */
const sectionsContent = sectionObjArr
  .map((s) => [s.head, toc, s.content].join(MD.TWO_NEW_LINES))
  .join(MD.TWO_NEW_LINES);

/* ---------------------------------------------------------- */
/*                    FILE DETAILS CALLOUT                    */
/* ---------------------------------------------------------- */
const infoCallout = await tp.user.include_file('40_00_project_info_callout');

/* ---------------------------------------------------------- */
/*                   MOVE FILE TO DIRECTORY                   */
/* ---------------------------------------------------------- */
const destinationDir = projectDir;

if (folderPath !== destinationDir) {
  await tp.file.move(`${destinationDir}${fileName}`);
}

/* ---------------------------------------------------------- */
/*                  PROJECT SETUP PARENT TASK                 */
/* ---------------------------------------------------------- */

/* -- FRONTMATTER TITLE, ALIASES, FILE NAME, AND DIRECTORY -- */
const parFullTitle = `Project Setup for ${title}`;
const parShortTitle = parFullTitle.toLowerCase();
const parFileName = `_proj_setup_${toSnakeCase(title)}`;

const parFileAliases = [
  parFullTitle,
  parShortTitle,
  toSnakeCase(parShortTitle),
  parFileName,
]
  .map((alias) => createYamlListItem(alias))
  .join(CHAR.NEW_LINE);

const parFileDir = `${projectDir}${parFileName}`;
const parFilePath = toMdExt(`${parFileDir}/${parFileName}`);

/* ------------ PARENT TASK FILE DETAILS CALLOUT ------------ */
const par_info = await tp.user.include_file(
  '41_21_par_ed_book_ch_info_callout'
);

/* ---------- PARENT TASK TABLE OF CONTENTS CALLOUT --------- */
const parToc = toc.replace(new RegExp(fileName, 'g'), parFileName);

/* --------- PARENT TASK PREVIEW AND REVIEW SECTION --------- */
sectionObjArr[0].content = await tp.user.include_template(
  tp,
  '41_parent_task_preview_review'
);
/* ---------- PARENT TASK TASKS AND EVENTS SECTION ---------- */
sectionObjArr[1].content = await tp.user.include_template(
  tp,
  '141_00_related_task_sect_parent'
);

/* ---------------- PARENT TASK FILE SECTIONS --------------- */
const parSectionsContent = sectionObjArr
  .map((s) => [s.head, parToc, s.content].join(MD.TWO_NEW_LINES))
  .join(MD.TWO_NEW_LINES);

/* ------------ PARENT TASK FILE DETAILS CALLOUT ------------ */
const parInfoCallout = await tp.user.include_file(
  '41_21_par_ed_book_ch_info_callout'
);

/* --------- PARENT TASK FRONTMATTER YAML PROPERTIES -------- */
const parYamlProperties = [
  `title: ${parFileName}`,
  `uuid: ${await tp.user.uuid()}`,
  `aliases: ${CHAR.NEW_LINE}${parFileAliases}`,
  `task_start: ${taskStartLink}`,
  `task_end:`,
  `due_do: ${doDueDateValue}`,
  `pillar: ${pillarYaml}`,
  `context: ${contextValue}`,
  `goal: ${goal}`,
  `project: ${projectYamlLink}`,
  `organization: ${organizationYaml}`,
  `contact: ${contactYaml}`,
  `library:`,
  `status: ${statusValue}`,
  `type: ${FILE_TYPES.PARENT_TASK.value}`,
  `file_class: ${FILE_TYPES.PARENT_TASK.class}`,
  `date_created: ${dateCreated}`,
  `date_modified: ${dateModified}`,
  `tags:`,
];
const parYaml = [MD.HR_LINE, ...parYamlProperties, MD.HR_LINE].join(
  CHAR.NEW_LINE
);

// Parent Task Content
const parFileContent = [
  parYaml,
  createHeading(1, parFullTitle),
  parInfoCallout,
  parSectionsContent,
].join(MD.TWO_NEW_LINES);

// Create Parent Task File
await this.app.vault.createFolder(parFileDir);
await this.app.vault.create(parFilePath, parFileContent);

// Final Frontmatter for the main project file
const mainYamlProperties = [
  `title: ${fileName}`,
  `uuid: ${await tp.user.uuid()}`,
  `aliases: ${CHAR.NEW_LINE}${fileAliases}`,
  `task_start: ${taskStartLink}`,
  `task_end: ${taskEndLink}`,
  `due_do: ${doDueDate[0]}`,
  `pillar: ${pillarYaml}`,
  `context: ${contextValue}`,
  `goal: ${goal}`,
  `organization: ${organizationYaml}`,
  `contact: ${contactYaml}`,
  `parent_task: ${createYamlListItem(createWikiLink(parFileName), true)}`,
  `library:`,
  `status: ${taskStatus[0]}`,
  `type: ${FILE_TYPES.PROJECT.value}`,
  `file_class: ${FILE_TYPES.PROJECT.class}`,
  `date_created: ${dateCreated}`,
  `date_modified: ${dateModified}`,
  `tags:`,
];

const mainYaml = [
  MD.HR_LINE,
  ...mainYamlProperties,
  MD.HR_LINE,
].join(CHAR.NEW_LINE);

// This is the final output for the project file itself
const mainFileContent = [
  mainYaml,
  createHeading(1, fullTitleName),
  infoCallout,
  sectionsContent,
].join(MD.TWO_NEW_LINES);

// The Templater plugin requires the final output to be returned
return mainFileContent;



/* ------ PARENT TASK DIRECTORY AND FILE PATH CREATION ------ */
await this.app.vault.createFolder(par_dir);
await this.app.vault.create(par_file_path, par_file_content);

tR += hr_line;
%>
title: <%* tR += file_name %>
uuid: <%* tR += await tp.user.uuid() %>
aliases: <%* tR += file_alias %>
task_start: <%* tR += task_start_link %>
task_end: <%* tR += task_end_link %>
due_do: <%* tR += due_do_value %>
pillar: <%* tR += pillar_yaml %>
context: <%* tR += context_value %>
goal: <%* tR += goal %>
organization: <%* tR += organization_yaml %>
contact: <%* tR += contact_yaml %>
library:
status: <%* tR += status_value %>
type: <%* tR += type_value %>
file_class: <%* tR += file_class %>
date_created: <%* tR += date_created %>
date_modified: <%* tR += date_modified %>
tags:
<%* tR += hr_line %>
# <%* tR += full_title_name %>

<%* tR += info %>
<%* tR += sections_content %>
