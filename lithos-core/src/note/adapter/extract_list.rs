//! List extraction from markdown event streams.
//!
//! Extracts plain lists, checkboxes, and promotes checkboxes to tasks
//! when they match promotion criteria.

use std::ops::Range;

use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::{
    config::aggregate::Config,
    note::{
        error::NoteError,
        list::{List, ListDepth, ListItem, ListType},
        position::SourceByteOffset,
        task::Task,
    },
};

/// Output from list extraction - either a list or a promoted task.
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "Variants accessed in tests; clippy doesn't track cfg(test) usage"
)]
pub enum ExtractionOutput {
    /// A complete list with items.
    List(List),
    /// A task promoted from a checkbox item.
    Task(Box<Task>),
}

/// Extractor for markdown lists, checkboxes, and task promotion.
///
/// Processes markdown list events and builds domain `List` entities.
/// Handles nested lists by maintaining a stack. Checkboxes with promotion
/// tags will be extracted as separate `Task` entities.
pub struct ListExtractor<'config> {
    #[expect(dead_code, reason = "Will be used for task promotion in Cycle 3")]
    config: &'config Config,
    list_stack: Vec<List>,
    current_item: Option<ItemBuilder>,
}

/// Builder for accumulating list item data during extraction.
struct ItemBuilder {
    position: SourceByteOffset,
    text: String,
    is_checkbox: bool,
    status_symbol: Option<char>,
}

impl ItemBuilder {
    fn new(position: SourceByteOffset) -> Self {
        Self {
            position,
            text: String::new(),
            is_checkbox: false,
            status_symbol: None,
        }
    }

    fn mark_as_checkbox(&mut self, checked: bool) {
        self.is_checkbox = true;
        self.status_symbol = Some(if checked {
            'x'
        } else {
            ' '
        });
    }
}

impl<'config> ListExtractor<'config> {
    /// Creates a new list extractor bound to the provided configuration.
    ///
    /// This is the standard constructor for creating a list extractor.
    #[inline]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Used in tests; will be used by reader.rs orchestration"
        )
    )]
    pub(super) const fn new(config: &'config Config) -> Self {
        Self {
            config,
            list_stack: Vec::new(),
            current_item: None,
        }
    }

    /// Adds a completed item to the current list.
    ///
    /// Takes `ItemBuilder` by value since it's consumed during list item
    /// construction.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "ItemBuilder is intentionally consumed to build ListItem"
    )]
    fn add_item_to_list(&mut self, item: ItemBuilder) -> Result<(), NoteError> {
        if let Some(list) = self.list_stack.last_mut() {
            if item.is_checkbox {
                use crate::config::task::StatusSymbol;
                let status =
                    StatusSymbol::try_new(item.status_symbol.unwrap_or(' '))?;
                list.add_item(ListItem::Checkbox {
                    text: item.text.trim().into(),
                    status,
                    position: item.position,
                    task_id: None,
                });
            } else {
                list.add_item(ListItem::Plain {
                    text: item.text.trim().into(),
                    position: item.position,
                });
            }
        }
        Ok(())
    }
}

