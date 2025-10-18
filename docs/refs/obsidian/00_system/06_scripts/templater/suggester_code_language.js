const lang_obj_arr = [
  { key: "AutoHotkey", value: "autohotkey", short: "ahk" },
  { key: "CSS", value: "css", short: "css" },
  { key: "Git", value: "git", short: "git" },
  { key: "Google Apps Script", value: "google_apps_script", short: "gas" },
  { key: "Google Sheets", value: "google_sheets", short: "gs" },
  { key: "HTML", value: "html", short: "html" },
  { key: "JavaScript", value: "javascript", short: "js" },
  { key: "Microsoft Excel", value: "ms_excel", short: "xl" },
  { key: "Python", value: "python", short: "py" },
  { key: "SQL", value: "sql", short: "sql" },
  { key: "Tableau", value: "tableau", short: "tbl" },
];

async function suggester_code_language(tp) {
  const lang_obj = await tp.system.suggester(
    (item) => item.key,
    lang_obj_arr,
    false,
    "Programming Language?"
  );
  const lang_name = lang_obj.key;
  const lang_value = lang_obj.value;
  const lang_value_short = lang_obj.short;
  const lang_link = ["[[" + lang_value, lang_name + "]]"].join("|");

  const language_obj = {
    name: lang_name,
    value: lang_value,
    value_short: lang_value_short,
    link: lang_link,
  };

  return language_obj;
}

module.exports = suggester_code_language;
