use crate::extra_keys::ExtraKey;
use crate::schema::{CodeCell, RawNotebook};
use serde_json::Value;

fn get_value_child_mut<'a, T: AsRef<str>>(
    value: &'a mut Value,
    path: &[T],
) -> Option<&'a mut Value> {
    let mut cur = value;
    for segment in path {
        cur = cur
            .as_object_mut()
            .and_then(|x| x.get_mut(segment.as_ref()))?;
    }
    Some(cur)
}
pub fn get_value_child<'a, T: AsRef<str>>(value: &'a Value, path: &[T]) -> Option<&'a Value> {
    let mut cur = value;
    for segment in path {
        cur = cur.as_object().and_then(|x| x.get(segment.as_ref()))?;
    }
    Some(cur)
}

pub fn pop_value_child<T: AsRef<str>>(
    value: &mut serde_json::Value,
    path: &[T],
) -> Option<serde_json::Value> {
    let (child_label, parent_path) = path.split_last()?;
    let parent = get_value_child_mut(value, parent_path)?;
    parent
        .as_object_mut()
        .and_then(|m| m.remove(child_label.as_ref()))
}

pub fn pop_cell_key(cell: &mut CodeCell, extra_key: &ExtraKey) -> Option<serde_json::Value> {
    let (cell, ExtraKey::CellMeta(cellmeta_key)) = (cell, extra_key) else {
        return None;
    };
    pop_value_child(&mut cell.metadata, cellmeta_key.parts.as_slice())
}
pub fn pop_meta_key(nb: &mut RawNotebook, extra_key: &ExtraKey) -> Option<serde_json::Value> {
    let (nb, ExtraKey::Metadata(meta_key)) = (nb, extra_key) else {
        return None;
    };
    pop_value_child(&mut nb.metadata, meta_key.parts.as_slice())
}
