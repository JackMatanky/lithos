# Template Function Registry

This file is a planning input for the dedicated `template-extension-registry` phase. It is explicitly out of scope for `template-foundation`.

Foundation uses MiniJinja built-ins only and does not implement `TemplateExtension`, `ExtensionRegistry`, or Lithos custom modules.

All operations are grouped into modules.

Functions generate data or perform side effects and are called directly: `{{ date.now() }}`

Filters transform data and are called using the pipe operator, implicitly taking the left-hand value as their first argument: `{{ my_string | str.slugify }}`

## 📁 File Module (file.*)

I/O operations and file metadata.

### Functions
- **file.read(path)**: Reads the contents of a file into a string.
- **file.read_lines(path)**: Reads a file and returns an array of strings (one per line).
- **file.write(path)**: Sets the intended output path for the rendered template (stateful side-effect, executed post-render).
- **file.size(path)**: Returns the size of the file in bytes.

### Filters
None. All file operations are executed as explicit functions.

## 🗺️ Path Module (path.*)

Path manipulation and filesystem inspection.

### Functions
- **path.join(base, append)**: Concatenates paths safely regardless of OS.

### Filters
- **path.absolute**: Converts a relative path string into an absolute path.
- **path.exists**: Returns true if the path string points to an existing file/directory.
- **path.is_file**: Returns true if the path points to a file.
- **path.is_dir**: Returns true if the path points to a directory.
- **path.name**: Returns the full filename with extension (e.g., "main.rs").
- **path.basename**: Returns the filename without the extension (e.g., "main").
- **path.stem**: Alias for `path.basename`.
- **path.extension**: Returns just the extension (e.g., "rs").
- **path.parent**: Returns the parent directory path.

## 📅 Date Module (date.*)

Time generation, formatting, and arithmetic.

### Functions
- **date.now()**: Returns the current UTC time as an RFC3339 string.
- **date.today()**: Returns the current date (no time component).
- **date.tomorrow()**: Returns tomorrow's date (no time component).
- **date.yesterday()**: Returns yesterday's date (no time component).
- **date.from_timestamp(unix_ts)**: Converts a Unix timestamp integer into an RFC3339 string.

### Filters
- **date.format(format_string)**: Parses an RFC3339 date string and reformats it using chrono formats (e.g., "%Y-%m-%d").
- **date.timestamp**: Converts an RFC3339 date string into a Unix timestamp (integer).
- **date.add(amount, unit)**: Adds the specified amount of time based on the unit (e.g., "days", "months", "years") to the date string.
- **date.subtract(amount, unit)**: Subtracts the specified amount of time based on the unit from the date string.
- **date.add_days(n)**: Adds n days to the date string.
- **date.sub_days(n)**: Subtracts n days from the date string.
- **date.add_months(n)**: Adds n months to the date string.
- **date.sub_months(n)**: Subtracts n months from the date string.
- **date.add_years(n)**: Adds n years to the date string.
- **date.sub_years(n)**: Subtracts n years from the date string.
- **date.diff_days(other_date_str)**: Returns the integer number of days between the piped date and the other date.
- **date.start_of_month**: Returns the date representing the first day of the given date's month.
- **date.end_of_month**: Returns the date representing the last day of the given date's month.
- **date.weekday**: Returns the day of the week as an integer (0-6).
- **date.is_leap_year**: Evaluates if an integer year or RFC3339 date falls in a leap year.
- **date.is_past**: Returns true if the date has already occurred.
- **date.is_future**: Returns true if the date is in the future.

## 🔤 String Module (str.*)

Text parsing, casing, and structural manipulation.

### Functions
None. All string operations are executed as pipeline filters.

