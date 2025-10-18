// ------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/nl_time.js
// Description: A prompt for natural language time input, including
//              relative and absolute options.
// ------------------------------------------------------------------

/* ---------------------------------------------------------- */
/*                Natural Language Time Options               */
/* ---------------------------------------------------------- */
const relative_time_options = [
  { key: 'Now', value: 'now' },
  { key: 'In 5 Minutes', value: 'in 5min' },
  { key: 'In 10 Minutes', value: 'in 10min' },
  { key: 'In 15 Minutes', value: 'in 15min' },
  { key: 'In 20 Minutes', value: 'in 20min' },
  { key: 'In 30 Minutes', value: 'in 30min' },
  { key: 'In 45 Minutes', value: 'in 45min' },
  { key: 'In 1 Hour', value: 'in 60min' },
  { key: 'In 1 Hour and 5 Minutes', value: 'in 65min' },
  { key: 'In 1 Hour and 10 Minutes', value: 'in 70min' },
  { key: 'In 1 Hour and 15 Minutes', value: 'in 75min' },
  { key: 'In 1 Hour and 20 Minutes', value: 'in 80min' },
  { key: 'In 1 Hour and 30 Minutes', value: 'in 90min' },
  { key: 'In 1 Hour and 45 Minutes', value: 'in 105min' },
  { key: 'In 2 Hours', value: 'in 120min' },
];

// Generate absolute clock time options in 5-minute increments
const absolute_time_options = (() => {
  const options = [];
  for (let h = 0; h < 24; h++) {
    for (let m = 0; m < 60; m += 5) {
      const hour = String(h).padStart(2, '0');
      const minute = String(m).padStart(2, '0');
      options.push({
        key: `${hour}${minute}`,
        value: `at ${hour}:${minute}`,
      });
    }
  }
  return options;
})();

const nl_time_obj_arr = [...relative_time_options, ...absolute_time_options];

/**
 * Prompt the user to select a natural language time.
 * @param {*} tp - Templater API object
 * @param {string} prompt - Optional prompt string (e.g., "Start Time?")
 * @returns {string} - Time in HH:mm format or "null" if no selection
 */
async function nl_time(tp, prompt) {
  const prompt_label = prompt || 'Start Time?';

  const nl_time_obj = await tp.system.suggester(
    (item) => item.key,
    nl_time_obj_arr,
    false,
    prompt_label
  );

  if (!nl_time_obj || nl_time_obj.value === 'null') {
    return 'null';
  }

  return app.plugins.plugins['nldates-obsidian']
    .parseDate(nl_time_obj.value)
    .moment.format('HH:mm');
}

module.exports = nl_time;
