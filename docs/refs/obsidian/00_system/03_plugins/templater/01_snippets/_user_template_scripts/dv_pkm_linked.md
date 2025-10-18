---
title: dv_pkm_linked
aliases:
  - Linked Personal Knowledge Files Dataview Table
  - Dataview Table for Linked Personal Knowledge Files
  - dv pkm linked
plugin: templater
language:
  - javascript
module:
  - user
cssclasses:
type: snippet
file_class: pkm_code
date_created: 2023-07-12T13:46
date_modified: 2023-10-25T16:23
tags: obsidian/templater, javascript
---
# Linked Knowledge Files Dataview Table

## Description

> [!snippet] Snippet Details
>  
> Plugin: [[Templater]]  
> Language: [[JavaScript]]  
> Input::  
> Output::  
> Description:: Return a dataview table or markdown table for linked personal knowledge files based on specific file class and type.

---

## Snippet

<!-- Add the full code including explanatory comments  -->

```javascript
// SECT: >>>>> DATAVIEW API <<<<<
const dv = app.plugins.plugins["dataview"].api;

//---------------------------------------------------------
// SECT: >>>>> DATA FIELDS <<<<<
//---------------------------------------------------------
// Title
const yaml_title = `file.frontmatter.title`;

// Alias
const alias = `file.frontmatter.aliases[0]`;

// Title link
const title_link = `link(file.link, ${alias}) AS Title`;

// Title link for DV query
const md_title_link = `"[[" + file.name + "\|" + ${alias} + "]]" AS Title`;

// YAML PKM Category
const yaml_category = `file.frontmatter.category`;

// YAML PKM Branch
const yaml_branch = `file.frontmatter.branch`;

// YAML PKM Field
const yaml_field = `file.frontmatter.field`;

// YAML PKM Subject
const yaml_subject = `file.frontmatter.subject`;

// YAML PKM Topic
const yaml_topic = `file.frontmatter.topic`;

// Status
const yaml_status = `file.frontmatter.status`;
const pkm_status = `choice(${yaml_status} = "schedule", "🤷Unknown", 
    choice(${yaml_status} = "review", "🔜Review", 
    choice(${yaml_status} = "clarify", "🌱Clarify", 
    choice(${yaml_status} = "develop", "🪴Develop",
    choice(${yaml_status} = "done", "🌳Done", "🗄️Resource")))))
	AS Status`;

// File type
const yaml_type = `file.frontmatter.type`;

// File subtype
const yaml_subtype = `file.frontmatter.subtype`;
const pkm_subtype = `choice(contains(${yaml_subtype}, "category"), "🏘️Category",
	choice(contains(${yaml_subtype}, "branch"), "🪑Branch",
	choice(contains(${yaml_subtype}, "field"), "🚪Field",
	choice(contains(${yaml_subtype}, "subject"), "🗝️Subject",
	choice(contains(${yaml_subtype}, "topic"), "🧱Topic", 
	choice(contains(${yaml_subtype}, "subtopic"), "🔩Subtopic",
	choice(contains(${yaml_subtype}, "question"), "❔Question",
	choice(contains(${yaml_subtype}, "evidence"), "⚖️Evidence",
	choice(contains(${yaml_subtype}, "conclusion"), "💡Conclusion",
	choice(contains(${yaml_subtype}, "problem"), "🪨Problem",
	choice(contains(${yaml_subtype}, "step"), "🪜Step",
	choice(contains(${yaml_subtype}, "answer"), "🎱Answer",
	choice(contains(${yaml_subtype}, "quote"), "⏺️Quote",
	choice(contains(${yaml_subtype}, "idea"), "💭Idea",
	choice(contains(${yaml_subtype}, "concept"), "🎞️Concept", "🪟Definition")))))))))))))))
	AS Subtype`;

const tree_subtype = `choice(contains(${yaml_subtype}, "category"), "🏘️Category",
	choice(contains(${yaml_subtype}, "branch"), "🪑Branch",
	choice(contains(${yaml_subtype}, "field"), "🚪Field",
	choice(contains(${yaml_subtype}, "subject"), "🗝️Subject",
	choice(contains(${yaml_subtype}, "topic"), "🧱Topic", "🔩Subtopic")))))
	AS Subtype`;

const note_subtype = `choice(contains(${yaml_subtype}, "question"), "❔Question",
	choice(contains(${yaml_subtype}, "evidence"), "⚖️Evidence",
	choice(contains(${yaml_subtype}, "conclusion"), "🏁Conclusion",
	choice(contains(${yaml_subtype}, "problem"), "🪨Problem",
	choice(contains(${yaml_subtype}, "step"), "🪜Step",
	choice(contains(${yaml_subtype}, "answer"), "🎱Answer",
	choice(contains(${yaml_subtype}, "quote"), "⏺️Quote",
	choice(contains(${yaml_subtype}, "idea"), "💭Idea",
	choice(contains(${yaml_subtype}, "concept"), "🎞️Concept", "🪟Definition")))))))))
	AS Subtype`;

// Tags
const tags = `file.etags AS Tags`;

const note_content = `choice(${yaml_subtype} = "qec_question", list(Context, Question),
	choice(${yaml_subtype} = "qec_evidence", Evidence,
	choice(${yaml_subtype} = "qec_conclusion", Conclusion,
	choice(${yaml_subtype} = "psa_problem", list(Context, Problem),
	choice(${yaml_subtype} = "psa_step", Step,
	choice(${yaml_subtype} = "psa_answer", Answer,
	choice(${yaml_subtype} = "quote", Quote,
	choice(${yaml_subtype} = "idea", Idea,
	choice(${yaml_subtype} = "concept", Description, Definition)))))))))
	AS Content`;

const tree_description = `Description AS Description`;

const tree_context = `choice(${yaml_subtype} = "subtopic", flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject, file.frontmatter.topic)),
	choice(${yaml_subtype} = "topic", flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject)), 
	choice(${yaml_subtype} = "subject", flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field)),
	choice(${yaml_subtype} = "field", flat(list(file.frontmatter.category, file.frontmatter.branch)), 
	choice(${yaml_subtype} = "branch", Category, "")))))
	AS Context`;

//---------------------------------------------------------
// SECT: >>>>> DATA SOURCE <<<<<
//---------------------------------------------------------
// Knowledge tree directory
const tree_dir = `"70_pkm_tree"`;

// Knowledge lab directory
const lab_dir = `"80_pkm_lab"`;

//---------------------------------------------------------
// SECT: >>>>> DATA FILTER <<<<<
//---------------------------------------------------------
// File class filter
const class_filter = `contains(file.frontmatter.file_class, "pkm")`;

// Current file filter
const current_file_filter = `file.name != this.file.name`;

// File inlinks and outlinks
const in_out_link_filter = `(contains(file.outlinks, this.file.link) 
	OR contains(file.inlinks, this.file.link))`;

// Tree child filter
const tree_child_filter = `(choice(contains(this.${yaml_subtype}, "category"), 
		contains(${yaml_category}, this.file.name),
		choice(contains(this.${yaml_subtype}, "branch"), 
			contains(${yaml_branch}, this.file.name),
			choice(contains(this.${yaml_subtype}, "field"), 
				contains(${yaml_field}, this.file.name),
				choice(contains(this.${yaml_subtype}, "subject"), 
					contains(${yaml_subject}, this.file.name), 
					contains(${yaml_topic}, this.file.name))))))`;

const tree_sibling_filter = `(choice(contains(this.${yaml_subtype}, "subtopic"), 
		contains(this.${yaml_topic}, ${yaml_topic}),
		choice(contains(this.${yaml_subtype}, "topic"), 
			contains(this.${yaml_subject}, ${yaml_subject}),
			choice(contains(this.${yaml_subtype}, "subject"), 
				contains(this.${yaml_field}, ${yaml_field}),
				choice(contains(this.${yaml_subtype}, "field"), 
					contains(this.${yaml_branch}, ${yaml_branch}), 
					contains(this.${yaml_category}, ${yaml_category}))))))`;

//---------------------------------------------------------
// SECT: >>>>> DATA SORTING <<<<<
//---------------------------------------------------------
const tree_subtype_sort = `choice(${yaml_subtype} = "category", 1,
	choice(${yaml_subtype} = "branch", 2,
	choice(${yaml_subtype} = "field", 3,
	choice(${yaml_subtype} = "subject", 4,
	choice(${yaml_subtype} = "topic", 5, 6)))))`;

const note_subtype_sort = `choice(${yaml_subtype} = "qec_question", 1,
	choice(${yaml_subtype} = "qec_evidence", 2,
	choice(${yaml_subtype} = "qec_conclusion", 3,
	choice(${yaml_subtype} = "psa_problem", 4,
	choice(${yaml_subtype} = "psa_step", 5,
	choice(${yaml_subtype} = "psa_answer", 6,
	choice(${yaml_subtype} = "quote", 7,
	choice(${yaml_subtype} = "idea", 8,
	choice(${yaml_subtype} = "concept", 9, 10)))))))))`;

//---------------------------------------------------------
// RELATED FILES DATAVIEW TABLES
//---------------------------------------------------------
// Unicode for backticks
const backtick = String.fromCodePoint(0x60);
const three_backtick = backtick.repeat(3);
const dataview_block = `${three_backtick}dataview`;

// VAR MD: "true", "false"
// VAR TYPE: "tree", "permanent", "literature", "fleeting", "info"
// VAR SUBTYPE: "category", "branch", "field", "subject", "topic", "subtopic", "qec_question", "qec_evidence", "qec_conclusion", "psa_problem", "psa_step", "psa_answer", "quote", "idea", "concept", "definition"
// VAR RELATION: "linked", "parent", "child", "sibling"
// EXP: "linked" for linked all pkm external to their hierarchy;
// EXP: "parent" for hierarchical anscestors
// EXP: "child" for direct descendents in hierarchy;
// EXP: "sibling" for pkm tree notes under the same tree object or qec and psa notes connected to each other in the relations callout.

async function dv_pkm_linked({
  type: type,
  subtype: subtype,
  relation: relation,
  md: md,
}) {
  const type_arg = `${type}`;
  const subtype_arg = `${subtype}`;
  const relation_arg = `${relation}`;
  const md_arg = `${md}`;

  const type_filter = `contains(${yaml_type}, "${type_arg}")`;
  let subtype_filter = `contains(${yaml_subtype}, "${subtype_arg}")`;
  let relation_filter;

  let dataview_query;
  if (type_arg.startsWith("know")) {
    if (relation_arg.startsWith("link")) {
      if (subtype_arg == "") {
        // Table for all linked KNOWLEDGE TREE HIERARCHY files
        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_subtype},
	${tree_description},
	${tree_context},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${in_out_link_filter}
SORT 
	${tree_subtype_sort},
	${yaml_title} ASC
${three_backtick}`;
      } else {
        // Table for linked KNOWLEDGE TREE HIERARCHY files by subtype
        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_subtype},
	${tree_description},
	${tree_context},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${in_out_link_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
      }
    } else if (relation_arg.startsWith("parent")) {
      if (subtype_arg.startsWith("branch")) {
        // Table for a BRANCHES's parent CATEGORY
        relation_filter = `contains(this.${yaml_category}, file.name)`;
        subtype_filter = `contains(${yaml_subtype}, "category")`;
        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_subtype},
	${tree_description}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${tree_subtype_sort},
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("field")) {
        // Table for a FIELD's parent BRANCH and CATEGORY
        relation_filter = `(contains(this.${yaml_category}, file.name)
		OR contains(this.${yaml_branch}, file.name))`;
        subtype_filter = `(contains(${yaml_subtype}, "category")
		OR contains(${yaml_subtype}, "branch"))`;
        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_subtype},
	${tree_description}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${tree_subtype_sort},
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("subject")) {
        // Table for a SUBJECT's parent FIELD, BRANCH, and CATEGORY
        relation_filter = `(contains(this.${yaml_category}, file.name)
		OR contains(this.${yaml_branch}, file.name)
		OR contains(this.${yaml_field}, file.name))`;
        subtype_filter = `(contains(${yaml_subtype}, "category")
		OR contains(${yaml_subtype}, "branch")
		OR contains(${yaml_subtype}, "field"))`;
        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_subtype},
	${tree_description}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${tree_subtype_sort},
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("topic")) {
        // Table for a TOPIC's parent SUBJECT, FIELD, BRANCH, and CATEGORY
        relation_filter = `(contains(this.${yaml_category}, file.name)
		OR contains(this.${yaml_branch}, file.name)
		OR contains(this.${yaml_field}, file.name)
		OR contains(this.${yaml_subject}, file.name))`;
        subtype_filter = `(contains(${yaml_subtype}, "category")
		OR contains(${yaml_subtype}, "branch")
		OR contains(${yaml_subtype}, "field")
		OR contains(${yaml_subtype}, "subject"))`;
        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_subtype},
	${tree_description}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${tree_subtype_sort},
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("subtopic")) {
        // Table for a SUBTOPIC's parent TOPIC, SUBJECT, FIELD, BRANCH, and CATEGORY
        relation_filter = `(contains(this.${yaml_category}, file.name)
		OR contains(this.${yaml_branch}, file.name)
		OR contains(this.${yaml_field}, file.name)
		OR contains(this.${yaml_subject}, file.name)
		OR contains(this.${yaml_topic}, file.name))`;
        subtype_filter = `(contains(${yaml_subtype}, "category")
		OR contains(${yaml_subtype}, "branch")
		OR contains(${yaml_subtype}, "field")
		OR contains(${yaml_subtype}, "subject")
		OR contains(${yaml_subtype}, "topic"))`;
        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_subtype},
	${tree_description}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${tree_subtype_sort},
	${yaml_title} ASC
