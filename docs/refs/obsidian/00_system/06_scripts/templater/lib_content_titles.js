async function lib_content_titles(str) {
  const regex_snake_case_under =
    /(;\s)|(:\s)|(\-\s\-)|(\s)|(\-)|([#:;\*<>\|\\/\-])/g;
  const regex_snake_case_remove = /(,|'|:|;|\?)/g;
  const snake_case_fmt = (name) =>
    name
      .replaceAll(regex_snake_case_under, '_')
      .replaceAll(regex_snake_case_remove, '')
      .toLowerCase();

  const initial_title = str.trim();
  const has_subtitle = initial_title.includes(':');

  // Split on first colon, if present
  const [raw_title, raw_subtitle] = initial_title
    .split(':')
    .map((title_part) => title_part?.trim());

  // Assign title and optional subtitle
  const title = raw_title;
  const title_value = snake_case_fmt(title);
  const subtitle = has_subtitle ? raw_subtitle : "";

  // Construct full display name and sanitized value
  const full_lib_title_name = has_subtitle ? `${title}: ${subtitle}` : title;

  const full_lib_title_value = has_subtitle
    ? `${snake_case_fmt(title)}_${snake_case_fmt(subtitle)}`
    : snake_case_fmt(title);

  return {
    full_title_name: full_lib_title_name,
    full_title_value: full_lib_title_value,
    main_title: title,
    main_title_value: title_value,
    sub_title: subtitle,
  };

  // Helper: Sanitize a string for filenames
  // const sanitize = (title_str) =>
  //   title_str
  //     .replace(/(\s)?[#:;*<>|\\/\-](\s)?/g, '_')
  //     .replace(/\?/g, '')
  //     .replace(/"/g, "'")
  //     .replace(/\s*/g, '_')
  //     .toLowerCase();

  // EXP: Check the initial title for a colon
  // const colon_index = initial_title.indexOf(':');

  // let full_lib_title_name;
  // let full_lib_title_value;

  // // EXP: If the title includes a colon, split into title and subtitle
  // if (colon_index === -1) {
  //   // EXP: If no colon is found,
  //   // EXP: assign the full title to the initial title
  //   title = initial_title;
  //   full_lib_title_name = title;
  //   full_lib_title_value = full_lib_title_name
  //     .replaceAll(/[#:\*<>\|\\/-]/g, '_')
  //     .replaceAll(/\?/g, '')
  //     .replaceAll(/"/g, "'")
  //     .toLowerCase();
  // } else {
  //   // EXP: If a colon is found,
  //   // EXP: split the initial title at the colon
  //   // EXP: into main and secondary titles
  //   title = initial_title.split(`:`)[0].trim();
  //   subtitle = initial_title.split(`:`)[1].trim();
  //   full_lib_title_name = `${title}: ${subtitle}`;
  //   title_value = title
  //     .replaceAll(/[#:\*<>\|\\/-\s]/g, '_')
  //     .replaceAll(/\?"/g, '')
  //     .toLowerCase();
  //   subtitle_value = subtitle
  //     .replaceAll(/[#:\*<>\|\\/-\s]/g, '_')
  //     .replaceAll(/\?"/g, '')
  //     .toLowerCase();
  //   full_lib_title_value = `${title_value}_${subtitle_value}`;
  // }

  //const title_obj = {
  //  full_title_name: full_lib_title_name,
  //  full_title_value: full_lib_title_value,
  //  main_title: title,
  //  sub_title: subtitle,
  //};

  // return title_obj;
}

module.exports = lib_content_titles;
