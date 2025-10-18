// -----------------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/dirContactNames.js
// Description: Prompts for a contact name and returns a structured object with
//              parsed name components. Uses helper functions for readability.
// -----------------------------------------------------------------------------

const SURNAME_PREFIX_ARR = [
  'da',
  'das',
  'de',
  'del',
  'dele',
  'della',
  'den',
  'der',
  'des',
  'di',
  'dos',
  'du',
  'het',
  'la',
  'le',
  'van',
  'von',
];

/* ---------------------------------------------------------- */
/*                      Helper Functions                      */
/* ---------------------------------------------------------- */
/**
 * Parses a single-part name (e.g., "Plato").
 * @param {string[]} parts - An array with one name part.
 * @returns {{lastName: string}}
 */
function parseOnePartName(parts) {
  return { lastName: parts[0] };
}

/**
 * Parses a two-part name (e.g., "John Smith" or "Jane Doe-Smith").
 * @param {string[]} parts - An array with two name parts.
 * @returns {{firstName: string, lastName: string, maidenName?: string}}
 */
function parseTwoPartName(parts) {
  const [firstName, secondPart] = parts;
  // Handle hyphenated last names.
  if (secondPart.includes('-')) {
    const [maidenName, lastName] = secondPart.split('-').map((s) => s.trim());
    return { firstName, maidenName, lastName };
  }
  return { firstName, lastName: secondPart };
}

/**
 * Parses a three-part name (e.g., "Maria de Rossi" or "John
 * Fitzgerald Kennedy").
 * @param {string[]} parts - An array with three name parts.
 * @param {string[]} prefixes - An array of known surname prefixes.
 * @returns {{firstName: string, lastName: string, maidenName?: string, surnamePrefix?: string}}
 */
function parseThreePartName(parts, prefixes) {
  const [firstName, second, third] = parts;
  const isPrefix = prefixes.includes(second.toLowerCase());
  return {
    firstName,
    surnamePrefix: isPrefix ? second : null,
    maidenName: !isPrefix ? second : null,
    lastName: third,
  };
}

/**
 * Parses a four-part name (e.g., "Maria van der Zee").
 * @param {string[]} parts - An array with four or more name parts.
 * @param {string[]} prefixes - An array of known surname prefixes.
 * @returns {object} The parsed name components.
 */
function parseFourPartName(parts, prefixes) {
  const [firstName, p1, p2, p3] = parts;
  const lower = parts.map((p) => p.toLowerCase());

  // Handles double prefixes like "van der".
  if (prefixes.includes(lower[1]) && prefixes.includes(lower[2])) {
    return { firstName, surnamePrefix: `${p1} ${p2}`, lastName: p3 };
  }
  // Handles prefix + maiden name.
  if (prefixes.includes(lower[1])) {
    return { firstName, surnamePrefix: p1, maidenName: p2, lastName: p3 };
  }
  // Handles maiden name + prefix.
  if (prefixes.includes(lower[2])) {
    return { firstName, maidenName: p1, surnamePrefix: p2, lastName: p3 };
  }
  // Default fallback for an unknown 4-part name structure.
  return { firstName, lastName: parts.slice(1).join(' ') };
}

/* ---------------------------------------------------------- */
/*                        Main Function                       */
/* ---------------------------------------------------------- */
/**
 * Returns an object with parsed name components.
 * @returns {object} With keys:
 *   - `fullName`: The original input string.
 *   - `firstName`: The parsed first name.
 *   - `lastName`: The parsed last name.
 *   - `maidenName`: The parsed maiden name (if applicable).
 *   - `surnamePrefix`: The parsed surname prefix (if applicable).
 *   - `lastFirstName`: The parsed "Last, First Middle" string.
 */
async function dirContactNames(tp) {
  const name = (await tp.system.prompt('Full Name?'))?.trim();
  if (!name) {
    return {
      fullName: null,
      firstName: null,
      lastName: null,
      maidenName: null,
      surnamePrefix: null,
      lastFirstName: null,
    };
  }

  const parts = name.split(' ').filter(Boolean);
  let parsed = {};

  // Dispatch to the correct helper function based on name length.
  switch (parts.length) {
    case 1:
      parsed = parseOnePartName(parts);
      break;
    case 2:
      parsed = parseTwoPartName(parts);
      break;
    case 3:
      parsed = parseThreePartName(parts, SURNAME_PREFIX_ARR);
      break;
    default: // Handles 4 or more parts.
      parsed = parseFourPartName(parts, SURNAME_PREFIX_ARR);
  }

  // Deconstruct the parsed result, providing null as a default
  // for any missing parts.
  const {
    firstName = null,
    lastName = null,
    maidenName = null,
    surnamePrefix = null,
  } = parsed;

  // Robustly construct the "Last, First Middle" string.
  const middleParts = [surnamePrefix, maidenName].filter(Boolean);
  const lastFirstName = !firstName
    ? lastName // For single names.
    : middleParts.length > 0
    ? `${lastName}, ${firstName} ${middleParts.join(' ')}`
    : `${lastName}, ${firstName}`;

  return {
    fullName: name,
    firstName,
    lastName,
    maidenName,
    surnamePrefix,
    lastFirstName,
  };
}

module.exports = dirContactNames;