${three_backtick}`;
      }
    } else if (relation_arg.startsWith("child")) {
      if (subtype_arg.startsWith("category")) {
        if (relation_arg.startsWith("child_b")) {
          // Table for a CATEGORY's direct BRANCHES
          relation_filter = `contains(${yaml_category}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "branch")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_f")) {
          // Table for a CATEGORY's direct FIELDS
          relation_filter = `contains(${yaml_category}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "field")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_subj")) {
          // Table for a CATEGORY's direct SUBJECTs
          relation_filter = `contains(${yaml_category}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subject")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_t")) {
          // Table for a CATEGORY's direct TOPICS
          relation_filter = `contains(${yaml_category}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "topic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_subt")) {
          // Table for a CATEGORY's direct SUBTOPICS
          relation_filter = `contains(${yaml_category}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subtopic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        }
      } else if (subtype_arg.startsWith("branch")) {
        if (relation_arg.startsWith("child_f")) {
          // Table for a BRANCH's direct FIELDS
          relation_filter = `contains(${yaml_branch}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "field")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_subj")) {
          // Table for a BRANCH's direct SUBJECTs
          relation_filter = `contains(${yaml_branch}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subject")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_t")) {
          // Table for a BRANCH's direct TOPICS
          relation_filter = `contains(${yaml_branch}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "topic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_subt")) {
          // Table for a BRANCH's direct SUBTOPICS
          relation_filter = `contains(${yaml_branch}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subtopic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        }
      } else if (subtype_arg.startsWith("field")) {
        if (relation_arg.startsWith("child_subj")) {
          // Table for a FIELD's direct SUBJECTs
          relation_filter = `contains(${yaml_field}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subject")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_t")) {
          // Table for a FIELD's direct TOPICS
          relation_filter = `contains(${yaml_field}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "topic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_subt")) {
          // Table for a FIELD's direct SUBTOPICS
          relation_filter = `contains(${yaml_field}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subtopic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        }
      } else if (subtype_arg.startsWith("subject")) {
        if (relation_arg.startsWith("child_t")) {
          // Table for a SUBJECT's direct TOPICS
          relation_filter = `contains(${yaml_subject}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "topic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        } else if (relation_arg.startsWith("child_subt")) {
          // Table for a SUBJECT's direct SUBTOPICS
          relation_filter = `contains(${yaml_subject}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subtopic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        }
      } else if (subtype_arg.startsWith("topic")) {
        if (relation_arg.startsWith("child_subt")) {
          // Table for a TOPIC's direct SUBTOPICS
          relation_filter = `contains(${yaml_topic}, this.file.name)`;
          subtype_filter = `contains(${yaml_subtype}, "subtopic")`;
          dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
        }
      }
    } else if (relation_arg.startsWith("sib")) {
      if (subtype_arg.startsWith("branch")) {
        relation_filter = `(contains(this.${yaml_category}, ${yaml_category}[0])
		OR contains(this.${yaml_category}, ${yaml_category}[1]))`;

        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tree_context},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("field")) {
        relation_filter = `(contains(this.${yaml_branch}, ${yaml_branch}[0])
		OR contains(this.${yaml_branch}, ${yaml_branch}[1]))`;

        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tree_context},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("subject")) {
        relation_filter = `(contains(this.${yaml_field}, ${yaml_field}[0])
		OR contains(this.${yaml_field}, ${yaml_field}[1]))`;

        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tree_context},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("topic")) {
        relation_filter = `(contains(this.${yaml_subject}, ${yaml_subject}[0])
		OR contains(this.${yaml_subject}, ${yaml_subject}[1]))`;

        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tree_context},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
      } else if (subtype_arg.startsWith("subtopic")) {
        relation_filter = `(contains(this.${yaml_topic}, ${yaml_topic}[0])
		OR contains(this.${yaml_topic}, ${yaml_topic}[1]))`;

        dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${tree_description},
	${tree_context},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${class_filter}
	AND ${type_filter}
	AND ${subtype_filter}
	AND ${relation_filter}
SORT 
	${yaml_title} ASC
${three_backtick}`;
      }
    }
  }

  if (
    type_arg.startsWith("perm") ||
    type_arg.startsWith("lit") ||
    type_arg.startsWith("fleet")
  ) {
    // Table for linked PERMANENT, LITERATURE, AND FLEETING NOTES
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${note_subtype},
	${pkm_status},
	${note_content},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	${note_subtype_sort},
	${yaml_title} ASC
${three_backtick}`;
  }

  if (type_arg.startsWith("info")) {
    // Table for linked CONCEPT, DEFINITION, AND GENERAL NOTES
    dataview_query = `${dataview_block}
TABLE WITHOUT ID
	${title_link},
	${note_subtype},
	${pkm_status},
	${note_content},
	${tags}
FROM
	${tree_dir}
	OR ${lab_dir}
WHERE
	${current_file_filter}
	AND ${in_out_link_filter}
	AND ${class_filter}
	AND ${type_filter}
SORT 
	${note_subtype_sort},
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
        .replace(title_link, md_title_link)
    );

    const markdown = await dv.queryMarkdown(md_query);
    dataview_query = markdown.value;
  }

  return dataview_query;
}

