use crate::{
    config::task::StatusSymbol,
    note::{list::ListDepth, position::SourceByteOffset, task::TaskId},
};

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
