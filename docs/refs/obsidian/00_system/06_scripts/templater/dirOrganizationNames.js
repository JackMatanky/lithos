// -----------------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/dirOrganizationNames.js
// Description: Prompts the user for an organization name and returns a
//              structured object with the original display name and a
//              slugified version for filename use.
// -----------------------------------------------------------------------------

// Grouped Regex constants for slugifying strings.
const REGEX = {
  // Removes commas and apostrophes.
  REMOVE_PUNCTUATION: /[,']/g,
  // Replaces spaces and periods with underscores.
  SPACES_AND_DOTS: /[\s\.]/g,
  // Replaces slashes with hyphens.
  FORWARD_SLASH: /\//g,
  // Replaces ampersands with "and".
  AMPERSAND: /&/g,
};

/**
 * Prompt for an organization name and return both the original name
 * and a sanitized, filename-safe "slug" version.
 *
 * @param {object} tp - The Templater plugin object.
 * @returns {Promise<{key: string, value: string}|{key: null, value: null}>}
 * An object with the display name (key) and slug (value), or a null object.
 */
async function dirOrganizationNames(tp) {
  // 1. Prompt user for input and sanitize by trimming whitespace.
  const orgName = (await tp.system.prompt('Organization Name?'))?.trim();

  // 2. Gracefully exit if the user cancels or provides no input.
  if (!orgName) {
    return { key: null, value: null };
  }

  // 3. Slugify the name using the REGEX object.
  const orgSlug = orgName
    .replaceAll(REGEX.REMOVE_PUNCTUATION, '')
    .replaceAll(REGEX.SPACES_AND_DOTS, '_')
    .replaceAll(REGEX.FORWARD_SLASH, '-')
    .replaceAll(REGEX.AMPERSAND, 'and')
    .toLowerCase();

  // 4. Return the structured object.
  return {
    key: orgName,
    value: orgSlug,
  };
}

module.exports = dirOrganizationNames;
