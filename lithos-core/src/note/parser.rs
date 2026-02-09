//! Markdown parsing adapter for extracting structural note data.
//!
//! Responsible for converting raw markdown strings into domain entities
//! like lists, tasks, and headings using `pulldown-cmark`.

//! Markdown adapter for extracting note lists and tasks.

use std::ops::Range;

use pulldown_cmark::{Event, Tag, TagEnd};

use super::{
    aggregate::Note,
    error::NoteError,
    list::{List, ListItem, ListType},
    task::Task,
    types::SourceByteOffset,
};
use crate::{
    config::task::{StatusSymbol, TaskConfig},
    fs::MarkdownParser,
};

/// Markdown parser for list and task extraction.
///
/// `NoteParser` uses `pulldown-cmark` to traverse a markdown document and
/// extract structural elements such as headings, lists, and tasks. It is
/// bound to a specific [`TaskConfig`] which defines the rules for task
/// promotion and metadata parsing.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::parser::NoteParser;
/// # use lithos_core::config::task::TaskConfig;
/// let config = TaskConfig::default();
/// let parser = NoteParser::new(&config);
///
/// let markdown = "- [ ] #task Review PR";
/// let (lists, tasks) = parser.parse_lists_and_tasks(markdown).unwrap();
///
/// assert_eq!(tasks.len(), 1);
/// ```
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct NoteParser<'config> {
    config: &'config TaskConfig,
}

impl<'config> NoteParser<'config> {
    /// Creates a new [`NoteParser`] bound to the provided task configuration.
    #[inline]
    #[must_use]
    pub const fn new(config: &'config TaskConfig) -> Self {
        Self {
            config,
        }
    }

    /// Parses markdown into lists and promoted tasks.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when task promotion or list extraction fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::parser::NoteParser;
    /// # use lithos_core::config::task::TaskConfig;
    /// let config = TaskConfig::default();
    /// let parser = NoteParser::new(&config);
    /// let (lists, tasks) = parser.parse_lists_and_tasks("- [ ] task").unwrap();
    /// ```
    #[inline]
    pub fn parse_lists_and_tasks(
        &self,
        markdown: &str,
    ) -> Result<ParseOutcome, NoteError> {
        let parser = MarkdownParser::with_tasklists();
        let mut state = ParseState::new(self.config);

        for (event, range) in parser.parse_offsets(markdown) {
            state.handle_event(event, range)?;
        }

        Ok(state.finish())
    }

    /// Parses markdown and appends extracted lists and tasks to a note.
    ///
    /// This is the primary entry point for populating a [`Note`] aggregate
    /// from markdown source.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::{aggregate::{Note, NoteId}, parser::NoteParser};
    /// # use lithos_core::config::task::TaskConfig;
    /// let config = TaskConfig::default();
    /// let mut note = Note::new(NoteId::new(), "test.md".to_string()).unwrap();
    /// let parser = NoteParser::new(&config);
    ///
    /// parser.apply_to_note(&mut note, "- [ ] #task Review PR").unwrap();
    /// assert_eq!(note.tasks().count(), 1);
    /// ```
    #[inline]
    pub fn apply_to_note(
        &self,
        note: &mut Note,
        markdown: &str,
    ) -> Result<(), NoteError> {
        let (lists, tasks) = self.parse_lists_and_tasks(markdown)?;
        for list in lists {
            note.add_list(list);
        }
        for task in tasks {
            note.add_task(task);
        }
        Ok(())
    }
}

type ParseOutcome = (Vec<List>, Vec<Task>);

#[derive(Debug)]
struct ParseState<'config> {
    config: &'config TaskConfig,
    lists: Vec<List>,
    tasks: Vec<Task>,
    list_stack: Vec<List>,
    current_item: Option<ItemState>,
}

impl<'config> ParseState<'config> {
    fn new(config: &'config TaskConfig) -> Self {
        Self {
            config,
            lists: Vec::new(),
            tasks: Vec::new(),
            list_stack: Vec::new(),
            current_item: None,
        }
    }

