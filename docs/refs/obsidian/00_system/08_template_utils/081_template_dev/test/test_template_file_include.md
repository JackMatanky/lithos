<%*
/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
// Template file to include
const file_link = `[[${pillar_name_alias}]]`;

const path = "00_system/06_template_include/53_10_action_week_note_review_preview_one.md";
const abstract_file = await app.vault.getAbstractFileByPath(path);
const obs_include = await app.vault.cachedRead(abstract_file);

tR += obs_include;
%>
