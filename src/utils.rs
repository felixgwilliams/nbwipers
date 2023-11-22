use crate::extra_keys::ExtraKey;
use crate::schema::{Cell, RawNotebook};
use itertools::Itertools;
use serde_json::Value;

// fn get_value_child_mut<'a, T: AsRef<str>>(
//     value: &'a mut Value,
//     path: &[T],
// ) -> Option<&'a mut Value> {
//     let mut cur = value;
//     for segment in path {
//         cur = cur
//             .as_object_mut()
//             .and_then(|x| x.get_mut(segment.as_ref()))?;
//     }
//     Some(cur)
// }

// pub fn pop_value_child<T: AsRef<str>>(
//     value: &mut serde_json::Value,
//     path: &[T],
// ) -> Option<serde_json::Value> {
//     let (child_label, parent_path) = path.split_last()?;
//     let parent = get_value_child_mut(value, parent_path)?;
//     parent
//         .as_object_mut()
//         .and_then(|m| m.remove(child_label.as_ref()))
// }

pub fn pop_value_child<T: AsRef<str>>(value: &mut serde_json::Value, path: &[T]) -> Option<Value> {
    let n_parts = path.len();
    let trial_key = path.iter().map(std::convert::AsRef::as_ref).join(".");
    let removed = value
        .as_object_mut()
        .and_then(|x| x.remove(trial_key.as_str()));
    if removed.is_some() {
        return removed;
    }

    for i in (1..n_parts).rev() {
        let trial_key = path[0..i].iter().map(std::convert::AsRef::as_ref).join(".");
        let tail_key = &path[i..n_parts];
        #[allow(clippy::unwrap_used)]
        if value
            .as_object()
            .is_some_and(|x| x.contains_key(trial_key.as_str()))
        {
            let inner = value
                .as_object_mut()
                .and_then(|x| x.get_mut(trial_key.as_str()))
                .unwrap();
            return pop_value_child(inner, tail_key);
        }
    }
    None
}

pub fn pop_cell_key(cell: &mut Cell, extra_key: &ExtraKey) -> Option<serde_json::Value> {
    let (cell, ExtraKey::CellMeta(cellmeta_key)) = (cell, extra_key) else {
        return None;
    };
    pop_value_child(cell.get_metadata_mut(), cellmeta_key.parts.as_slice())
}
pub fn pop_meta_key(nb: &mut RawNotebook, extra_key: &ExtraKey) -> Option<serde_json::Value> {
    let (nb, ExtraKey::Metadata(meta_key)) = (nb, extra_key) else {
        return None;
    };
    pop_value_child(&mut nb.metadata, meta_key.parts.as_slice())
}

pub fn get_value_child<'a, T: AsRef<str>>(value: &'a Value, path: &[T]) -> Option<&'a Value> {
    let n_parts = path.len();
    if n_parts == 0 {
        return Some(value);
    }

    for i in (1..=n_parts).rev() {
        let trial_key = path[0..i].iter().map(std::convert::AsRef::as_ref).join(".");
        let tail_key = &path[i..n_parts];

        if let Some(inner) = value.as_object().and_then(|x| x.get(trial_key.as_str())) {
            return get_value_child(inner, tail_key);
        }
    }
    None
}

// fn get_value_child_mut<'a, T: AsRef<str>>(
//     value: &'a mut Value,
//     path: &[T],
// ) -> Option<&'a mut Value> {
//     let n_parts = path.len();
//     if n_parts == 0 {
//         return Some(value);
//     }

//     for i in (1..=n_parts).rev() {
//         let trial_key = path[0..i].iter().map(std::convert::AsRef::as_ref).join(".");
//         let tail_key = &path[i..n_parts];

//         #[allow(clippy::unwrap_used)]
//         if value
//             .as_object()
//             .is_some_and(|x| x.contains_key(trial_key.as_str()))
//         {
//             let inner = value
//                 .as_object_mut()
//                 .and_then(|x| x.get_mut(trial_key.as_str()))
//                 .unwrap();
//             return get_value_child_mut(inner, tail_key);
//         }
//     }
//     None
// }

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_value_child_simple() {
        let metadata = json!({"keep_output": true});
        let keep_output = get_value_child(&metadata, &["keep_output"]);
        assert!(keep_output.is_some_and(|x| x.as_bool().is_some_and(|b| b)));
    }

    #[test]
    fn test_get_value_child_period() {
        let mut metadata = json!({"application/vnd.databricks.v1+cell": "bananas"});
        let key = ExtraKey::from_str("cell.metadata.application/vnd.databricks.v1+cell").unwrap();
        println!("{:?}", key.get_parts());
        println!("{key:?}");

        let output = get_value_child(&metadata, key.get_parts());
        println!("{output:?}");

        assert!(output.is_some());
        assert_eq!(output.unwrap().as_str().unwrap(), "bananas");

        pop_value_child(&mut metadata, key.get_parts());

        assert!(metadata.as_object().is_some_and(serde_json::Map::is_empty));
    }
}
