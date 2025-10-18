// -----------------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/getFilePath.js
// Description: Gets the full vault path of a markdown file by its base name.
//              Searches for a file ending in `/<fileName>.md` to ensure an
//              exact match. Returns the full path string or null if not found.
// -----------------------------------------------------------------------------

/**
 * Gets the full vault path of a markdown file by its base name.
 *
 * @param {string} fileName - The base name of the file (no path or extension).
 * @returns {string|null} The full path to the file, or null if not found.
 */
async function getFilePath(fileName) {
  // 1. Guard against null, undefined, or empty input.
  if (!fileName || fileName === 'null') {
    return null;
  }

  const targetFile = `${fileName}.md`;

  // 2. Search for a file that has an exact match at the end of its path.
  const match = app.vault
    .getMarkdownFiles()
    .find((file) => file.path.endsWith(`/${targetFile}`));

  // 3. If no match is found, log a warning for debugging and return null.
  if (!match) {
    console.warn(`[getFilePath] File not found: ${targetFile}`);
    return null;
  }

  // 4. Return the full path of the matched file.
  return match.path;
}

module.exports = getFilePath;
