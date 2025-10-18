// --------------------------------------------------------
// RegEx Variables
// --------------------------------------------------------
// Match Patterns
const re_pattern_tex_block_start = /\n\$\$(\n\S)/g;
const re_pattern_tex_block_end = /(\S\n)\$\$\n?/g;
const re_pattern_tex_inline = /(?<!\\)\$(.+?)\$(?!\w)/g;
//const re_pattern_tex_inline = /(\s)\$(\S.*?\S)\$(\s|\.)/g;
const re_pattern_underscore_caret_brace = /(\^|_|\\{|\\})/g;
const re_pattern_equation_new_line = /(\\\\)/g;
const re_pattern_ordered_list_escape = /^(\d+)\\\.(?=\s)/gm;

// Replace Strings
const re_replace_tex_block_start = '\n\\\\[$1';
const re_replace_tex_block_end = '$1\\\\]\n';
const re_replace_tex_inline = '\\\\($1\\\\)';
//const re_replace_tex_inline = '$1\\\\($2\\\\)$3';
const re_replace_underscore_caret_brace = '\\$1';
const re_replace_equation_new_line = '\\\\$1';
const re_replace_ordered_list_escape = '$1.';

// RegEx Object Array
const regex_obj_arr = [
  {
    key: 'LaTex Math Block Start',
    re_pattern: re_pattern_tex_block_start,
    re_replace: re_replace_tex_block_start,
  },
  {
    key: 'LaTex Math Block End',
    re_pattern: re_pattern_tex_block_end,
    re_replace: re_replace_tex_block_end,
  },
  {
    key: 'LaTex Inline Math',
    re_pattern: re_pattern_tex_inline,
    re_replace: re_replace_tex_inline,
  },
  {
    key: 'LaTex Characters to Escape',
    re_pattern: re_pattern_underscore_caret_brace,
    re_replace: re_replace_underscore_caret_brace,
  },
  {
    key: 'LaTex Equation New Line',
    re_pattern: re_pattern_equation_new_line,
    re_replace: re_replace_equation_new_line,
  },
  {
    key: 'MD Escaped Ordered List',
    re_pattern: re_pattern_ordered_list_escape,
    re_replace: re_replace_ordered_list_escape,
  },
];

// --------------------------------------------------------
// Unicode Backslash Definitions
// --------------------------------------------------------
const BACKSLASH = String.fromCodePoint(0x5c);
const DOUBLE_BACKSLASH = BACKSLASH.repeat(2);
const FOUR_BACKSLASH = BACKSLASH.repeat(4);

// Cleanup Function: Removes Quadruple Backslashes
const clean_backslashes = (text) => {
  const brackets_parentheses = ['[', ']', '(', ')'];
  brackets_parentheses.forEach((char) => {
    text = text.replaceAll(FOUR_BACKSLASH + char, DOUBLE_BACKSLASH + char);
  });
  return text;
};

const apply_transformations = (text) =>
  clean_backslashes(
    regex_obj_arr.reduce(
      (updated_text, { re_pattern, re_replace }) =>
        updated_text.replace(re_pattern, re_replace),
      text
    )
  );

// -----------------------------------------------
// Anki Markdown Converter
// -----------------------------------------------
module.exports = async (params) => {
  const {
    app,
    quickAddApi: { utility },
  } = params;

  // Retrieve active editor
  const editor = app.workspace.activeEditor?.editor;
  if (!editor) {
    return void new Notice('No active editor found.');
  }

  // Retrieve selected text
  const selected_text = editor.getSelection();
  if (!selected_text) {
    return void new Notice('No text selected.');
  }

  // Apply conversions
  const converted_text = apply_transformations(selected_text);

  await utility.setClipboard(converted_text);

  // Concatenate original + converted text with two blank lines
  const final_text = `${selected_text}\n\n${converted_text}`;

  // Replace selection with concatenated text
  editor.replaceSelection(final_text);
};
