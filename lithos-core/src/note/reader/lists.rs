use super::state::ListItemRecord;
use crate::{
    config::{aggregate::Config, task::StatusSymbol},
    note::{
        error::NoteError,
        list::{List, ListDepth, ListItem, ListItemEntry},
        position::SourceByteOffset,
        tag::scan_tags,
        task::{Task, TaskBuilder},
    },
};

type TaskPromotion = (Option<StatusSymbol>, Option<Task>);

pub(super) fn promote_task_from_item(
    is_checkbox: bool,
    status_symbol: Option<char>,
    position: SourceByteOffset,
    raw_text: &str,
    config: &Config,
) -> Result<TaskPromotion, NoteError> {
    if !is_checkbox {
        return Ok((None, None));
    }

    let symbol = status_symbol.unwrap_or(' ');
    let checkbox_status = StatusSymbol::try_new(symbol).map_err(|_error| {
        NoteError::Task(crate::note::error::TaskError::InvalidStatusSymbol {
            symbol,
            reason: "status symbol must be a single ASCII character",
        })
    })?;

    if config.task().tags().is_empty() {
        return Ok((Some(checkbox_status), None));
    }

    let tags_for_task = scan_tags(raw_text);
    let builder = TaskBuilder::new(config.task());
    let promoted = builder.promote_from_checkbox(
        raw_text,
        tags_for_task,
        checkbox_status,
        position,
    )?;
    Ok((Some(checkbox_status), promoted))
}

pub(super) fn add_list_item(
    list_stack: &mut [List],
    raw_text: &str,
    position: SourceByteOffset,
    status: Option<StatusSymbol>,
    task_id: Option<crate::note::task::TaskId>,
) {
    let Some(list) = list_stack.last_mut() else {
        return;
    };
    if let Some(checkbox_status) = status {
        list.add_item(ListItem::Checkbox {
            text: raw_text.trim().into(),
            status: checkbox_status,
            position,
            task_id,
        });
    } else {
        list.add_item(ListItem::Plain {
            text: raw_text.trim().into(),
            position,
        });
    }
}

pub(super) fn record_list_item(
    list_items: &mut Vec<ListItemEntry>,
    record: &ListItemRecord,
) {
    list_items.push(ListItemEntry::new(
        record.position(),
        record.depth(),
        record.parent(),
        record.status(),
        record.task_id(),
    ));
}

pub(super) fn parent_for_depth(
    depth: ListDepth,
    open_item_by_depth: &[SourceByteOffset],
) -> Option<SourceByteOffset> {
    let depth_index = usize::from(depth.as_u8());
    if depth_index == 0 {
        return None;
    }
    open_item_by_depth.get(depth_index.saturating_sub(1)).copied()
}