module.exports = dv_pkm_linked;
```

### Templater

<!-- Add the full code as it should appear in the template  -->  
<!-- Exclude explanatory comments  -->

```javascript
//---------------------------------------------------------
// RELATED PKM FILES DATAVIEW TABLE
//---------------------------------------------------------
// MD: "true", "false"
// TYPE: "tree", "permanent", "literature", "fleeting", "info"
// SUBTYPE: "category", "branch", "field", "subject", "topic", "subtopic", "qec_question", "qec_evidence", "qec_conclusion", "psa_problem", "psa_step", "psa_answer", "quote", "idea", "concept", "definition"
// RELATION: "linked", "parent", "child", "child_b", "child_f", "child_subj", "child_t", "child_subt", "sibling"
// "linked" for linked all pkm external to their hierarchy;
// "parent" for hierarchical anscestors
// "child" for direct descendents in hierarchy;
// "sibling" for pkm tree notes under the same tree object or qec and psa notes connected to each other in the relations callout.
const linked_pkm_file_table = await tp.user.dv_pkm_linked({
  type: type,
  subtype: subtype,
  relation: relation,
  md: md,
})
```

#### Examples

```javascript
//---------------------------------------------------------
// RELATED PKM FILES DATAVIEW TABLE
//---------------------------------------------------------
// LINKED KNOWLEDGE TREE FILES TABLES
const pkm_tree_link = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "",
  relation: "linked",
  md: "false",
})

