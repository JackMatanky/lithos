// ----------------------------------------------
//  Suggester for Date
// ----------------------------------------------
// >>> Constants <<<
// Earliest year included in selection
const START_YEAR = 1700;

// Leap year used to ensure Feb 29 is included
const BASE_LEAP_YEAR = 2000;

// Month names from moment.js - ["January", ..., "December"]
const MONTHS = moment.months();

// Special options for year and day selection
const NULL_YEAR_OPTIONS = ['null', '_user_input'];
const NULL_MONTH_DAY_OPTION = { key: 'Null', value: 'null' };

// >>> Helpers <<<
// Generate descending list of years plus null/user-input options
const generate_year_options = () => {
  const current_year = moment().year();
  const years = [];
  for (let y = current_year; y >= START_YEAR; y--) {
    years.push(y.toString());
  }
  return [...NULL_YEAR_OPTIONS, ...years];
};

// Generate all valid month-day combinations using BASE_LEAP_YEAR
const generate_month_day_options = () => {
  const options = [NULL_MONTH_DAY_OPTION];

  for (let month_index = 0; month_index < 12; month_index++) {
    const days_in_month = moment(
      `${BASE_LEAP_YEAR}-${month_index + 1}`,
      'YYYY-M'
    ).daysInMonth();
    for (let day = 1; day <= days_in_month; day++) {
      const mm = String(month_index + 1).padStart(2, '0');
      const dd = String(day).padStart(2, '0');
      options.push({
        key: `${MONTHS[month_index]} ${dd}`, // User-facing format
        value: `-${mm}-${dd}`, // Output as suffix for YYYY-MM-DD
      });
    }
  }

  return options;
};

// >>> Main Function <<<
// Suggester for date input, with year and month-day selection
async function suggester_date(tp) {
  const year_options = generate_year_options();
  const month_day_options = generate_month_day_options();

  // Prompt user to select or enter a year
  let year = await tp.system.suggester(
    year_options,
    year_options,
    false,
    'Year?'
  );
  if (year === '_user_input') {
    year = await tp.system.prompt('Year? (YYYY format)', null, false, false);
  }

  // Prompt user to select a month-day value
  const month_day_obj = await tp.system.suggester(
    (item) => item.key,
    month_day_options,
    false,
    'Month and Day?'
  );
  const month_day = month_day_obj.value;

  // Combine year and day into a full ISO-style date, handling special cases
  let date;
  if (year === 'null' && month_day === 'null') {
    date = ''; // No input
  } else if (year === 'null') {
    date = `0001${month_day}`; // Unknown year
  } else if (month_day === 'null') {
    date = `${year}-01-01`; // Default to Jan 1
  } else {
    date = `${year}${month_day}`; // Full date
  }

  return date;
}

module.exports = suggester_date;
