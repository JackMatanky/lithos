async function file_name_alias_by_class_type_status({
  dir: directory,
  file_class: yaml_class,
  type: yaml_type,
  status: yaml_status,
}) {
  // VAR CLASS: "task"
  // VAR TYPE: "project"
  // VAR STATUS: "to_do", "in_progress", "done", "discarded", "schedule", "on_hold", "active", "finish", "determine"
  // active equals "to_do" and "in_progress"
  // finish equals "to_do", "in_progress", and "done"
  // conclude equals "done" and "discarded"
  // determine equals "schedule" and "on_hold"
  const class_arg = yaml_class;
  const type_arg = yaml_type;
  const status_arg = yaml_status;

  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((file) =>
    file.path.includes(directory)
  );

  const mapped_file_promises = file_paths.map(async (file) => {
    const file_cache = await app.metadataCache.getFileCache(file);

    const class_frontmatter = file_cache?.frontmatter?.file_class;
    const type_frontmatter = file_cache?.frontmatter?.type;
    const status_frontmatter = file_cache?.frontmatter?.status;

    // If the file class and type frontmatter values
    // equal type_arg and start with class_arg
    // , mark it to be included
    if (status_arg == "") {
      file.shouldInclude =
        type_frontmatter == type_arg && class_frontmatter.startsWith(class_arg);
    } else if (status_arg == "active") {
      file.shouldInclude =
        type_frontmatter == type_arg &&
        class_frontmatter.startsWith(class_arg) &&
        (status_frontmatter == "to_do" || status_frontmatter == "in_progress");
    } else if (status_arg == "active_comp") {
      file.shouldInclude =
        type_frontmatter == type_arg &&
        class_frontmatter.startsWith(class_arg) &&
        (status_frontmatter == "to_do" ||
          status_frontmatter == "in_progress" ||
          status_frontmatter == "done");
    } else if (status_arg == "done") {
      file.shouldInclude =
        type_frontmatter == type_arg &&
        class_frontmatter.startsWith(class_arg) &&
        (status_frontmatter == "done" || status_frontmatter == "discarded");
    } else if (status_arg == "determine") {
      file.shouldInclude =
        type_frontmatter == type_arg &&
        class_frontmatter.startsWith(class_arg) &&
        (status_frontmatter == "schedule" || status_frontmatter == "on_hold");
    } else {
      file.shouldInclude =
        type_frontmatter == type_arg &&
        class_frontmatter.startsWith(class_arg) &&
        status_frontmatter == status_arg;
    }

    return file;
  });

  // Wait for all files to be processed
  // because getting frontmatter is asynchronous
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

  // Sort the array by file name
  file_obj_arr.sort((a, b) => {
    let value_a = a.value.toLowerCase(),
      value_b = b.value.toLowerCase();

    if (value_a < value_b) {
      return -1;
    }
    if (value_a > value_b) {
      return 1;
    }
    return 0;
  });

  // Add an object for null values
  const obj_arr = [{ key: "Null", value: "null" }];

  // Append the file array object to the null array object
  obj_arr.push(file_obj_arr);

  // Reassign the flattened null array object to the file array object
  file_obj_arr = obj_arr.flat();

  return file_obj_arr;
}

module.exports = file_name_alias_by_class_type_status;
