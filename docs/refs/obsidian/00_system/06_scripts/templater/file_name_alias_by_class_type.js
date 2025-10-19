async function file_name_alias_by_class_type({
  dir: directory,
  file_class: yaml_class,
  type: yaml_type,
}) {
  const obsidian_md_files = app.vault.getMarkdownFiles();

  const file_paths = obsidian_md_files.filter((file) =>
    file.path.includes(directory)
  );

  const mapped_file_promises = file_paths.map(async (file) => {
    const file_cache = await app.metadataCache.getFileCache(file);

    const class_frontmatter = file_cache?.frontmatter?.file_class;
    const type_frontmatter = file_cache?.frontmatter?.type;

    // If the file class and type frontmatter values
    // equal yaml_type and start with yaml_class
    // , mark it to be included
    if (!yaml_type) {
      file.shouldInclude =
        class_frontmatter && class_frontmatter.startsWith(yaml_class);
    } else if (yaml_class == "task" && yaml_type == "parent_task") {
      file.shouldInclude =
        class_frontmatter && class_frontmatter.startsWith("task_parent");
    } else {
      file.shouldInclude =
        type_frontmatter == yaml_type &&
        class_frontmatter &&
        class_frontmatter.startsWith(yaml_class);
    }
    // let first_element = "";
    // if (Array.isArray(class_frontmatter)) {
    //   firstElement = class_frontmatter[0];
    // }
    // if (yaml_type == "" && typeof firstElement === "string") {
    //   file.shouldInclude = firstElement.startsWith(yaml_class);
    // } else if (typeof firstElement === "string") {
    //   file.shouldInclude =
    //     type_frontmatter == yaml_type && firstElement.startsWith(yaml_class);
    // }
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
  const obj_arr = [
    { key: "Null", value: "null" },
    { key: "User Input", value: "_user_input" },
  ];

  // Append the file array object to the null array object
  obj_arr.push(file_obj_arr);

  // Reassign the flattened null array object to the file array object
  file_obj_arr = obj_arr.flat();

  return file_obj_arr;
}

module.exports = file_name_alias_by_class_type;
