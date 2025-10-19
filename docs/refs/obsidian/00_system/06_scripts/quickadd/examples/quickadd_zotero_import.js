const API_KEY_OPTION = 'Zotero API Key';
const USER_ID_OPTION = 'Zotero User ID';
const ANNOTATION_FORMAT_OPTION = 'Annotation Format';
const AUTHOR_FORMAT_OPTION = 'Author Format';
const API_URL = 'https://api.zotero.org';

module.exports = {
  entry: start,
  settings: {
    author: 'Christian B. B. Houmann',
    name: 'Zotero Importer',
    options: {
      [API_KEY_OPTION]: {
        type: 'text',
        defaultValue: '',
        placeholder: 'Zotero API Key',
        secret: true,
      },
      [USER_ID_OPTION]: {
        type: 'text',
        defaultValue: '',
        placeholder: 'Zotero User ID',
      },
      [ANNOTATION_FORMAT_OPTION]: {
        type: 'format',
        defaultValue: `> {{VALUE:quote}}\n\n{{VALUE:note}}`,
        placeholder: 'Annotation Format',
      },
      [AUTHOR_FORMAT_OPTION]: {
        type: 'format',
        defaultValue: '[[{{VALUE:firstName}} {{VALUE:lastName}}]]',
        placeholder: 'Author Format',
      },
    },
  },
};

let QuickAdd;
let Settings;

async function start(params, settings) {
  QuickAdd = params;
  Settings = settings;

  if (
    !safeInvariant(
      Settings[API_KEY_OPTION],
      'Please set your Zotero API Key and User ID in the settings for this script.'
    ) ||
    !safeInvariant(
      Settings[USER_ID_OPTION],
      'Please set your Zotero User ID in the settings for this script.'
    )
  ) {
    return;
  }

  const topLevelItems = (await fetchTopLevelItems()).filter(
    (item) => item.data.title
  );
  const targetItem = await QuickAdd.quickAddApi.suggester(
    topLevelItems.map((item) => item.data.title),
    topLevelItems
  );

  const pdfChildren = await fetchItemPDFChildren(targetItem);
  const annotations = pdfChildren.flatMap(getAnnotations);
  const markdownAnnotations = addMarkdownAttributesToAnnotations(annotations);

  const variables = {
    fileName: replaceIllegalFileNameCharactersInString(targetItem.data.title),
    ...targetItem,
    ...targetItem.data,
    abstract: targetItem.data.abstractNote ?? '',
    shortAuthor: targetItem.meta.creatorSummary,
    doi: targetItem.data.DOI ?? '',
    year: targetItem.data.date?.split('-')[0] ?? '',
    issn: targetItem.data.ISSN ?? '',
  };

  const formattedAnnotations = [];
  for (const annotation of markdownAnnotations) {
    const formattedAnnotation = await QuickAdd.quickAddApi.format(
      Settings[ANNOTATION_FORMAT_OPTION],
      {
        ...annotation,
        ...variables,
        note: annotation.note.length === 0 ? '-' : annotation.note,
        quote: annotation.quote.length === 0 ? '-' : annotation.quote,
      }
    );

    formattedAnnotations.push(formattedAnnotation);
  }

  const authors = [...(targetItem.data.creators ?? [])];
  const formattedAuthors = [];
  for (const author of authors) {
    const formattedAuthor = await QuickAdd.quickAddApi.format(
      Settings[AUTHOR_FORMAT_OPTION],
      {
        ...author,
        firstName: author.firstName ?? '',
        lastName: author.lastName ?? '',
      }
    );

    formattedAuthors.push(formattedAuthor);
  }

  Object.assign(QuickAdd.variables, variables, {
    formattedAnnotations: formattedAnnotations.join('\n\n'),
    authors: formattedAuthors.join(', '),
  });
}

function safeInvariant(condition, message) {
  if (!condition) {
    new Notice(message);
    return false;
  }

  return true;
}

function addMarkdownAttributesToAnnotations(annotations) {
  return annotations.map((annotation) => {
    const note = annotation.data.annotationComment ?? '';
    annotation['note'] = QuickAdd.obsidian.htmlToMarkdown(note);

    const text = annotation.data.annotationText ?? '';
    annotation['quote'] = QuickAdd.obsidian.htmlToMarkdown(text);

    return annotation;
  });
}

function getAnnotations(pdfChildren) {
  const annotations = pdfChildren.filter(
    (child) => child.data.itemType === 'annotation'
  );

  return annotations;
}

async function fetchItemPDFChildren(item) {
  const children = await fetchItemChildren(item);
  if (!children) return [];
  const pdfChildren = children.filter(
    (child) => child?.data?.contentType === 'application/pdf'
  );

  const resArr = (
    await Promise.all(
      pdfChildren.map((child) => {
        const url = `${child.links.self.href}/children`;
        return makeRequest(url);
      })
    )
  ).map((res) => res.json);

  return resArr;
}

async function fetchItemChildren(item) {
  const url = `${API_URL}/users/${Settings[USER_ID_OPTION]}/items/${item.key}/children?format=json&limit=100`;
  const response = await makeRequest(url);

  return response.json;
}

async function fetchTopLevelItems() {
  const route = new URL(
    `${API_URL}/users/${Settings[USER_ID_OPTION]}/items/top`
  );
  route.searchParams.set('limit', 10000);
  const response = await makeRequest(route.toString());

  return response.json;
}

async function makeRequest(url, options = {}) {
  const headers = {
    'Zotero-API-Version': 3,
    'Zotero-API-Key': Settings[API_KEY_OPTION],
    ...options.headers,
  };

  return requestUrl({
    ...options,
    url,
    headers,
  });
}

function replaceIllegalFileNameCharactersInString(string) {
  return string.replace(/[\\,#%&\{\}\/*<>$\'\":@]*/g, '');
}
