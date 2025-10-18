<%*
/* ---------------------------------------------------------- */
/*                    FORMATTING CHARACTERS                   */
/* ---------------------------------------------------------- */
//Characters
const new_line = String.fromCodePoint(0xa);
const two_new_line = new_line.repeat(2);
const space = String.fromCodePoint(0x20);
const two_space = space.repeat(2);
const hash = String.fromCodePoint(0x23);
const hyphen = String.fromCodePoint(0x2d);
const two_hyphen = hyphen.repeat(2);
const hr_line = hyphen.repeat(3);
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const colon = String.fromCodePoint(0x3a);
const two_percent = String.fromCodePoint(0x25).repeat(2);
const less_than = String.fromCodePoint(0x3c);
const great_than = String.fromCodePoint(0x3e);
const excl = String.fromCodePoint(0x21);

//Text Formatting
const tbl_start = `${String.fromCodePoint(0x7c)}${space}`;
const tbl_pipe = `${space}${String.fromCodePoint(0x7c)}${space}`;
const tbl_end = `${space}${String.fromCodePoint(0x7c)}`;
const tbl_left = `${colon}${hyphen.repeat(8)}${space}`;
const tbl_right = `${space}${hyphen.repeat(8)}${colon}`;
const tbl_cent = `${colon}${hyphen.repeat(8)}${colon}`;
const ul = `${hyphen}${space}`;
const ul_yaml = `${space.repeat(2)}${ul}`;
const checkbox = `${ul}[${space}]${space}`;
const call_start = `${great_than}${space}`;
const call_ul = `${call_start}${ul}`;
const call_ul_indent = `${call_start}${space.repeat(4)}${ul}`;
const call_check = `${call_start}${checkbox}`;
const call_check_indent = `${call_start}${space.repeat(4)}${checkbox}`;
const call_tbl_start = `${call_start}${tbl_start}`;
const call_tbl_end = `${tbl_end}${two_space}`;
const dv_colon = `${colon.repeat(2)}${space}`;

/* ---------------------------------------------------------- */
/*                      HELPER FUNCTIONS                      */
/* ---------------------------------------------------------- */
// FORMATTING
const head_lvl = (level, heading) => [hash.repeat(level), heading].join(space);
const regex_snake_case_under = /(;\s)|(:\s)|(\-\s\-)|(\s)|(\-)/g;
const regex_snake_case_remove = /(,|'|:|;)/g;
const snake_case_fmt = (name) =>
  name
    .replaceAll(regex_snake_case_under, '_')
    .replaceAll(regex_snake_case_remove, '')
    .toLowerCase();
const md_ext = (file_name) => file_name + '.md';
const quote_enclose = (content) => `"${content}"`;

const code_inline = (content) => backtick + content + backtick;
const cmnt_obsidian = (content) =>
  [two_percent, content, two_percent].join(space);
const cmnt_html = (content) =>
  [less_than + excl + two_hyphen, content, two_hyphen + great_than].join(space);

// LINKS
const regex_link = /.*\[([\w_].+)\|([\w\s].+)\].+/g;
const link_alias = (file, alias) => '[[' + [file, alias].join('|') + ']]';
const link_tbl_alias = (file, alias) => '[[' + [file, alias].join('\\|') + ']]';

// YAML PROPERTIES
const yaml_li = (value) => new_line + ul_yaml + `"${value}"`;
const yaml_li_link = (file, alias) =>
  new_line + ul_yaml + `"${link_alias(file, alias)}"`;

// CALLOUT
const call_title = (call_type, title) =>
  [great_than, `[!${call_type}]`, title].join(space);

// CALLOUT TABLE
const call_tbl_row = (content) =>
  [
    great_than,
    String.fromCodePoint(0x7c),
    content,
    String.fromCodePoint(0x7c),
    space,
  ].join(space);
const call_tbl_div = (int) =>
  call_tbl_row(Array(int).fill(tbl_cent).join(tbl_pipe));

/* ---------------------------------------------------------- */
/*                  RELATED DIRECTORY SECTION                 */
/* ---------------------------------------------------------- */
// Outgoing Contacts
//heading = `${head_lvl(3)}Outgoing Contact Links${two_new_line}`;
//comment = `${cmnt_html_start}Link related contacts here${cmnt_html_end}${two_new_line}`;
//const contact_outlink = `${heading}${comment}`;

// Contacts
// heading = `${head_lvl(3)}Contacts${two_new_line}`;
// query = await tp.user.dv_dir_linked("contact", "false");
// const dir_contact = `${heading}${query}${two_new_line}`;

// Outgoing Organizations
// heading = `${head_lvl(3)}Outgoing Organization Links${two_new_line}`;
// comment = `${cmnt_html_start}Link related organizations here${cmnt_html_end}${two_new_line}`;
// const org_outlink = `${heading}${comment}`;

// Organizations
// heading = `${head_lvl(3)}Organizations${two_new_line}`;
// query = await tp.user.dv_dir_linked("organization", "false");
// const dir_org = `${heading}${query}${two_new_line}`;

/* --------------------- BUTTONS CALLOUT --------------------- */
const sectionReplaceButton = await tp.user.include_file(
  '00_142_buttons_related_sect_task_child'
);

const buttonsSection = [
  cmnt_html('Adjust replace lines'),
  sectionReplaceButton,
].join(two_new_line);

/* ------ DYNAMIC DATAVIEW QUERIES AND OUTLINK SECTIONS ----- */
const subsectionArr = ['Contact', 'Organization'];

const outlinkQuerySections = await Promise.all(
  subsectionArr.map(async (sect) => {
    const subjectPlural = `${sect}s`;
    const subjectLower = sect.toLowerCase();

    const outlink = [
      head_lvl(3, `Outgoing ${sect} Links`),
      cmnt_html(`Link related ${subjectPlural.toLowerCase()} here`),
    ].join(two_new_line);

    const query = [
      head_lvl(3, subjectPlural),
      await tp.user.dv_dir_linked(subjectLower, 'false'),
    ].join(two_new_line);

    return [outlink, query].join(two_new_line);
  })
);

/* --------------------- FINAL ASSEMBLY --------------------- */
const finalOutput =
  [buttonsSection, outlinkQuerySections.join(two_new_line), hr_line].join(
    two_new_line
  ) + new_line;

tR += finalOutput;
%>