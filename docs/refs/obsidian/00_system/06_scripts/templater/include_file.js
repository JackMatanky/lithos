async function include_file(file_basename) {
  const file_md_ext = `${file_basename}.md`;
  const file_path = app.vault
    .getMarkdownFiles()
    .filter((file) => file.path.endsWith(file_md_ext))
    .map((file) => file.path);
  const abstract_file = app.vault.getAbstractFileByPath(file_path);
  const file_content = app.vault.cachedRead(abstract_file);

  return file_content;
}

module.exports = include_file;
