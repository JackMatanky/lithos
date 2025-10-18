async function folder_name({ dir: directory, index: path_index }) {
  const obsidian_folders = app.vault.getAllLoadedFiles();

  // Filter the directory paths to those with files
  const all_folder_paths = obsidian_folders
    .filter((i) => i.children)
    .map((folder) => folder.path);

  const folder_paths = all_folder_paths.filter((folder_path) =>
    folder_path.includes(directory)
  );

  const folder_names = folder_paths
    .map((folder_path) => folder_path.split("/")[path_index])
    .filter((folder_name) => folder_name);

  const folders_set = [...new Set(folder_names)];

  folders_set.sort();

  const folders_arr = ["null"];

  folders_arr.push(folders_set);

  const folders = folders_arr.flat();

  return folders;
}

module.exports = folder_name;
