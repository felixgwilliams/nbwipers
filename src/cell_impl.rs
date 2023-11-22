use rustc_hash::FxHashSet;
use serde_json::Value;

use crate::schema::{Cell, CodeCell, SourceValue};

impl SourceValue {
    fn is_empty(&self) -> bool {
        match self {
            SourceValue::String(ref s) => s.trim().is_empty(),
            SourceValue::StringArray(ref s_vec) => s_vec.iter().all(|s| s.trim().is_empty()),
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
    pub fn as_codecell(&self) -> Option<&CodeCell> {
        if let Cell::Code(codecell) = self {
            Some(codecell)
        } else {
            None
        }
    }
    pub fn as_codecell_mut(&mut self) -> Option<&mut CodeCell> {
        if let Cell::Code(codecell) = self {
            Some(codecell)
        } else {
            None
        }
    }
    pub fn is_clear_id(&self, cell_number: usize) -> bool {
        let id = self.get_id();
        id.is_none() || id.as_ref().is_some_and(|id| id == &cell_number.to_string())
    }

    pub fn get_source(&self) -> &SourceValue {
        match self {
            Cell::Code(ref c) => &c.source,
            Cell::Markdown(ref c) => &c.source,
            Cell::Raw(ref c) => &c.source,
        }
    }

    pub fn get_metadata(&self) -> &Value {
        match self {
            Cell::Code(ref c) => &c.metadata,
            Cell::Markdown(ref c) => &c.metadata,
            Cell::Raw(ref c) => &c.metadata,
        }
    }
    pub fn get_metadata_mut(&mut self) -> &mut Value {
        match self {
            Cell::Code(ref mut c) => &mut c.metadata,
            Cell::Markdown(ref mut c) => &mut c.metadata,
            Cell::Raw(ref mut c) => &mut c.metadata,
        }
    }
    pub fn get_id(&self) -> &Option<String> {
        match self {
            Cell::Code(ref c) => &c.id,
            Cell::Markdown(ref c) => &c.id,
            Cell::Raw(ref c) => &c.id,
        }
    }
    pub fn set_id(&mut self, new_id: Option<String>) -> Option<String> {
        let prev_id = match self {
            Cell::Code(c) => c.id.clone(),
            Cell::Markdown(c) => c.id.clone(),
            Cell::Raw(c) => c.id.clone(),
        };
        match self {
            Cell::Code(ref mut c) => c.id = new_id,
            Cell::Markdown(ref mut c) => c.id = new_id,
            Cell::Raw(ref mut c) => c.id = new_id,
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

        if let Some(tags) = tags {
            tags.iter()
                .filter_map(|v| v.as_str())
                .any(|s| drop_tagged_cells.contains(s))
        } else {
            false
        }
    }
}
