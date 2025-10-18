async function vault_file(directory) {
  const obsidian_files = app.vault.getFiles();

  const full_file_paths = obsidian_files.filter((f) =>
    f.path.includes(directory)
  );

  const file_names = full_file_paths.map((f) => f.name).sort();

  const null_arr = ["null", "_user_input"];

  null_arr.push(file_names);

  const files = null_arr.flat();

  return files;
}

module.exports = vault_file;
