async function md_file_name(directory) {
  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((f) =>
    f.path.includes(directory)
  );

  const file_names = file_paths.map((f) => f.basename).sort();

  const files_arr = ["null"];

  files_arr.push(file_names);

  const files = files_arr.flat();

  return files;
};

module.exports = md_file_name;
