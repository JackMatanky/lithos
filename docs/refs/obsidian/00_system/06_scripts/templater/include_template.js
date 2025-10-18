async function include_template(tp, file_basename) {
  const file_md_ext = `${file_basename}.md`;
  const file_path = await app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(file_md_ext))
    .map((file) => file.path);
  const abstract_file = await app.vault.getAbstractFileByPath(file_path);
  let tp_include;
  try {
    tp_include = await tp.file.include(abstract_file);
  } catch {
    tp_include = await tp.file.include(`[[${file_basename}]]`);
  }
  const file_content = tp_include.toString();

  return file_content;
}

module.exports = include_template;
