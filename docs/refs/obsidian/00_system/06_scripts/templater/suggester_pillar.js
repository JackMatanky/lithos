// -----------------------------------------------------------------------------
// Filename: suggester_pillar.js
// Description:
//   Wrapper for multi_suggester() to select pillar files with context-aware
//   preset. Returns array of objects with value, alias, link, and yaml fields.
// -----------------------------------------------------------------------------

function link_alias(file, alias) {
  return `[[${file}|${alias}]]`;
}
function yaml_li(value) {
  return `\n  - "${value}"`;
}

/**
 * Select pillar files using multi_suggester with a context-based preset.
 *
 * @param {object} tp - Templater object
 * @param {string} [context="default"] - One of: career, know, mental
 * @returns {Promise<Array<{ value, alias, link, yaml }>>}
 */
async function suggester_pillar(tp, context = "default") {
  const pillars_dir = "20_pillars/";
  const pillar_items = await tp.user.md_file_name_alias(pillars_dir);

  const result = await tp.user.multi_suggester({
    tp,
    items: pillar_items,
    prompt: "Select Pillar(s)",
    context, // <-- passed to resolve preset internally
    type: "pillar", // optional: if you want pillar-specific input handlers
  });

  // Map to structured output
  return result.raw.map(({ key, value }) => {
    const link = link_alias(value, key);
    return {
      value,
      alias: key,
      link,
      yaml: yaml_li(link),
    };
  });
}

module.exports = suggester_pillar;