const pkm_category_link = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "linked",
  md: "false",
})

const pkm_branch_link = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "branch",
  relation: "linked",
  md: "false",
})

const pkm_field_link = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "field",
  relation: "linked",
  md: "false",
})

const pkm_subject_link = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subject",
  relation: "linked",
  md: "false",
})

const pkm_topic_link = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "topic",
  relation: "linked",
  md: "false",
})

const pkm_subtopic_link = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subtopic",
  relation: "linked",
  md: "false",
})

// CHILD KNOWLEDGE TREE FILES TABLES
const pkm_category_child_branch = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_branch",
  md: "false",
})

const pkm_category_child_field = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_field",
  md: "false",
})

const pkm_category_child_subject = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_subject",
  md: "false",
})

const pkm_category_child_topic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_topic",
  md: "false",
})

const pkm_category_child_subtopic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "category",
  relation: "child_subtopic",
  md: "false",
})

const pkm_branch_child_field = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "branch",
  relation: "child_field",
  md: "false",
})

const pkm_branch_child_subject = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "branch",
  relation: "child_subject",
  md: "false",
})

const pkm_branch_child_topic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "branch",
  relation: "child_topic",
  md: "false",
})

const pkm_branch_child_subtopic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "branch",
  relation: "child_subtopic",
  md: "false",
})

