use crate::{
    config::task::StatusSymbol,
    note::{list::ListDepth, position::SourceByteOffset, task::TaskId},
};

#[derive(Debug, Default)]
pub(super) struct InlineText {
    buffer: String,
}

impl InlineText {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn push_text(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    pub(super) fn push_break(&mut self) {
        if !self.buffer.ends_with(' ') {
            self.buffer.push(' ');
        }
    }

    pub(super) fn finish(self) -> String {
        self.buffer
    }
}

#[derive(Debug)]
pub(super) struct ListItemBuilder {
    position: SourceByteOffset,
    depth: ListDepth,
    text: InlineText,
    is_checkbox: bool,
    status_symbol: Option<char>,
}

impl ListItemBuilder {
    pub(super) fn new(position: SourceByteOffset, depth: ListDepth) -> Self {
        Self {
            position,
            depth,
            text: InlineText::new(),
            is_checkbox: false,
            status_symbol: None,
        }
    }

    pub(super) fn mark_as_checkbox(&mut self, checked: bool) {
        self.is_checkbox = true;
        self.status_symbol = Some(if checked {
            'x'
        } else {
            ' '
        });
    }

    pub(super) const fn position(&self) -> SourceByteOffset {
        self.position
    }

    pub(super) const fn depth(&self) -> ListDepth {
        self.depth
    }

    pub(super) const fn is_checkbox(&self) -> bool {
        self.is_checkbox
    }

    pub(super) const fn status_symbol(&self) -> Option<char> {
        self.status_symbol
    }

    pub(super) fn add_text(&mut self, text: &str) {
        self.text.push_text(text);
    }

    pub(super) fn add_break(&mut self) {
        self.text.push_break();
    }

    pub(super) fn into_text(self) -> String {
        self.text.finish()
    }
}

pub(super) struct ListItemRecord {
    position: SourceByteOffset,
    depth: ListDepth,
    parent: Option<SourceByteOffset>,
    status: Option<StatusSymbol>,
    task_id: Option<TaskId>,
}

impl ListItemRecord {
    pub(super) const fn new(
        position: SourceByteOffset,
        depth: ListDepth,
        parent: Option<SourceByteOffset>,
        status: Option<StatusSymbol>,
        task_id: Option<TaskId>,
    ) -> Self {
        Self {
            position,
            depth,
            parent,
            status,
            task_id,
        }
    }

    pub(super) const fn position(&self) -> SourceByteOffset {
        self.position
    }

    pub(super) const fn depth(&self) -> ListDepth {
        self.depth
    }

    pub(super) const fn parent(&self) -> Option<SourceByteOffset> {
        self.parent
    }

    pub(super) const fn status(&self) -> Option<StatusSymbol> {
        self.status
    }

    pub(super) const fn task_id(&self) -> Option<TaskId> {
        self.task_id
    }
}
