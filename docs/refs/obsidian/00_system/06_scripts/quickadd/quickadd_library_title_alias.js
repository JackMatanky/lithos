// RegEx Variables
// Match Patterns
const re_pattern_toc_callout_links =
  /(Context|Reflect|Goals|Events|Directory|Content|Knowledge|Notes|Information|Compass|cards|Code|Snippets)\|(Re|In|Go|Ta|Di|Li|PKM|Tr|No|Id|Fl|Co|Sn)/g;
const re_pattern_mathjax_block = /\\(\[|\])/g;
const re_pattern_mathjax_inline = /\\(\(|\))/g;

// Replace Strings
const re_replace_toc_callout_links = '$1|$2';
const re_replace_mathjax_block = '\$\$';
const re_replace_mathjax_inline = '\$';

// RegEx Object Suggester
const regex_obj_arr = [
  {
    key: 'Table of Contents Callout Links',
    value: 'toc_callout_links',
    re_pattern: re_pattern_toc_callout_links,
    re_replace: re_replace_toc_callout_links,
  },
  {
    key: 'Replace MathJax with LaTeX',
    value: 'mathjax_to_latex',
    re_pattern: [re_pattern_mathjax_block, re_pattern_mathjax_inline],
    re_replace: [re_replace_mathjax_block, re_replace_mathjax_inline],
  },
];

module.exports = async (params) => {
  const {
    app,
    quickAddApi: { suggester, checkboxPrompt },
  } = params;
  // Retrieve the Current File as a TFile
  const current_file = app.workspace.getActiveFile();
  if (!current_file) {
    return void new Notice('No active file.');
  } else {
    console.log('Found active file: ', current_file.basename);
  }
  const file_path = current_file.path;

  const regex_obj = await suggester((item) => item.key, regex_obj_arr);
  console.log(regex_obj);
  const re_value = regex_obj.value;
  let re_pattern = regex_obj.re_pattern;
  let re_replace = regex_obj.re_replace;

  app.vault.process(current_file, (content) => {
    if (re_value == 'mathjax_to_latex') {
      return content
        .replaceAll(re_pattern[0], re_replace[0])
        .replaceAll(re_pattern[1], re_replace[1]);
    } else {
      return content.replaceAll(re_pattern, re_replace);
    }
  });
};
