//! Integration tests for parser lexical policy invariants.

use lithos_core::{config::task::TaskConfigSpec, note::parser::MarkdownParser};

fn task_spec_fixture() -> TaskConfigSpec {
    TaskConfigSpec::new(
        true,
        true,
        vec!['\u{1f4c5}', '\u{2705}', '\u{23f0}', '\u{1f6eb}', '\u{23f3}']
            .into(),
        vec!["task".into()].into(),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_tags_inside_link_label_and_code_and_math() {
        let md = "[x #link](http://example.test) `#code` $#math$ #ok";
        let raw =
            MarkdownParser::parse(md, &task_spec_fixture()).expect("parse");

        let tags: Vec<_> =
            raw.tags.iter().map(|tag| tag.value.as_ref()).collect();
        assert_eq!(tags, vec!["#ok"]);
    }

    #[test]
    fn excludes_block_ref_inside_link_label() {
        let md = "[x ^inside](http://example.test) ^outside";
        let raw =
            MarkdownParser::parse(md, &task_spec_fixture()).expect("parse");

        let refs: Vec<_> =
            raw.block_refs.iter().map(|r| r.id.as_ref()).collect();
        assert_eq!(refs, vec!["outside"]);
    }

    #[test]
    fn excludes_tags_inside_list_item_link_labels() {
        let md = "- item [x #hidden](http://example.test) #visible";
        let raw =
            MarkdownParser::parse(md, &task_spec_fixture()).expect("parse");

        let tags: Vec<_> =
            raw.tags.iter().map(|tag| tag.value.as_ref()).collect();
        assert!(tags.contains(&"#visible"));
        assert!(!tags.contains(&"#hidden"));
    }
}