const pkm_field_child_subject = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "field",
  relation: "child_subject",
  md: "false",
})

const pkm_field_child_topic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "field",
  relation: "child_topic",
  md: "false",
})

const pkm_field_child_subtopic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "field",
  relation: "child_subtopic",
  md: "false",
})

const pkm_subject_child_topic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subject",
  relation: "child_topic",
  md: "false",
})

const pkm_subject_child_subtopic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subject",
  relation: "child_subtopic",
  md: "false",
})

const pkm_topic_child_subtopic = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "topic",
  relation: "child_subtopic",
  md: "false",
})

// PARENT KNOWLEDGE TREE FILES TABLES
const pkm_branch_parent = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "branch",
  relation: "parent",
  md: "false",
})

const pkm_field_parent = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "field",
  relation: "parent",
  md: "false",
})

const pkm_subject_parent = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subject",
  relation: "parent",
  md: "false",
})

const pkm_topic_parent = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "topic",
  relation: "parent",
  md: "false",
})

const pkm_subtopic_parent = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subtopic",
  relation: "parent",
  md: "false",
})

// SIBLING KNOWLEDGE TREE FILES TABLES
const pkm_tree_sibling = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "",
  relation: "sibling",
  md: "false",
})

const pkm_branch_sibling = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "branch",
  relation: "sibling",
  md: "false",
})