    fn handle_event(
        &mut self,
        event: Event<'_>,
        range: Range<usize>,
    ) -> Result<(), NoteError> {
        match event {
            Event::Start(Tag::List(start)) => self.start_list(start)?,
            Event::End(TagEnd::List(_)) => self.end_list(),
            Event::Start(Tag::Item) => self.start_item(range.start)?,
            Event::End(TagEnd::Item) => self.end_item()?,
            Event::TaskListMarker(checked) => {
                if let Some(item) = self.current_item.as_mut() {
                    item.status = Some(status_symbol_from_marker(checked)?);
                }
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some(item) = self.current_item.as_mut() {
                    item.text.push_str(text.as_ref());
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(item) = self.current_item.as_mut() {
                    item.text.push(' ');
                }
            }
            Event::Start(_)
            | Event::End(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Rule => {}
        }

        Ok(())
    }

    fn start_list(&mut self, start: Option<u64>) -> Result<(), NoteError> {
        let depth = parse_depth(self.list_stack.len())?;
        let list_type = match start {
            Some(start) => ListType::Ordered {
                start,
            },
            None => ListType::Unordered,
        };
        let list = List::with_depth(list_type, depth);
        self.list_stack.push(list);
        Ok(())
    }

    fn end_list(&mut self) {
        if let Some(list) = self.list_stack.pop() {
            self.lists.push(list);
        }
    }

    fn start_item(&mut self, start: usize) -> Result<(), NoteError> {
        let position = parse_offset(start)?;
        self.current_item = Some(ItemState::new(position));
        Ok(())
    }

    fn end_item(&mut self) -> Result<(), NoteError> {
        let Some(item) = self.current_item.take() else {
            return Ok(());
        };
        let Some(list) = self.list_stack.last_mut() else {
            return Ok(());
        };

        let raw_text = item.text.trim();
        if let Some(status) = item.status {
            let mut task_id = None;
            if Task::should_promote(raw_text, self.config) {
                let task = Task::from_checkbox(
                    raw_text,
                    status,
                    item.position,
                    self.config,
                )?;
                task_id = Some(task.id());
                self.tasks.push(task);
            }

            list.add_item(ListItem::Checkbox {
                text: raw_text.to_owned().into_boxed_str(),
                status,
                position: item.position,
                task_id,
            });
        } else {
            list.add_item(ListItem::Plain {
                text: raw_text.to_owned().into_boxed_str(),
                position: item.position,
            });
        }

        Ok(())
    }

    fn finish(mut self) -> (Vec<List>, Vec<Task>) {
        if !self.list_stack.is_empty() {
            self.lists.append(&mut self.list_stack);
        }
        (self.lists, self.tasks)
    }
}

#[derive(Debug)]
struct ItemState {
    position: SourceByteOffset,
    text: String,
    status: Option<StatusSymbol>,
}

impl ItemState {
    fn new(position: SourceByteOffset) -> Self {
        Self {
            position,
            text: String::new(),
            status: None,
        }
    }
}

fn parse_offset(offset: usize) -> Result<SourceByteOffset, NoteError> {
    SourceByteOffset::try_from(offset).map_err(|error| {
        NoteError::Structure(format!("source offset out of range: {error}"))
    })
}

fn parse_depth(depth: usize) -> Result<u8, NoteError> {
    u8::try_from(depth).map_err(|error| {
        NoteError::Structure(format!("list depth out of range: {error}"))
    })
}

fn status_symbol_from_marker(checked: bool) -> Result<StatusSymbol, NoteError> {
    let symbol = if checked {
        'x'
    } else {
        ' '
    };
    StatusSymbol::try_new(symbol).map_err(|error| {
        NoteError::Task(format!("invalid status symbol '{symbol}': {error}"))
    })
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;
    use crate::note::aggregate::NoteId;

    #[test]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Test matches &ListItem using match ergonomics."
    )]
    fn parses_checkbox_list_and_promotes_tasks() -> Result<(), NoteError> {
        let config = TaskConfig::default();
        let parser = NoteParser::new(&config);
        let markdown = "- [ ] #task Review PR [priority:: 1]\n- [x] Buy milk\n";

        let (lists, tasks) = parser.parse_lists_and_tasks(markdown)?;
        assert_eq!(lists.len(), 1, "expected one list");
        assert_eq!(tasks.len(), 1, "expected one promoted task");

        let list = lists.first().expect("list should exist");
        assert!(matches!(list.list_type(), ListType::Unordered));
        assert_eq!(list.items().len(), 2, "expected two list items");

        let first_item = list.items().first().expect("first item");
        let (task_id, status) = match first_item {
            ListItem::Checkbox {
                task_id,
                status,
                ..
            } => (task_id, status),
            ListItem::Plain {
                ..
            } => {
                return Err(NoteError::Structure(
                    "expected checkbox list item".to_owned(),
                ));
            }
        };
        assert_eq!(status.value(), ' ', "expected unchecked status");
        assert!(task_id.is_some(), "expected promoted task id");

        let task = tasks.first().expect("task should exist");
        assert_eq!(task_id, &Some(task.id()));
        Ok(())
    }

    #[test]
    fn captures_list_depths() -> Result<(), NoteError> {
        let config = TaskConfig::default();
        let parser = NoteParser::new(&config);
        let markdown = "1. First\n   - [ ] #task Nested\n";

        let (lists, _tasks) = parser.parse_lists_and_tasks(markdown)?;
        assert_eq!(lists.len(), 2, "expected two lists");

        let ordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Ordered { .. }))
            .expect("ordered list should exist");
        assert_eq!(ordered.depth(), 0, "ordered list should be top-level");

        let unordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list should exist");
        assert_eq!(unordered.depth(), 1, "nested list should have depth 1");
        Ok(())
    }

    #[test]
    fn apply_to_note_appends_lists_and_tasks() -> Result<(), NoteError> {
        let config = TaskConfig::default();
        let parser = NoteParser::new(&config);
        let markdown = "- [ ] #task Review PR\n";

        let mut note = Note::new(NoteId::new(), "notes/test.md".to_owned())?;

        parser.apply_to_note(&mut note, markdown)?;

        assert_eq!(note.lists().count(), 1, "note should have 1 list");
        assert_eq!(note.tasks().count(), 1, "note should have 1 task");
        Ok(())
    }
}
