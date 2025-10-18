// -----------------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/get_frontmatter_field.js
// Description:
//   Retrieves a specified frontmatter field from a markdown file.
//   Uses `resolve_file_path` to locate the file and Obsidian’s metadata cache
//   to extract frontmatter.
//
//   If the file or field is not found, returns `null` and logs a warning.
// -----------------------------------------------------------------------------

/**
 * Resolve the full path of a markdown file by its base name.
 *
 * @param {string} file_name - The base name of the file (without extension).
 * @returns {Promise<string|null>} Full vault path or null if not found.
 */
const resolve_file_path = async (file_name) => {
  const target_ext = file_name + ".md";
  const match = app.vault
    .getMarkdownFiles()
    .find((file) => file.path.endsWith(`/${target_ext}`));
  return match?.path || null;
};

/**
 * Retrieves a specific frontmatter field from a markdown file.
 *
 * @param {string} file_name - The base name of the markdown file.
 * @param {string} field - The frontmatter key to retrieve.
 * @returns {Promise<any|null>} The field value or null if not found.
 */
async function get_frontmatter_field(file_name, field) {
  const path = await resolve_file_path(file_name);
  if (!path) {
    console.warn(`[get_frontmatter_field] File not found: ${file_name}.md`);
    return null;
  }

  const abstract_file = await app.vault.getAbstractFileByPath(path);
  const cache = await app.metadataCache.getFileCache(abstract_file);

  const value = cache?.frontmatter?.[field] ?? null;

  if (value === null) {
    console.warn(
      `[get_frontmatter_field] Field "${field}" not found in: ${file_name}.md`,
    );
  }

  return value;
}

module.exports = get_frontmatter_field;