impl Extractor for ListExtractor<'_> {
    type Error = NoteError;
    type Output = ExtractionOutput;

    fn finish(self) -> Result<Vec<ExtractionOutput>, NoteError> {
        // Flush any incomplete lists
        Ok(self.list_stack.into_iter().map(ExtractionOutput::List).collect())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Event preferred for clarity"
    )]
    fn process(
        &mut self,
        event: &Event<'_>,
        text: CowStr<'_>,
        range: Range<usize>,
        _ctx: &ExtractionContext,
    ) -> Result<ExtractionState<ExtractionOutput>, NoteError> {
        match event {
            Event::Start(CmarkTag::List(start)) => {
                // Start a new list
                let depth = ListDepth::try_new(self.list_stack.len())?;
                let list_type = match *start {
                    Some(start_num) => ListType::Ordered {
                        start: start_num,
                    },
                    None => ListType::Unordered,
                };
                self.list_stack.push(List::with_depth(list_type, depth));
                Ok(ExtractionState::Continue)
            }

            Event::Start(CmarkTag::Item) => {
                // Start a new list item
                let position = SourceByteOffset::try_from_usize(range.start)?;
                self.current_item = Some(ItemBuilder::new(position));
                Ok(ExtractionState::Continue)
            }

            Event::TaskListMarker(checked) => {
                // Mark current item as checkbox
                if let Some(item) = self.current_item.as_mut() {
                    item.mark_as_checkbox(*checked);
                }
                Ok(ExtractionState::Continue)
            }

            Event::Text(_) => {
                // Accumulate text in current item
                if let Some(item) = self.current_item.as_mut() {
                    item.text.push_str(&text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::Item) => {
                // Complete current item and add to list
                if let Some(item) = self.current_item.take() {
                    self.add_item_to_list(item)?;
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::List(_)) => {
                // Complete list and emit
                if let Some(list) = self.list_stack.pop() {
                    return Ok(ExtractionState::Emit(ExtractionOutput::List(
                        list,
                    )));
                }
                Ok(ExtractionState::Continue)
            }

            // Ignore other events
            Event::Start(_)
            | Event::End(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Rule => Ok(ExtractionState::Continue),
        }
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        note::{
            adapter::reader::{ExtractionContext, ExtractionState},
            list::ListType,
        },
    };

    #[test]
    fn extracts_plain_unordered_list() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        let result1 = extractor
            .process(
                &Event::Start(CmarkTag::List(None)),
                CowStr::Borrowed(""),
                0..2,
                &ctx,
            )
            .unwrap();
        assert!(matches!(result1, ExtractionState::Continue));

        // Start item
        extractor
            .process(
                &Event::Start(CmarkTag::Item),
                CowStr::Borrowed(""),
                2..4,
                &ctx,
            )
            .unwrap();

        // Item text
        extractor
            .process(
                &Event::Text(CowStr::Borrowed("Buy milk")),
                CowStr::Borrowed("Buy milk"),
                4..12,
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(
                &Event::End(TagEnd::Item),
                CowStr::Borrowed(""),
                12..13,
                &ctx,
            )
            .unwrap();

        // End list - should emit
        let result = extractor
            .process(
                &Event::End(TagEnd::List(false)),
                CowStr::Borrowed(""),
                13..14,
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list emission");
        };
        assert_eq!(list.items().count(), 1);
        assert!(matches!(list.list_type(), ListType::Unordered));
    }

    #[test]
    fn extracts_ordered_list() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list with start number
        extractor
            .process(
                &Event::Start(CmarkTag::List(Some(1))),
                CowStr::Borrowed(""),
                0..2,
                &ctx,
            )
            .unwrap();

        // Start item
        extractor
            .process(
                &Event::Start(CmarkTag::Item),
                CowStr::Borrowed(""),
                2..4,
                &ctx,
            )
            .unwrap();

        // Item text
        extractor
            .process(
                &Event::Text(CowStr::Borrowed("First item")),
                CowStr::Borrowed("First item"),
                4..14,
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(
                &Event::End(TagEnd::Item),
                CowStr::Borrowed(""),
                14..15,
                &ctx,
            )
            .unwrap();

        // End list - should emit
        let result = extractor
            .process(
                &Event::End(TagEnd::List(true)),
                CowStr::Borrowed(""),
                20..21,
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list emission");
        };
        assert_eq!(list.items().count(), 1);
        assert!(matches!(list.list_type(), ListType::Ordered {
            start: 1
        }));
    }

    #[test]
    fn extracts_checkbox_unchecked() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(
                &Event::Start(CmarkTag::List(None)),
                CowStr::Borrowed(""),
                0..2,
                &ctx,
            )
            .unwrap();

        // Start item
        extractor
            .process(
                &Event::Start(CmarkTag::Item),
                CowStr::Borrowed(""),
                2..4,
                &ctx,
            )
            .unwrap();

        // Checkbox marker (unchecked)
        extractor
            .process(
                &Event::TaskListMarker(false),
                CowStr::Borrowed(""),
                4..7,
                &ctx,
            )
            .unwrap();

        // Text
        extractor
            .process(
                &Event::Text(CowStr::Borrowed("Buy milk")),
                CowStr::Borrowed("Buy milk"),
                7..15,
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(
                &Event::End(TagEnd::Item),
                CowStr::Borrowed(""),
                15..16,
                &ctx,
            )
            .unwrap();

        // End list
        let result = extractor
            .process(
                &Event::End(TagEnd::List(false)),
                CowStr::Borrowed(""),
                16..17,
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list");
        };
        let item = list.items().next().unwrap();
        assert!(item.status().is_some()); // Is a checkbox
        assert!(item.task_id().is_none()); // Not promoted
    }

    #[test]
    fn extracts_checkbox_checked() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(
                &Event::Start(CmarkTag::List(None)),
                CowStr::Borrowed(""),
                0..2,
                &ctx,
            )
            .unwrap();

        // Start item
        extractor
            .process(
                &Event::Start(CmarkTag::Item),
                CowStr::Borrowed(""),
                2..4,
                &ctx,
            )
            .unwrap();

        // Checkbox marker (checked)
        extractor
            .process(
                &Event::TaskListMarker(true),
                CowStr::Borrowed(""),
                4..7,
                &ctx,
            )
            .unwrap();

        // Text
        extractor
            .process(
                &Event::Text(CowStr::Borrowed("Done task")),
                CowStr::Borrowed("Done task"),
                7..16,
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(
                &Event::End(TagEnd::Item),
                CowStr::Borrowed(""),
                16..17,
                &ctx,
            )
            .unwrap();

        // End list
        let result = extractor
            .process(
                &Event::End(TagEnd::List(false)),
                CowStr::Borrowed(""),
                17..18,
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list");
        };
        let item = list.items().next().unwrap();
        assert!(item.status().is_some()); // Is a checkbox
    }

    fn test_config() -> Config {
        let raw = RawConfig::default();
        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
        )
        .expect("failed to build test config")
    }
}
