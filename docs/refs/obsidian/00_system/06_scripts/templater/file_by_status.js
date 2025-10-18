async function file_by_status({ dir: directory, status: yaml_value }) {
  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((file) =>
    file.path.includes(directory)
  );

  const mapped_file_promises = file_paths.map(async (file) => {
    const file_cache = await app.metadataCache.getFileCache(file);

    const status_frontmatter = file_cache?.frontmatter?.status;

    // If the status frontmatter value
    // equals yaml_value, mark it to be included
    file.shouldInclude = status_frontmatter === yaml_value;

    return file;
  });

  // Wait for all files to be processed (have to wait because getting frontmatter is asynchronous)
  const mapped_files = await Promise.all(mapped_file_promises);

  // Filter out files that shouldn't be included
  const filtered_files = mapped_files.filter((file) => file.shouldInclude);

  // Create an array for the filtered files
  const filtered_files_arr = [];

  // Append the filtered files to the array
  filtered_files_arr.push(filtered_files);

  // Flatten the array from two dimensions to one
  const file_arr = filtered_files_arr.flat();

  // const file_obj_arr = file_arr;

  let file_name;
  let file_alias;
  let file_obj_arr = [];

  for (let i = 0; i < file_arr.length; i++) {
    // Retrieve the file alias
    const file_cache = await app.metadataCache.getFileCache(file_arr[i]);
    file_alias = file_cache.frontmatter.aliases[0];
    // Retrieve the file name
    file_name = file_arr[i].basename;
    // Push the key-value object into the file object array
    file_obj_arr.push({ key: file_alias, value: file_name });
  }

  // Sort the array by key
  file_obj_arr.sort((a, b) => {
    let key_a = a.key.toLowerCase(),
      key_b = b.key.toLowerCase();

    if (key_a < key_b) {
      return -1;
    }
    if (key_a > key_b) {
      return 1;
    }
    return 0;
  });

  // Add an object for null values
  const null_obj_arr = [{ key: "Null", value: "null" }];

  // Append the file array object to the null array object
  null_obj_arr.push(file_obj_arr);

  // Reassign the flattened null array object to the file array object
  file_obj_arr = null_obj_arr.flat();

  return file_obj_arr;
}

module.exports = file_by_status;
