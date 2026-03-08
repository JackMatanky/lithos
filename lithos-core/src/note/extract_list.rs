//! List extraction from markdown event streams.
//!
//! Builds ordered/unordered lists from pulldown-cmark list events, tracks list
//! depth, and captures list items with source offsets. Checkbox items may be
//! promoted to tasks when configured promotion tags are present; the list item
//! retains the task id linkage.

use std::ops::Range;

use pulldown_cmark::{Event, Tag as CmarkTag, TagEnd};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::{
    config::{aggregate::Config, task::StatusSymbol},
    note::{
        error::NoteError,
        list::{List, ListDepth, ListItem, ListType},
        position::SourceByteOffset,
        tag::scan_tags,
        task::{Task, TaskBuilder},
    },
};

/// Output from list extraction - either a list or a promoted task.
#[derive(Debug)]
pub enum ExtractionOutput {
    /// A complete list with items.
    List(List),
    /// A task promoted from a checkbox item with a promotion tag.
    Task(Box<Task>),
}

/// Extractor for markdown lists, checkboxes, and task promotion.
///
/// Processes markdown list events and builds domain `List` entities.
/// Handles nested lists by maintaining a stack. Checkboxes with promotion
/// tags will be extracted as separate `Task` entities.
///
/// ## Task Promotion
///
/// When a checkbox contains a tag matching the configured task promotion tags,
/// it will be promoted to a `Task` entity and emitted immediately. The list
/// item will link to the task via `task_id`.
pub struct ListExtractor<'config> {
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

    fn add_text(&mut self, text: &str) {
        self.text.push_str(text);
    }
}

impl<'config> ListExtractor<'config> {
    /// Creates a new list extractor bound to the provided configuration.
    ///
    /// This is the standard constructor for creating a list extractor.
    #[inline]
    pub(super) const fn new(config: &'config Config) -> Self {
        Self {
            config,
            list_stack: Vec::new(),
            current_item: None,
        }
    }

