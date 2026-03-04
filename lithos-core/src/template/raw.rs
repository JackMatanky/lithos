//! Raw template input definitions.

use std::collections::HashMap;

use uuid::Uuid;

use super::{
    aggregate::{Template, TemplateName},
    block::{BlockStrategy, TemplateBlock},
    error::TemplateError,
    value::InputSpec,
};

/// Raw template definition (Input).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTemplate {
    /// Unique identity for the template. If missing, a new one is generated.
    pub id: Option<Uuid>,
    /// Unique template name.
    pub name: String,
    /// Optional parent template name for inheritance.
    pub extends: Option<String>,
    /// List of raw block definitions.
    #[serde(default)]
    pub blocks: Vec<RawTemplateBlock>,
    /// Map of input specifications.
    #[serde(default)]
    pub inputs: HashMap<String, InputSpec>,
}

/// Raw block definition (Input).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTemplateBlock {
    /// Block identifier.
    pub name: String,
    /// Block content (raw text).
    pub content: String,
    /// Composition strategy.
    #[serde(default)]
    pub strategy: BlockStrategy,
}

impl TryFrom<RawTemplate> for Template {
    type Error = TemplateError;

    #[inline]
    fn try_from(raw: RawTemplate) -> Result<Self, Self::Error> {
        let name = TemplateName::try_from(raw.name.as_str())?;
        let extends = raw
            .extends
            .map(|s| TemplateName::try_from(s.as_str()))
            .transpose()?;

        let blocks = raw
            .blocks
            .into_iter()
            .map(|b| {
                TemplateBlock::new(
                    b.name.as_str(),
                    b.content.as_str(),
                    b.strategy,
                )
            })
            .collect();

        let mut inputs = HashMap::with_capacity(raw.inputs.len());
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Populating a HashMap where insertion order does not \
                      affect correctness."
        )]
        for (k, v) in raw.inputs {
            let input_name =
                crate::template::aggregate::InputName::try_from(k.as_str())?;
            inputs.insert(input_name, v);
        }

        let mut template = Template::try_new(&name, extends, blocks, inputs)?;
        if let Some(id) = raw.id {
            template.id = id;
        }
        Ok(template)
    }
}
