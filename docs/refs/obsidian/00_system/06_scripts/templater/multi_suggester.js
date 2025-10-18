// -----------------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/multi_suggester.js
// Description:
//   Templater utility to select multiple items via Obsidian's suggester,
//   with support for:
//     - Object or string arrays
//     - Optional preset (explicit or by context/type)
//     - "_user_input" dispatch to custom handler functions
//     - YAML-formatted output for use in metadata blocks
//
// Usage:
//   const result = await tp.user.multi_suggester({
//     tp,
//     items: await tp.user.md_file_name_alias("20_pillars/"),
//     type: "pillar",
//     context: "career", // Optional
//     // prompt: "Select Pillar(s)", // Optional
//   });
// -----------------------------------------------------------------------------

/* ---------------------------------------------------------- */
/*                    Formatting Utilities                    */
/* ---------------------------------------------------------- */

/**
 * Create a Markdown wikilink with alias.
 * Example: [[slug|alias]]
 */
const link_alias = (slug, alias) => `[[${slug}|${alias}]]`;

/**
 * Convert a value to a properly quoted YAML list item.
 */
const yaml_li = (value) => `\n  - "${value}"`;


/* ---------------------------------------------------------- */
/*                          Constants                         */
/* ---------------------------------------------------------- */

/**
 * Options for confirming additional selections.
 */
const YES_NO_CHOICES = [
  { key: '✔️ YES ✔️', value: 'yes' },
  { key: '❌ NO ❌', value: 'no' },
];

/**
 * Values that represent user cancellation or null-like input.
 */
const NULL_LIKE_VALUES = ['', 'null', '[[null|Null]]', null];

/**
 * Optional presets based on a known type-context pair.
 * Used to prefill the first selection automatically.
 */
const context_preset_map = {
  pillar: {
    education: {
      key: 'Knowledge Expansion',
      value: 'knowledge_expansion',
    },
    mental: {
      key: 'Mental Health',
      value: 'mental_health',
    },
    professional: {
      key: 'Career Development',
      value: 'career_development',
    },
  },
  // Extend for other types as needed
};

/**
 * Custom input dispatch handlers for _user_input selection.
 * These allow users to create new values dynamically (e.g. from prompts).
 */
const dispatch_handler_map = {
  contact: {
    user_input_handler: async (tp) => {
      const obj = await tp.user.dirContactNames(tp);
      const slug = obj.lastFirstName
        .replaceAll(/,/g, '')
        .replaceAll(/[^\w]/g, '_')
        .toLowerCase();
      return { key: obj.fullName, value: slug };
    },
  },
  organization: {
    user_input_handler: async (tp) => {
      return await tp.user.dirOrganizationNames(tp); // returns { key, value }
    },
  },
  // Extend for other types as needed
};

/* ---------------------------------------------------------- */
/*                   Main Suggester Function                  */
/* ---------------------------------------------------------- */
/**
 * Show a multi-select suggester with optional context and dynamic input support.
 *
 * @param {object} options
 * @param {object} options.tp - Templater object
 * @param {Array<string|{ key: string, value: string }>} options.items
 * @param {string} [options.prompt] - Optional custom prompt to display
 * @param {string|object} [options.preset] - Optional preset to prefill
 * @param {string} [options.type] - Optional type identifier (e.g. "pillar")
 * @param {string} [options.context] - Optional context identifier (e.g. "career")
 * @returns {Promise<{ value: string, name: string, link: string, raw: object[] }>}
 */
async function multi_suggester({
  tp,
  items: item_array,
  prompt = null,
  preset = null,
  type = null,
  context = null,
}) {
  /* -------------------- Resolve Preset -------------------- */
  const resolved_context_preset =
    type && context && context_preset_map?.[type]?.[context]
      ? context_preset_map[type][context]
      : null;

  /* -------------------- Resolve Prompt -------------------- */
  const initial_preset = preset || resolved_context_preset;

  const prompt_prefix = type
    ? `Select ${type
        .replace(/_/g, ' ')
        .replace(/\b\w/g, (c) => c.toUpperCase())}`
    : 'Make a selection';

  const preset_label =
    initial_preset?.key && type
      ? ` (${initial_preset.key} already included)`
      : '';

  const resolved_prompt = prompt ?? `${prompt_prefix}${preset_label}?`;

  /* ------------------ Resolve User Input ------------------ */
  const user_input_handler =
    type && dispatch_handler_map?.[type]?.user_input_handler
      ? dispatch_handler_map[type].user_input_handler
      : null;

  /* -------------- Multi-Select Suggester Loop ------------- */
  const selected = [];
  const selected_filter = [];

  if (initial_preset) {
    selected.push(initial_preset);
    selected_filter.push(initial_preset.value);
  }

  for (let i = 0; i < 10; i++) {
    const remaining = item_array.filter((item) => {
      const val = typeof item === 'string' ? item : item?.value ?? item?.key;
      return !selected_filter.includes(val);
    });

    if (remaining.length === 0) break;

    const display_list =
      typeof remaining[0] === 'string'
        ? remaining
        : remaining.map((item) => item.key);

    const selected_item = await tp.system.suggester(
      display_list,
      remaining,
      false,
      resolved_prompt
    );

    // ❌ Handle cancel or null-like entries
    if (
      typeof selected_item === 'string' &&
      NULL_LIKE_VALUES.includes(selected_item)
    ) {
      if (selected.length === 0) selected.push({ key: 'Null', value: 'null' });
      break;
    }

    if (
      typeof selected_item === 'object' &&
      NULL_LIKE_VALUES.includes(selected_item?.value)
    ) {
      break;
    }

    // 🔄 Handle _user_input dispatch case
    if (
      typeof selected_item === 'object' &&
      selected_item.value === '_user_input' &&
      user_input_handler
    ) {
      const custom = await user_input_handler(tp);
      selected.push(custom);
      selected_filter.push(custom.value);
    } else {
      selected.push(selected_item);
      selected_filter.push(
        typeof selected_item === 'string' ? selected_item : selected_item.value
      );
    }

    // ❓ Ask user if they want to continue
    const continue_choice = await tp.system.suggester(
      (item) => item.key,
      YES_NO_CHOICES,
      false,
      `Add another ${type}?`
    );

    if (continue_choice.value === 'no') break;
  }

  /* -------------------------------------------------------- */
  /*                     Output Formatting                    */
  /* -------------------------------------------------------- */
  const value_arr = [];
  const name_arr = [];
  const link_arr = [];

  selected.forEach(({ key, value }) => {
    const wikilink = link_alias(value, key);
    value_arr.push(value);
    name_arr.push(key);
    link_arr.push(yaml_li(wikilink));
  });

  return {
    value: value_arr.join(', '),
    name: name_arr.join(', '),
    link: link_arr.join(''),
    raw: selected,
  };
}

module.exports = multi_suggester;