    /// Adds a completed item to the current list, potentially promoting to
    /// task.
    ///
    /// Takes `ItemBuilder` by value since it's consumed during list item
    /// construction.
    ///
    /// Returns `Some(Task)` if the checkbox was promoted to a task.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "ItemBuilder is intentionally consumed to build ListItem"
    )]
    fn add_item_to_list(
        &mut self,
        item: ItemBuilder,
    ) -> Result<Option<Box<Task>>, NoteError> {
        let mut promoted_task = None;
        let mut status = None;

        if item.is_checkbox {
            // Check for task promotion
            let checkbox_status =
                StatusSymbol::try_new(item.status_symbol.unwrap_or(' '))?;
            if !self.config.task().tags().is_empty() {
                let tags = scan_tags(&item.text);
                let builder = TaskBuilder::new(self.config.task());
                promoted_task = builder.promote_from_checkbox(
                    &item.text,
                    tags,
                    checkbox_status,
                    item.position,
                )?;
            }
            status = Some(checkbox_status);
        }

        if let Some(list) = self.list_stack.last_mut() {
            if let Some(checkbox_status) = status {
                let task_id = promoted_task.as_ref().map(Task::id);
                list.add_item(ListItem::Checkbox {
                    text: item.text.trim().into(),
                    status: checkbox_status,
                    position: item.position,
                    task_id,
                });
            } else {
                list.add_item(ListItem::Plain {
                    text: item.text.trim().into(),
                    position: item.position,
                });
            }
        }

        Ok(promoted_task.map(Box::new))
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
        text: &str,
        range: &Range<usize>,
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
                    item.add_text(text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::Item) => {
                // Complete current item and add to list (potentially promoting
                // to task)
                if let Some(item) = self.current_item.take()
                    && let Some(task) = self.add_item_to_list(item)?
                {
                    // Checkbox was promoted - emit task immediately
                    return Ok(ExtractionState::Emit(ExtractionOutput::Task(
                        task,
                    )));
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
    use pulldown_cmark::{Event, Tag as CmarkTag, TagEnd};

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        note::{
            list::ListType,
            reader::{ExtractionContext, ExtractionState},
        },
    };

    #[test]
    fn extracts_plain_unordered_list() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        let result1 = extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();
        assert!(matches!(result1, ExtractionState::Continue));

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Item text
        extractor
            .process(
                &Event::Text("Buy milk".into()),
                "Buy milk",
                &(4..12),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(12..13), &ctx)
            .unwrap();

        // End list - should emit
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(13..14), &ctx)
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
            .process(&Event::Start(CmarkTag::List(Some(1))), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Item text
        extractor
            .process(
                &Event::Text("First item".into()),
                "First item",
                &(4..14),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(14..15), &ctx)
            .unwrap();

        // End list - should emit
        let result = extractor
            .process(&Event::End(TagEnd::List(true)), "", &(20..21), &ctx)
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
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (unchecked)
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..7), &ctx)
            .unwrap();

        // Text
        extractor
            .process(
                &Event::Text("Buy milk".into()),
                "Buy milk",
                &(7..15),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(15..16), &ctx)
            .unwrap();

        // End list
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(16..17), &ctx)
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
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (checked)
        extractor
            .process(&Event::TaskListMarker(true), "", &(4..7), &ctx)
            .unwrap();

        // Text
        extractor
            .process(
                &Event::Text("Done task".into()),
                "Done task",
                &(7..16),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(16..17), &ctx)
            .unwrap();

        // End list
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(17..18), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list");
        };
        let item = list.items().next().unwrap();
        assert!(item.status().is_some()); // Is a checkbox
    }

    #[test]
    fn checkbox_without_promotion_tag_stays_as_list_item() {
        let config = test_config();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (unchecked)
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..5), &ctx)
            .unwrap();

        // Text without promotion tag
        extractor
            .process(
                &Event::Text("Buy milk".into()),
                "Buy milk",
                &(5..13),
                &ctx,
            )
            .unwrap();

        // End item
        extractor
            .process(&Event::End(TagEnd::Item), "", &(13..14), &ctx)
            .unwrap();

        // End list - should emit List (not Task)
        let result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(14..15), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = result else {
            panic!("Expected list, not task");
        };

        let item = list.items().next().unwrap();
        assert!(matches!(item, ListItem::Checkbox { .. }));
        assert!(item.task_id().is_none()); // Not promoted
    }

    #[test]
    fn checkbox_with_promotion_tag_becomes_task() {
        let config = test_config_with_task_tag();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker (unchecked)
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..5), &ctx)
            .unwrap();

        // Text with promotion tag
        extractor
            .process(
                &Event::Text("#task Review PR".into()),
                "#task Review PR",
                &(5..20),
                &ctx,
            )
            .unwrap();

        // End item - should emit Task immediately
        let result = extractor
            .process(&Event::End(TagEnd::Item), "", &(20..21), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::Task(task)) = result else {
            panic!("Expected task emission on item end");
        };

        assert_eq!(task.text(), "Review PR");
        assert!(task.tags().any(|t| t.full_path() == "task"));
    }

    #[test]
    fn promoted_task_links_to_list_item() {
        let config = test_config_with_task_tag();
        let mut extractor = ListExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start list
        extractor
            .process(&Event::Start(CmarkTag::List(None)), "", &(0..2), &ctx)
            .unwrap();

        // Start item
        extractor
            .process(&Event::Start(CmarkTag::Item), "", &(2..4), &ctx)
            .unwrap();

        // Checkbox marker
        extractor
            .process(&Event::TaskListMarker(false), "", &(4..5), &ctx)
            .unwrap();

        // Text with promotion tag
        extractor
            .process(
                &Event::Text("#task Deploy".into()),
                "#task Deploy",
                &(5..17),
                &ctx,
            )
            .unwrap();

        // End item - emit task
        let task_result = extractor
            .process(&Event::End(TagEnd::Item), "", &(17..18), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::Task(task)) = task_result
        else {
            panic!("Expected task");
        };
        let task_id = task.id();

        // End list - emit list
        let list_result = extractor
            .process(&Event::End(TagEnd::List(false)), "", &(18..19), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(ExtractionOutput::List(list)) = list_result
        else {
            panic!("Expected list");
        };

        // Verify link
        let item = list.items().next().unwrap();
        assert_eq!(item.task_id(), Some(task_id));
    }

    fn test_config() -> Config {
        let raw = RawConfig::default();
        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }

    fn test_config_with_task_tag() -> Config {
        use crate::config::raw::RawTaskConfig;

        let raw = RawConfig {
            task: Some(RawTaskConfig {
                task_tags: Some(vec!["#task".into()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }
}
