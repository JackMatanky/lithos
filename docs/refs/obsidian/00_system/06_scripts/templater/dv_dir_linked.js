// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//-------------------------------------------------------------------
// File name
const file_name = "file.name";

// Title
const yaml_title = "file.frontmatter.title";

// Alias
const yaml_alias = "file.frontmatter.aliases[0]";

// file title name and link
const title_link = `link(${file_name}, ${yaml_alias}) AS Name`;

// Tags
const tags = `file.etags AS Tags`;

// CONTACT
const yaml_job = "file.frontmatter.job_title";
const job_title = `${yaml_job} AS "Job Title"`

// Organization
const yaml_org = "file.frontmatter.organization";
const organization = `${yaml_org} AS Organization`;

// ORGANIZATION
const website = `elink(file.frontmatter.url, "Website") AS Website`;

const linkedin = `elink(file.frontmatter.linkedin_url, "LinkedIn") AS LinkedIn`;

const org_about = `file.frontmatter.about AS About`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA SOURCE
//-------------------------------------------------------------------
// Organization and Contacts directory
const contact_dir = `"51_contacts"`;
const organization_dir = `"52_organizations"`;

//-------------------------------------------------------------------
// SECT: >>>>> DATA FILTER
//-------------------------------------------------------------------
// Current file filter
const current_file_filter = `file.name != this.file.name`;

// File inlinks and outlinks
const in_out_link_filter = `(contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))`;

// File class filter
const class_filter = `contains(file.frontmatter.file_class, "dir")`;

//-------------------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//-------------------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE: "contact", "organization"

async function dv_dir_linked(type, md) {
  const type_arg = `${type}`;
  const md_arg = `${md}`;

  const type_filter = `contains(file.frontmatter.file_class, "${type_arg}")`;

  let dataview_query;

  if (type_arg.startsWith("cont")) {
    const directory_dir = contact_dir;
    // Table for linked CONTACTS
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${title_link},
    ${job_title},
    ${organization},
    ${tags}
FROM
    ${directory_dir}
WHERE
    ${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${in_out_link_filter}
SORT
    ${yaml_title} ASC
${three_backtick}`;
  } else {
    const directory_dir = organization_dir;
    // Table for linked ORGANIZATIONS
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
    ${title_link},
    ${website},
    ${linkedin},
    ${org_about},
    ${tags}
FROM
    ${directory_dir}
WHERE
    ${current_file_filter}
    AND ${class_filter}
    AND ${type_filter}
    AND ${in_out_link_filter}
SORT
    ${yaml_title} ASC
${three_backtick}`;
  }
  if (md_arg == "true") {
    const dataview_block_start_regex = /^```dataview\n/g;
    const dataview_block_end_regex = /\n```$/g;

    const md_query = String(
      dataview_query
        .replace(dataview_block_start_regex, "")
        .replace(dataview_block_end_regex, "")
        .replaceAll(/\n\s+/g, " ")
        .replaceAll(/\n/g, " ")
    );

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
  }
  return dataview_query;
}

module.exports = dv_dir_linked;