### Filters
- **str.regex_replace(pattern, repl)**: Replaces substrings based on a regular expression. (Note: MiniJinja has a built-in replace for simple strings).
- **str.regex_match(pattern)**: Returns true if the string matches the regex pattern.
- **str.split(delimiter)**: Splits a string by a specific delimiter into an array. (Wrapper for built-in split).
- **str.split_lines**: Splits a string by \n or \r\n into an array.
- **str.trim**: Removes leading and trailing whitespace from the string. (Wrapper for built-in trim).
- **str.trim_prefix(prefix)**: Removes a specific prefix if it exists.
- **str.trim_suffix(suffix)**: Removes a specific suffix if it exists.
- **str.pad(length, char)**: Center-pads the string with the specified character up to the given length.
- **str.pad_left(length, char)**: Left-pads a string.
- **str.pad_right(length, char)**: Right-pads a string.
- **str.slugify**: Converts a string to a URL-friendly slug.
- **str.snake_case**: Converts "CamelCase" or "kebab-case" to "snake_case". (Wrapper for the convert_case crate).
- **str.camel_case**: Converts "snake_case" to "camelCase". (Wrapper for the convert_case crate).
- **str.pascal_case**: Converts "snake_case" to "PascalCase". (Wrapper for the convert_case crate).
- **str.kebab_case**: Converts "CamelCase" to "kebab-case". (Wrapper for the convert_case crate).
- **str.title_case**: Converts a string to "Title Case" (capitalizing the first letter of each word). (Wrapper for the convert_case crate or built-in title).
- **str.truncate(length, ellipsis)**: Truncates by character count, appending the ellipsis. (Wrapper for built-in truncate).
- **str.truncate_words(count, ellipsis)**: Truncates by word count, appending the ellipsis.
- **str.starts_with(substring)**: Returns true if the string begins with the substring. (Maps to built-in test startingwith).
- **str.ends_with(substring)**: Returns true if the string ends with the substring. (Maps to built-in test endingwith).
- **str.contains(substring)**: Returns true if the substring exists within the string.
- **str.length**: Returns the number of characters in the string. (Wrapper for built-in length).
- **str.word_count**: Returns the number of words in the string. (Wrapper for built-in wordcount).
- **str.reverse**: Reverses the string character by character. (Wrapper for built-in reverse).
- **str.repeat(n)**: Repeats the string n times.

## 🔢 Numeric Module (num.*)

Number operations, mathematical calculations, rounding, and type coercion.

### Functions
None. All num and coercion operations are executed as pipeline filters.

### Filters
- **num.clamp(min, max)**: Restricts a value to remain between min and max.
- **num.format(decimals)**: Formats a float to exactly n decimal places.
- **num.int**: Coerces a string or float to an i64. (Wrapper for built-in int).
- **num.float**: Coerces a string or integer to an f64. (Wrapper for built-in float).
- **num.abs**: Returns the absolute value. (Wrapper for built-in abs).
- **num.ceil**: Rounds a float up to the nearest integer.
- **num.floor**: Rounds a float down to the nearest integer.
- **num.round(decimals)**: Rounds a float to the nearest value at n decimal places. (Wrapper for built-in round).
- **num.is_even**: Returns true if an integer is even. (Maps to built-in test even).
- **num.is_odd**: Returns true if an integer is odd. (Maps to built-in test odd).
- **num.is_positive**: Returns true if the number is greater than zero.
- **num.is_negative**: Returns true if the number is less than zero.
- **num.pow(exponent)**: Raises the piped base to the given exponent.
- **num.sqrt**: Returns the square root of the number.
- **num.modulo(divisor)**: Returns the remainder of division.
- **num.to_hex**: Converts an integer to its hexadecimal string representation.
- **num.to_binary**: Converts an integer to its binary string representation.
- **num.to_octal**: Converts an integer to its octal string representation.

## 💬 Prompt Module (prompt.*)

Blocking UI interactions that request input from the user during template execution.

### Functions
- **prompt.text(message, default_value)**: Pauses execution to ask the user for text input. Returns the typed string.
- **prompt.select(message, options_array)**: Pauses execution to present a single-choice list to the user. Returns the selected option as a string.
- **prompt.multi_select(message, options_array)**: Pauses execution to present a multiple-choice list to the user. Returns an array of selected strings.

### Filters
None. Prompts inherently trigger side effects and generate data, so they are not used as pipeline filters.
