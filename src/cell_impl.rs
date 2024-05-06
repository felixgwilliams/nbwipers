use rustc_hash::FxHashSet;
use serde_json::Value;

use crate::schema::{Cell, CodeCell, SourceValue};

impl SourceValue {
    fn is_empty(&self) -> bool {
        match self {
            Self::String(ref s) => s.trim().is_empty(),
            Self::StringArray(ref s_vec) => s_vec.iter().all(|s| s.trim().is_empty()),
        }
    }
}

impl CodeCell {
    pub fn is_clear_outputs(&self) -> bool {
        self.outputs.is_empty()
    }
    pub fn is_clear_exec_count(&self) -> bool {
        let clear_exec_count = self.execution_count.is_none();

        let output_exec_counts = self
            .outputs
            .iter()
            .filter_map(|v| v.as_object())
            .filter_map(|x| x.get("execution_count"))
            .any(|v| v.as_number().is_some());
        clear_exec_count && !output_exec_counts
    }

    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }
    pub fn clear_counts(&mut self) {
        self.execution_count = None;
        self.outputs
            .iter_mut()
            .filter_map(|v| v.as_object_mut())
            .for_each(|x| {
                x.insert("execution_count".into(), Value::Null);
            });
    }
    pub fn should_clear_output(&self, drop_output: bool, strip_init_cell: bool) -> bool {
        // drop_output
        let Some(cell_metadata) = self.metadata.as_object() else {
            return drop_output;
        };
        if let Some(init_cell) = cell_metadata.get("init_cell") {
            return !init_cell.as_bool().unwrap_or(false) || strip_init_cell;
        };

        if drop_output {
            let keep_output_metadata = cell_metadata.contains_key("keep_output");
            let keep_output_tags = self
                .metadata
                .as_object()
                .and_then(|x| x.get("tags"))
                .and_then(|x| x.as_array())
                .is_some_and(|x| x.iter().any(|s| s.as_str() == Some("keep_output")));
            !(keep_output_metadata || keep_output_tags)
        } else {
            false
        }
    }
}

impl Cell {
    pub const fn as_codecell(&self) -> Option<&CodeCell> {
        if let Self::Code(codecell) = self {
            Some(codecell)
        } else {
            None
        }
    }
    pub fn as_codecell_mut(&mut self) -> Option<&mut CodeCell> {
        if let Self::Code(codecell) = self {
            Some(codecell)
        } else {
            None
        }
    }
    pub fn is_clear_id(&self, cell_number: usize) -> bool {
        let id = self.get_id();
        id.is_none() || id.as_ref().is_some_and(|id| id == &cell_number.to_string())
    }

    pub const fn get_source(&self) -> &SourceValue {
        match self {
            Self::Code(ref c) => &c.source,
            Self::Markdown(ref c) => &c.source,
            Self::Raw(ref c) => &c.source,
        }
    }

    pub const fn get_metadata(&self) -> &Value {
        match self {
            Self::Code(ref c) => &c.metadata,
            Self::Markdown(ref c) => &c.metadata,
            Self::Raw(ref c) => &c.metadata,
        }
    }
    pub fn get_metadata_mut(&mut self) -> &mut Value {
        match self {
            Self::Code(ref mut c) => &mut c.metadata,
            Self::Markdown(ref mut c) => &mut c.metadata,
            Self::Raw(ref mut c) => &mut c.metadata,
        }
    }
    pub const fn get_id(&self) -> &Option<String> {
        match self {
            Self::Code(ref c) => &c.id,
            Self::Markdown(ref c) => &c.id,
            Self::Raw(ref c) => &c.id,
        }
    }
    pub fn set_id(&mut self, new_id: Option<String>) -> Option<String> {
        let prev_id = match self {
            Self::Code(c) => c.id.clone(),
            Self::Markdown(c) => c.id.clone(),
            Self::Raw(c) => c.id.clone(),
        };
        match self {
            Self::Code(ref mut c) => c.id = new_id,
            Self::Markdown(ref mut c) => c.id = new_id,
            Self::Raw(ref mut c) => c.id = new_id,
        };
        prev_id
    }

    pub fn should_drop(
        &self,
        drop_empty_cells: bool,
        drop_tagged_cells: &FxHashSet<String>,
    ) -> bool {
        if drop_empty_cells && self.get_source().is_empty() {
            return true;
        }
        if drop_tagged_cells.is_empty() {
            return false;
        }
        let tags = self
            .get_metadata()
            .as_object()
            .and_then(|x| x.get("tags"))
            .and_then(|x| x.as_array());

        tags.map_or(false, |tags| {
            tags.iter()
                .filter_map(|v| v.as_str())
                .any(|s| drop_tagged_cells.contains(s))
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_empty_single_string() {
        let sv = SourceValue::String("  ".into());
        assert!(sv.is_empty());
    }

    #[test]
    fn test_clear_cell_without_meta() {
        let cell = CodeCell {
            execution_count: None,
            id: None,
            metadata: json!([]),
            outputs: vec![],
            source: SourceValue::StringArray(vec![]),
        };
        assert!(cell.should_clear_output(true, true));
        assert!(!cell.should_clear_output(false, true));
    }
}
