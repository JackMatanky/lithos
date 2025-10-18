// Returns an array of objects with file alias as key and file name as value from a given directory
async function md_file_name_alias(directory) {
  const markdownFiles = app.vault.getMarkdownFiles();

  // Filter files inside the specified directory
  const filePaths = markdownFiles.filter((f) => f.path.includes(directory));

  const fileObjArr = [];

  for (const file of filePaths) {
    const cache = await app.metadataCache.getFileCache(file);

    // Skip files without frontmatter or aliases
    if (!cache?.frontmatter?.aliases || !cache.frontmatter.aliases.length)
      continue;

    const alias = cache.frontmatter.aliases[0];
    const fileName = file.basename;

    fileObjArr.push({ key: alias, value: fileName });
  }

  // Sort alphabetically by alias (key)
  fileObjArr.sort((a, b) =>
    a.key.toLowerCase().localeCompare(b.key.toLowerCase())
  );

  // Default/fallback entries
  const defaultEntries = [
    { key: 'Null', value: 'null' },
    { key: 'User Input', value: '_user_input' },
  ];

  return [...defaultEntries, ...fileObjArr];
}

module.exports = md_file_name_alias;

// async function md_file_name_alias(directory) {
//   const obsidian_md_files = app.vault.getMarkdownFiles();

//   const file_paths = obsidian_md_files.filter((f) =>
//     f.path.includes(directory)
//   );

//   let file_name;
//   let file_alias;
//   let file_obj_arr = [];

//   for (let i = 0; i < file_paths.length; i++) {
//     // Retrieve the file alias
//     const file_cache = await app.metadataCache.getFileCache(file_paths[i]);
//     file_alias = file_cache.frontmatter.aliases[0];
//     // Retrieve the file name
//     file_name = file_paths[i].basename;
//     // Push the key-value object into the file object array
//     file_obj_arr.push({ key: file_alias, value: file_name });
//   }

//   // Sort the array by key
//   file_obj_arr.sort((a, b) => {
//     let key_a = a.key.toLowerCase(),
//       key_b = b.key.toLowerCase();

//     if (key_a < key_b) {
//       return -1;
//     }
//     if (key_a > key_b) {
//       return 1;
//     }
//     return 0;
//   });

//   // Add an object for null and user input values
//   const obj_arr = [
//     { key: "Null", value: "null" },
//     { key: "User Input", value: "_user_input" },
//   ];
//   // Append the file array object to the null array object
//   obj_arr.push(file_obj_arr);
//   // Reassign the flattened null array object to the file array object
//   file_obj_arr = obj_arr.flat();

//   return file_obj_arr;
// }

// module.exports = md_file_name_alias;
