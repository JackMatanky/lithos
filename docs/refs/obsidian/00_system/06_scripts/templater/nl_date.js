// ------------------------------------------------------------------
// Filename: 00_system/06_scripts/template_scripts/nl_date.js
// Description: A prompt for natural language date input,
//              returning an ISO 8601 date string.
// ------------------------------------------------------------------

/* ---------------------------------------------------------- */
/*             Natural Language Date Option Groups            */
/* ---------------------------------------------------------- */

/* --------------- Short Relative Date Phrases -------------- */
const basic_relative_dates = [
  { key: 'Today', value: 'today' },
  { key: 'Tomorrow', value: 'tomorrow' },
  { key: 'Yesterday', value: 'yesterday' },
  { key: 'In Two Days', value: 'in two days' },
  { key: 'In Three Days', value: 'in three days' },
  { key: 'In Four Days', value: 'in four days' },
  { key: 'In Five Days', value: 'in five days' },
  { key: 'In Six Days', value: 'in six days' },
  { key: 'Two Days Ago', value: 'two days ago' },
  { key: 'Three Days Ago', value: 'three days ago' },
  { key: 'Four Days Ago', value: 'four days ago' },
  { key: 'Five Days Ago', value: 'five days ago' },
  { key: 'Six Days Ago', value: 'six days ago' },
];

/* ----------- Relative Weekdays: This, Next, Last ---------- */
const weekday_relative_dates = [
  'Sunday',
  'Monday',
  'Tuesday',
  'Wednesday',
  'Thursday',
  'Friday',
  'Saturday',
].flatMap((day) => [
  { key: `This ${day}`, value: `this ${day.toLowerCase()}` },
  { key: `Next ${day}`, value: `next ${day.toLowerCase()}` },
  { key: `Last ${day}`, value: `last ${day.toLowerCase()}` },
]);

/* ------ Time Period Expressions: Weeks, Months, Years ----- */
const time_period_relative_dates = ['Week', 'Month', 'Year'].flatMap((date) => [
  { key: `This ${date}`, value: `this ${date.toLowerCase()}` },
  { key: `Next ${date}`, value: `next ${date.toLowerCase()}` },
  { key: `Last ${date}`, value: `last ${date.toLowerCase()}` },
  { key: `In Two ${date}s`, value: `in two ${date.toLowerCase()}s` },
  { key: `In Three ${date}s`, value: `in three ${date.toLowerCase()}s` },
  { key: `In Four ${date}s`, value: `in four ${date.toLowerCase()}s` },
]);

// Generate fixed "March 26" → "mar26" type dates
const fixed_month_day_dates = (() => {
  const months = moment.monthsShort(); // ["Jan", "Feb", ...]
  const result = [];

  for (let m = 0; m < 12; m++) {
    const month_short = months[m].toLowerCase();
    const days_in_month = moment(`${2000}-${m + 1}`, 'YYYY-M').daysInMonth();

    for (let d = 1; d <= days_in_month; d++) {
      const padded_day = String(d).padStart(2, '0');
      result.push({
        key: `${moment().month(m).format('MMMM')} ${padded_day}`,
        value: `${month_short}${padded_day}`,
      });
    }
  }

  return result;
})();

// Null option (to return an empty result)
const null_date_option = [{ key: 'Null', value: 'null' }];

// Combined list of all options
const date_obj_arr = [
  ...basic_relative_dates,
  ...weekday_relative_dates,
  ...time_period_relative_dates,
  ...fixed_month_day_dates,
  ...null_date_option,
];

/* ---------------------------------------------------------- */
/*                     Prompt Label Helper                    */
/* ---------------------------------------------------------- */
/**
 * Determine the label to show in the suggester.
 * @param {string} type - "start", "end", or a custom prompt string
 * @returns {string} - Suggester prompt
 */
function get_prompt_label(type) {
  if (type === 'start') return 'Start Date?';
  if (type === 'end') return 'End Date?';
  if (type) return type;
  return 'When is the date?';
}

/* ---------------------------------------------------------- */
/*                        Main Function                       */
/* ---------------------------------------------------------- */
/**
 * Prompt for a natural language date and return an ISO 8601 string.
 * @param {*} tp - Templater API object
 * @param {string} type - Optional type ("start", "end", or custom label)
 * @returns {string} ISO-formatted date or empty string
 */
async function nl_date(tp, type) {
  const prompt = get_prompt_label(type);

  const nl_date_obj = await tp.system.suggester(
    (item) => item.key,
    date_obj_arr,
    false,
    prompt
    // 419
  );

  const nl_input = nl_date_obj.value;

  if (nl_input !== 'null') {
    return app.plugins.plugins['nldates-obsidian']
      .parseDate(nl_input)
      .moment.format('YYYY-MM-DD');
  }

  return '';
}

module.exports = nl_date;
