// -----------------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/yaml_multiline.js
// Description: Templater script to convert free-text input (including lists)
//              into a YAML multiline block string (pipe style `|`) with
//              properly indented lines and paragraph spacing.
// -----------------------------------------------------------------------------

// Original Templater Code
// const about_value = about
//   .replaceAll(/^(\s*)([^\s])/g, '$2')
//   .replaceAll(/(\s*)\n/g, '\n')
//   .replaceAll(/([^\s])(\s*)$/g, '$1')
//   .replaceAll(/\n{1,6}/g, '<new_line>')
//   .replaceAll(/(<new_line>)(\w|\*\*\w)/g, '\n \n $2')
//   .replaceAll(/(<new_line>)(-\s|\d\.\s)/g, '\n $2');

// -----------------------------------------------------------------------------
// Regex Definitions
// -----------------------------------------------------------------------------

// Match leading spaces followed by a non-space character (remove indentation)
const REGEX_LEADING_SPACES = /^(\s*)([^\s])/gm;

// Match trailing spaces on each line (clean trailing whitespace)
const REGEX_TRAILING_SPACES = /([^\s])(\s*)$/gm;

// Match multiple newlines (replace with placeholder for structured control)
const REGEX_MULTI_NEWLINES = /\n{1,6}/g;

// Match bolded section headers (e.g., **Context**) or regular words after newline
const REGEX_PLACEHOLDER_BEFORE_PARAGRAPH = /<new_line>(\*\*[^\n*]+?\*\*|\w)/g;

// Replace placeholder with one newline before list items (- item or 1. item)
const REGEX_PLACEHOLDER_BEFORE_LIST = /<new_line>(-\s|\d+\.\s)/g;

// -----------------------------------------------------------------------------
// Function Definition
// -----------------------------------------------------------------------------

/**
 * Format text input into a properly indented YAML multiline string.
 *
 * @param {string} text - The free-form input (paragraphs, lists, bolded lines).
 * @returns {Promise<string>} YAML-compatible block string with indentation.
 */
async function yaml_multiline(text) {
  // Step-by-step formatting
  const formatted = text
    .replace(REGEX_LEADING_SPACES, "$2")
    .replace(REGEX_TRAILING_SPACES, "$1")
    .replace(REGEX_MULTI_NEWLINES, "<new_line>")
    .replace(REGEX_PLACEHOLDER_BEFORE_PARAGRAPH, "\n\n$1")
    .replace(REGEX_PLACEHOLDER_BEFORE_LIST, "\n$1");

  // Prepare YAML block line prefix
  const yaml_prefix = "|\n";

  // Indent all lines with two spaces
  const indented_body = formatted.replace(/\n/g, "\n  ");

  // Combine prefix and body
  const yaml_block_string = `${yaml_prefix}  ${indented_body}`;

  return indented_body;
}

module.exports = yaml_multiline;
