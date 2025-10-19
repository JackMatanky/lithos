// -----------------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/durationMin.js
// Description: Prompts the user to select a duration from a dynamically
//              generated list of time increments and returns the total minutes.
// -----------------------------------------------------------------------------

/**
 * Generates a user-friendly string for a given duration in minutes.
 * Example: 75 -> "1 Hour and 15 Minutes"
 * @param {number} totalMinutes - The duration in total minutes.
 * @returns {string} A formatted string for display.
 */
function formatMinutesToKey(totalMinutes) {
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  const hourString = `${hours} Hour${hours > 1 ? 's' : ''}`;
  const minuteString = `${minutes} Minutes`;

  if (hours > 0 && minutes > 0) {
    return `${hourString} and ${minuteString}`;
  } else if (hours > 0) {
    return hourString;
  } else {
    return minuteString;
  }
}

/**
 * Generates an array of duration options for the suggester.
 * @param {number} maxHours - The maximum number of hours to generate options for.
 * @param {number} increment - The increment in minutes between options.
 * @returns {Array<{key: string, value: string}>} The array of options.
 */
function generateDurationOptions(maxHours, increment) {
  const options = [
    { key: "Null", value: "null" },
    { key: "User Input", value: "_user_input" },
  ];

  for (let m = increment; m <= maxHours * 60; m += increment) {
    options.push({
      key: formatMinutesToKey(m),
      value: String(m),
    });
  }
  return options;
}

/* ---------------------------------------------------------- */
/*                        Main Function                       */
/* ---------------------------------------------------------- */
/**
 * Prompts the user to select a duration from a dynamically generated list
 * of time increments and returns the total minutes.
 * @param {TP} tp - The Templating Plugin API.
 * @returns {string} The selected duration in minutes. If the user selects
 *                   "User Input", the user is prompted to enter the duration
 *                   in minutes. If the user selects "Null", "null" is returned.
 */
async function durationMin(tp) {
  // Generate the duration options dynamically
  const durationMinObjArr = generateDurationOptions(10, 5);

  const durationMinObj = await tp.system.suggester(
    (item) => item.key,
    durationMinObjArr,
    false,
    "Duration?"
  );

  let selectedDuration = durationMinObj?.value ?? "null";

  if (selectedDuration === "_user_input") {
    selectedDuration = await tp.system.prompt("Duration in Minutes?", null, false, false) ?? "null";
  }

  return selectedDuration;
}

module.exports = durationMin;