const pkm_field_sibling = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "field",
  relation: "sibling",
  md: "false",
})

const pkm_subject_sibling = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subject",
  relation: "sibling",
  md: "false",
})

const pkm_topic_sibling = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "topic",
  relation: "sibling",
  md: "false",
})

const pkm_subtopic_sibling = await tp.user.dv_pkm_linked({
  type: "tree",
  subtype: "subtopic",
  relation: "sibling",
  md: "false",
})

// RELATED KNOWLEDGE LAB FILES TABLES
const linked_pkm_permanent = await tp.user.dv_pkm_linked({
  type: "perm",
  subtype: "",
  relation: "",
  md: "false",
})

const linked_pkm_lit = await tp.user.dv_pkm_linked({
  type: "lit",
  subtype: "",
  relation: "",
  md: "false",
})

const linked_pkm_fleet = await tp.user.dv_pkm_linked({
  type: "fleet",
  subtype: "",
  relation: "",
  md: "false",
})

const linked_pkm_info = await tp.user.dv_pkm_linked({
  type: "info",
  subtype: "",
  relation: "",
  md: "false",
})
```

### Language Reference

<!-- Recreate the code with links to files  -->

### Explanation

```javascript

```

### Use Cases

#### Files

<!-- Files containing the snippet  -->

1. [[50_00_project|General Project Template]]
2. [[51_00_parent_task|General Parent Task Template]]
3. [[80_00_pkm_tree|General Knowledge Tree Template]]
4. [[80_01_tree_category|Knowledge Tree Category Template]]
5. [[80_02_tree_branch|Knowledge Tree Branch Template]]
6. [[80_03_tree_field|Knowledge Tree Field Template]]
7. [[80_04_tree_subject|Knowledge Tree Subject Template]]
8. [[80_05_tree_topic|Knowledge Tree Topic Template]]
9. [[80_06_tree_subtopic|Knowledge Tree Subtopic Template]]
10. [[90_00_note|General Note Template]]
11. [[90_10_note_fleeting(X)|General Fleeting Note Template]]
12. [[90_11_note_quote|Quote Fleeting Note Template]]
13. [[90_12_note_idea|Idea Fleeting Note Template]]
14. [[90_20_note_literature(X)|General Literature Note Template]]
15. [[90_30_note_lit_qec(X)|Literature QEC Note Template]]
16. [[90_31_note_question|QEC Question Note Template]]
17. [[90_32_note_evidence|QEC Evidence Note Template]]
18. [[90_33_note_conclusion|QEC Conclusion Note Template]]
19. [[90_40_note_lit_psa(X)|PSA Note Template]]
20. [[90_41_note_problem|PSA Problem Note Template]]
21. [[90_42_note_steps|PSA Steps Note Template]]
22. [[90_43_note_answer|PSA Answer Note Template]]
23. [[90_50_note_info(X)|General Info Note Template]]
24. [[90_51_note_concept|Concept Note Template]]
25. [[90_52_note_definition|Definition Note Template]]

#### In Conjunction

<!-- Snippets used together with this snippet  -->

1. [[note_related_section|Note Related Section Dataview Tables]]
2. [[50_00_proj_related_section|Project Related Section Dataview Tables]]

---

## Related

### Script Link

<!-- Link the user template script here -->  

1. [[dv_pkm_linked.js]]

### Outgoing Snippet Links

<!-- Link related snippet here -->

1. [[dv_linked_file(X)|Linked File Dataview Table]]
2. [[dv_task_linked|Linked Tasks and Events Files Dataview Table]]
3. [[dv_dir_linked|Linked Directory Files Dataview Table]]
4. [[dv_lib_linked|Linked Library Files Dataview Table]]

### All Snippet Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Snippet,
	Description AS Description,
	file.etags AS Tags
WHERE 
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "snippet"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

### Outgoing Function Links

<!-- Link related functions here -->

### All Function Links

<!-- Query limit 10  -->

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Function,
	Definition AS Definition
WHERE 
	file.frontmatter.file_class = "pkm_code"
	AND file.frontmatter.type = "function"
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
SORT file.name
LIMIT 10
```

---

## Resources

---

## Flashcards
